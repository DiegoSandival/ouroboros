use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};

use crate::cell::Celula;
use crate::config::Config;
use crate::error::{Error, Result};
use crate::genoma::{get_ghost, set_ghost};

#[derive(Debug)]
pub struct OuroborosDb {
    file: File,
    cursor: u32,
    phase: bool,
    max_records: u32,
}

impl OuroborosDb {
    pub fn open_default() -> Result<Self> {
        let config = Config::load_default()?;
        Self::open_with_config(&config)
    }

    pub fn open_from_path(path: impl AsRef<std::path::Path>) -> Result<Self> {
        let config = Config::from_path(path)?;
        Self::open_with_config(&config)
    }

    pub fn open_with_config(config: &Config) -> Result<Self> {
        let expected_size = Self::expected_file_size(config.max_records)?;

        if let Some(parent) = config.data_path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }

        let existed = config.data_path.exists();
        let mut file = OpenOptions::new()
            .create(true)
            .read(true)
            .write(true)
            .open(&config.data_path)?;

        let actual_size = file.metadata()?.len();
        let (cursor, phase) = if !existed || actual_size == 0 {
            file.set_len(expected_size)?;
            (0, false)
        } else if actual_size != expected_size {
            return Err(Error::InvalidFileSize {
                expected: expected_size,
                actual: actual_size,
            });
        } else {
            Self::recover_state(&mut file, config.max_records)?
        };

