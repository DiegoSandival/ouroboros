// src/engine.rs
use std::fs::{File, OpenOptions};
use std::path::Path;

// Importaciones específicas del Sistema Operativo para I/O posicional
#[cfg(unix)]
use std::os::unix::fs::FileExt;
#[cfg(windows)]
use std::os::windows::fs::FileExt;

use crate::error::{OuroborosError, Result};
use crate::types::{OuroborosConfig, PhaseBit, RecordIndex};

// --- Abstracción Multiplataforma para Lectura/Escritura sin Cursor ---
trait PositionalIo {
    fn read_exact_at_pos(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()>;
    fn write_all_at_pos(&self, buf: &[u8], offset: u64) -> std::io::Result<()>;
}

#[cfg(unix)]
impl PositionalIo for File {
    fn read_exact_at_pos(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        self.read_exact_at(buf, offset)
    }
    fn write_all_at_pos(&self, buf: &[u8], offset: u64) -> std::io::Result<()> {
        self.write_all_at(buf, offset)
    }
}

#[cfg(windows)]
impl PositionalIo for File {
    fn read_exact_at_pos(&self, buf: &mut [u8], offset: u64) -> std::io::Result<()> {
        let mut read = 0;
        while read < buf.len() {
            let n = self.seek_read(&mut buf[read..], offset + read as u64)?;
            if n == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, "Fin de archivo inesperado"));
            }
            read += n;
        }
        Ok(())
    }
    fn write_all_at_pos(&self, buf: &[u8], offset: u64) -> std::io::Result<()> {
        let mut written = 0;
        while written < buf.len() {
            let n = self.seek_write(&buf[written..], offset + written as u64)?;
            if n == 0 {
                return Err(std::io::Error::new(std::io::ErrorKind::WriteZero, "Fallo al escribir en disco"));
            }
            written += n;
        }
        Ok(())
    }
}

// --- Motor Principal ---

pub struct OuroborosDB {
    file: File,
    cursor: RecordIndex,
    phase: PhaseBit,
    config: OuroborosConfig,
}

impl OuroborosDB {
    pub fn open<P: AsRef<Path>>(path: P, config: OuroborosConfig) -> Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let total_size = config.record_size() * (config.max_records as u64);

        if file.metadata()?.len() == 0 {
            file.set_len(total_size)?;
        } else if file.metadata()?.len() != total_size {
            return Err(OuroborosError::ConfigError(
                "El tamaño del archivo físico no coincide con los metadatos".into(),
            ));
        }

        let (cursor, phase) = Self::recover_state(&file, &config)?;

