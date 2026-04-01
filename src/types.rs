// src/types.rs

/// Bit de fase almacenado en cada slot del anillo.
///
/// El motor usa este bit para distinguir entre una vuelta y la siguiente y poder
/// reconstruir el siguiente punto de escritura al reabrir la base.
/// Solo los valores `0` y `1` tienen significado.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhaseBit(pub u8);

impl PhaseBit {
    /// Invierte la fase activa cuando el cursor completa una vuelta.
    pub fn toggle(&mut self) {
        self.0 ^= 1;
    }
}

/// Indice de un slot fisico dentro del anillo.
///
/// No es un identificador monotono global ni un orden cronologico estable.
/// Un mismo indice puede contener distintos payloads a lo largo del tiempo cuando
/// la base da vueltas y sobrescribe slots antiguos.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RecordIndex(pub u32);

/// Configuracion estructural inmutable de la base.
///
/// Una vez creada la metadata sidecar, estos valores se convierten en la forma
/// canonica de interpretar el archivo principal.
#[derive(Debug, Clone)]
pub struct OuroborosConfig {
    /// Tamano exacto del payload de usuario, en bytes.
    pub data_size: usize,
    /// Cantidad total de slots fisicos del anillo.
    pub max_records: u32,
}

impl OuroborosConfig {
    /// Devuelve el tamano en disco de cada slot.
    ///
    /// Cada slot reserva un byte adicional para el `phase bit`.
    pub fn record_size(&self) -> u64 {
        (self.data_size + 1) as u64 // 1 byte reservado para la fase
    }
}