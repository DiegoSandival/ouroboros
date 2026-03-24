// src/types.rs

/// Representa el bit de fase que garantiza la integridad matemática del anillo.
/// Solo puede valer 0 o 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseBit(pub u8);

impl PhaseBit {
    /// Invierte la fase (0 -> 1, 1 -> 0) cuando la serpiente se muerde la cola.
    pub fn toggle(&mut self) {
        self.0 ^= 1;
    }
}

/// Representa la posición lógica de un registro en la base de datos (0 hasta MAX_RECORDS - 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordIndex(pub u32);

/// Representa la configuración inmutable de la base de datos.
#[derive(Debug, Clone)]
pub struct OuroborosConfig {
    pub data_size: usize,
    pub max_records: u32,
}

impl OuroborosConfig {
    /// Calcula el tamaño real en disco de cada registro (Fase + Datos)
    pub fn record_size(&self) -> u64 {
        (self.data_size + 1) as u64 // 1 byte reservado para la fase
    }
}