        Ok(Self {
            file,
            cursor,
            phase,
            config,
        })
    }

   /// Escribe secuencialmente y devuelve el índice donde se guardó el registro.
    pub fn append(&mut self, data: &[u8]) -> Result<RecordIndex> { // Cambio de firma: -> Result<RecordIndex>
        if data.len() != self.config.data_size {
            return Err(OuroborosError::InvalidDataSize {
                expected: self.config.data_size,
                received: data.len(),
            });
        }

        // 1. Guardamos el índice actual antes de mover el cursor
        let saved_index = self.cursor; 

        let mut buffer = vec![0u8; self.config.record_size() as usize];
        buffer[0] = self.phase.0;
        buffer[1..].copy_from_slice(data);

        // 2. Usamos el índice capturado para el offset
        let offset = (saved_index.0 as u64) * self.config.record_size();
        self.file.write_all_at_pos(&buffer, offset)?;

        // 3. Lógica de avance del cursor (se mantiene igual)
        self.cursor.0 += 1;
        if self.cursor.0 >= self.config.max_records {
            self.cursor.0 = 0;
            self.phase.toggle();
        }

        // 4. Devolvemos el índice donde efectivamente se escribió
        Ok(saved_index)
    }

    /// Actualiza un registro. Requiere acceso exclusivo (&mut self).
    pub fn update(&mut self, index: RecordIndex, data: &[u8]) -> Result<()> {
        if index.0 >= self.config.max_records {
            return Err(OuroborosError::IndexOutOfBounds {
                requested: index.0,
                max: self.config.max_records,
            });
        }
        if data.len() != self.config.data_size {
            return Err(OuroborosError::InvalidDataSize {
                expected: self.config.data_size,
                received: data.len(),
            });
        }

        let original_phase = Self::read_phase_bit_raw(&self.file, index.0, &self.config)?;

        let mut buffer = vec![0u8; self.config.record_size() as usize];
        buffer[0] = original_phase.0;
        buffer[1..].copy_from_slice(data);

        let offset = (index.0 as u64) * self.config.record_size();
        self.file.write_all_at_pos(&buffer, offset)?;

        Ok(())
    }

    /// ¡LA MAGIA CONCURRENTE! Ahora solo requiere un préstamo inmutable (&self).
    /// Esto permite infinitas lecturas simultáneas a nivel de aplicación.
    pub fn read(&self, index: RecordIndex) -> Result<Vec<u8>> {
        if index.0 >= self.config.max_records {
            return Err(OuroborosError::IndexOutOfBounds {
                requested: index.0,
                max: self.config.max_records,
            });
        }

        let offset = (index.0 as u64) * self.config.record_size();
        let mut buffer = vec![0u8; self.config.record_size() as usize];
        
        self.file.read_exact_at_pos(&mut buffer, offset)?;

        Ok(buffer[1..].to_vec())
    }

    // --- RECUPERACIÓN PRIVADA SIN CURSOR ---

    fn recover_state(file: &File, config: &OuroborosConfig) -> Result<(RecordIndex, PhaseBit)> {
        let mut low = 0;
        let mut high = config.max_records - 1;

        let phase_first = Self::read_phase_bit_raw(file, 0, config)?;
        let phase_last = Self::read_phase_bit_raw(file, high, config)?;

        if phase_first == phase_last {
            let mut start_phase = phase_first;
            start_phase.toggle();
            return Ok((RecordIndex(0), start_phase));
        }

        while low < high {
            let mid = low + (high - low) / 2;
            let phase_mid = Self::read_phase_bit_raw(file, mid, config)?;

            if phase_mid == phase_first {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        Ok((RecordIndex(low), phase_first))
    }

    fn read_phase_bit_raw(file: &File, index: u32, config: &OuroborosConfig) -> Result<PhaseBit> {
        let offset = (index as u64) * config.record_size();
        let mut buf = [0u8; 1];
        file.read_exact_at_pos(&mut buf, offset)?;
        Ok(PhaseBit(buf[0] & 1))
    }
}

// Añadir al FINAL de src/engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    // Helper para crear una configuración pequeña para tests
    fn test_config() -> OuroborosConfig {
        OuroborosConfig {
            data_size: 4,  // Datos muy pequeños (4 bytes)
            max_records: 5, // Capacidad muy pequeña para forzar la vuelta rápido
        }
    }

    #[test]
    fn test_append_and_read() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = test_config();
        let mut db = OuroborosDB::open(temp_file.path(), config).unwrap();

        // Escribimos un dato
        let data = vec![10, 20, 30, 40];
        db.append(&data).unwrap();

        // Leemos el dato en el índice 0
        let read_data = db.read(RecordIndex(0)).unwrap();
        assert_eq!(read_data, data);
    }

    #[test]
    fn test_ouroboros_wrap_around() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = test_config(); // max_records: 5
        let mut db = OuroborosDB::open(temp_file.path(), config).unwrap();

        // Escribimos 5 registros (llenamos la DB, fase = 0)
        for i in 0..5 {
            db.append(&[i; 4]).unwrap();
        }

        assert_eq!(db.cursor.0, 0); // El cursor debió dar la vuelta
        assert_eq!(db.phase, PhaseBit(1)); // La fase debió cambiar a 1

        // Escribimos un 6to registro (sobrescribe el índice 0)
        db.append(&[99; 4]).unwrap();

        // Verificamos que el índice 0 tiene el nuevo dato
        let read_data = db.read(RecordIndex(0)).unwrap();
        assert_eq!(read_data, vec![99, 99, 99, 99]);
    }

    #[test]
    fn test_update_preserves_phase() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = test_config();
        let mut db = OuroborosDB::open(temp_file.path(), config).unwrap();

        // Escribimos un registro
        db.append(&[1, 1, 1, 1]).unwrap();

        // Lo actualizamos
        db.update(RecordIndex(0), &[2, 2, 2, 2]).unwrap();

        // Verificamos que el dato cambió
        let read_data = db.read(RecordIndex(0)).unwrap();
        assert_eq!(read_data, vec![2, 2, 2, 2]);

        // Verificamos que el bit de fase interno NO se rompió
        let mut file = file = OpenOptions::new().read(true).open(temp_file.path()).unwrap();
        let phase = OuroborosDB::read_phase_bit_raw(&mut file, 0, &test_config()).unwrap();
        assert_eq!(phase, PhaseBit(0)); 
    }

    #[test]
    fn test_crash_recovery() {
        let temp_file = NamedTempFile::new().unwrap();
        let config = test_config(); // max_records: 5

        // Bloque 1: Simulamos un servidor corriendo
        {
            let mut db = OuroborosDB::open(temp_file.path(), config.clone()).unwrap();
            
            // Escribimos 7 registros (Llena los 5, da la vuelta, escribe 2 más)
            // Esto significa que los índices 0 y 1 tienen fase 1.
            // Los índices 2, 3 y 4 tienen fase 0.
            for i in 0..7 {
                db.append(&[i; 4]).unwrap();
            }
            // El servidor "crushea" aquí (la DB sale de scope y se cierra sin avisar)
        }

        // Bloque 2: El servidor se reinicia y reabre la DB
        let db_recovered = OuroborosDB::open(temp_file.path(), config).unwrap();

        // ¡La magia de Ouroboros! Debe saber exactamente dónde se quedó.
        // El siguiente registro a escribir debería ser el índice 2, y la fase actual debería ser 1.
        assert_eq!(db_recovered.cursor.0, 2);
        assert_eq!(db_recovered.phase, PhaseBit(1));
    }

    #[test]
fn test_append_returns_correct_index() {
    let temp_file = NamedTempFile::new().unwrap();
    let config = test_config(); // max_records: 5
    let mut db = OuroborosDB::open(temp_file.path(), config).unwrap();

    let idx0 = db.append(&[1; 4]).unwrap();
    let idx1 = db.append(&[2; 4]).unwrap();

    assert_eq!(idx0, RecordIndex(0));
    assert_eq!(idx1, RecordIndex(1));
}
}