        Ok(Self {
            file,
            cursor,
            phase,
            max_records: config.max_records,
        })
    }

    pub fn max_records(&self) -> u32 {
        self.max_records
    }

    pub fn cursor(&self) -> u32 {
        self.cursor
    }

    pub fn phase(&self) -> bool {
        self.phase
    }

    pub fn append(&mut self, celula: Celula) -> Result<u32> {
        if celula.is_empty() {
            return Err(Error::InvalidCell(
                "the all-zero cell is reserved as the empty slot marker",
            ));
        }

        let adjusted = Celula {
            genoma: set_ghost(celula.genoma, self.phase),
            ..celula
        };
        self.write_raw(self.cursor, adjusted)?;

        let saved = self.cursor;
        self.cursor += 1;
        if self.cursor >= self.max_records {
            self.cursor = 0;
            self.phase = !self.phase;
        }

        Ok(saved)
    }

    pub fn read(&mut self, index: u32) -> Result<Celula> {
        self.read_raw(index)
    }

    pub fn read_auth(&mut self, index: u32, secret: &[u8]) -> Result<Celula> {
        let celula = self.read(index)?;
        let expected_hash = Celula::hash_secret(&celula.salt, secret);
        if expected_hash != celula.hash {
            return Err(Error::Unauthorized);
        }
        Ok(celula)
    }

    pub fn update(&mut self, index: u32, nuevo_genoma: u32, x: u32, y: u32, z: u32) -> Result<()> {
        let current = self.read(index)?;
        let updated = Celula {
            hash: current.hash,
            salt: current.salt,
            genoma: set_ghost(nuevo_genoma, get_ghost(current.genoma)),
            x,
            y,
            z,
        };
        self.write_raw(index, updated)
    }

    pub fn update_auth(
        &mut self,
        index: u32,
        secret: &[u8],
        nuevo_genoma: u32,
        x: u32,
        y: u32,
        z: u32,
    ) -> Result<()> {
        let current = self.read_auth(index, secret)?;
        let updated = Celula {
            hash: current.hash,
            salt: current.salt,
            genoma: set_ghost(nuevo_genoma, get_ghost(current.genoma)),
            x,
            y,
            z,
        };
        self.write_raw(index, updated)
    }

    fn expected_file_size(max_records: u32) -> Result<u64> {
        u64::from(max_records)
            .checked_mul(Celula::SIZE as u64)
            .ok_or(Error::ArithmeticOverflow)
    }

    fn offset_for(index: u32) -> Result<u64> {
        u64::from(index)
            .checked_mul(Celula::SIZE as u64)
            .ok_or(Error::ArithmeticOverflow)
    }

    fn read_raw(&mut self, index: u32) -> Result<Celula> {
        if index >= self.max_records {
            return Err(Error::IndexOutOfBounds {
                index,
                max_records: self.max_records,
            });
        }

        let mut buffer = [0u8; Celula::SIZE];
        self.file.seek(SeekFrom::Start(Self::offset_for(index)?))?;
        self.file.read_exact(&mut buffer)?;
        Ok(Celula::deserialize(buffer))
    }

    fn write_raw(&mut self, index: u32, celula: Celula) -> Result<()> {
        if index >= self.max_records {
            return Err(Error::IndexOutOfBounds {
                index,
                max_records: self.max_records,
            });
        }

        self.file.seek(SeekFrom::Start(Self::offset_for(index)?))?;
        self.file.write_all(&celula.serialize())?;
        self.file.flush()?;
        Ok(())
    }

    fn recover_state(file: &mut File, max_records: u32) -> Result<(u32, bool)> {
        for index in 0..max_records {
            let cell = Self::read_raw_from_file(file, index)?;
            if cell.is_empty() {
                return Ok((index, false));
            }
        }

        let first = Self::read_raw_from_file(file, 0)?;
        let last = Self::read_raw_from_file(file, max_records - 1)?;
        let first_phase = get_ghost(first.genoma);
        let last_phase = get_ghost(last.genoma);

        if first_phase == last_phase {
            return Ok((0, !first_phase));
        }

        let mut low = 0;
        let mut high = max_records - 1;

        while low < high {
            let mid = low + (high - low) / 2;
            let middle = Self::read_raw_from_file(file, mid)?;
            if get_ghost(middle.genoma) == first_phase {
                low = mid + 1;
            } else {
                high = mid;
            }
        }

        Ok((low, first_phase))
    }

    fn read_raw_from_file(file: &mut File, index: u32) -> Result<Celula> {
        let mut buffer = [0u8; Celula::SIZE];
        file.seek(SeekFrom::Start(Self::offset_for(index)?))?;
        file.read_exact(&mut buffer)?;
        Ok(Celula::deserialize(buffer))
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::Config;
    use crate::genoma::Genoma;

    use super::OuroborosDb;
    use crate::cell::Celula;

    fn temp_dir() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("ouroboros-db-{unique}"));
        fs::create_dir_all(&path).expect("temporary directory should be created");
        path
    }

    fn test_config(directory: &std::path::Path, max_records: u32) -> Config {
        Config {
            data_path: directory.join("ring.db"),
            max_records,
        }
    }

    #[test]
    fn append_wraps_and_recovers_cursor() {
        let directory = temp_dir();
        let config = test_config(&directory, 4);

        {
            let mut db = OuroborosDb::open_with_config(&config).expect("db should open");
            for value in 0..6u32 {
                let cell = Celula::with_secret([value as u8; 16], b"clave", Genoma::LEER_SELF, value, value + 1, value + 2);
                db.append(cell).expect("append should succeed");
            }
            assert_eq!(db.cursor(), 2);
            assert!(db.phase());
        }

        let reopened = OuroborosDb::open_with_config(&config).expect("db should reopen");
        assert_eq!(reopened.cursor(), 2);
        assert!(reopened.phase());

        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }

    #[test]
    fn read_auth_and_update_auth_preserve_phase() {
        let directory = temp_dir();
        let config = test_config(&directory, 2);
        let mut db = OuroborosDb::open_with_config(&config).expect("db should open");
        let secret = b"secreto";

        let index = db
            .append(Celula::with_secret([5u8; 16], secret, Genoma::ESCRIBIR_SELF, 1, 2, 3))
            .expect("append should succeed");

        let stored = db.read_auth(index, secret).expect("secret should match");
        assert!(!crate::get_ghost(stored.genoma));

        db.update_auth(index, secret, Genoma::BORRAR_SELF, 9, 8, 7)
            .expect("update_auth should succeed");

        let updated = db.read_auth(index, secret).expect("secret should still match");
        assert_eq!(updated.x, 9);
        assert_eq!(updated.y, 8);
        assert_eq!(updated.z, 7);
        assert!(!crate::get_ghost(updated.genoma));

        fs::remove_dir_all(directory).expect("temporary directory should be removed");
    }
}