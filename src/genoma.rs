pub struct Genoma;

impl Genoma {
    pub const LEER_SELF: u32 = 1 << 0;
    pub const LEER_ANY: u32 = 1 << 1;
    pub const ESCRIBIR_SELF: u32 = 1 << 2;
    pub const ESCRIBIR_ANY: u32 = 1 << 3;
    pub const BORRAR_SELF: u32 = 1 << 4;
    pub const BORRAR_ANY: u32 = 1 << 5;
    pub const DIFERIR: u32 = 1 << 6;
    pub const FUSIONAR: u32 = 1 << 7;
    pub const CLONAR: u32 = 1 << 8;
    pub const DOMINANTE: u32 = 1 << 9;
    pub const LEER_LIBRE: u32 = 1 << 10;
    pub const MIGRADA: u32 = 1 << 11;
    pub const GHOST_FLAG: u32 = 1 << 31;
}

pub fn normalize_genoma(genoma: u32) -> u32 {
    genoma & !Genoma::GHOST_FLAG
}

/// Parses a 4-byte little-endian genome and removes the internal ghost bit.
pub fn parse_genoma_le_bytes(bytes: [u8; 4]) -> u32 {
    normalize_genoma(u32::from_le_bytes(bytes))
}

/// Returns true when the internal ghost bit is set.
pub fn get_ghost(genoma: u32) -> bool {
    (genoma & Genoma::GHOST_FLAG) != 0
}

/// Sets or clears the internal ghost bit according to the current ring phase.
pub fn set_ghost(genoma: u32, phase: bool) -> u32 {
    if phase {
        genoma | Genoma::GHOST_FLAG
    } else {
        genoma & !Genoma::GHOST_FLAG
    }
}

#[cfg(test)]
mod tests {
    use super::{get_ghost, normalize_genoma, parse_genoma_le_bytes, set_ghost, Genoma};

    #[test]
    fn normalize_clears_ghost_flag_only() {
        let with_ghost = Genoma::LEER_SELF | Genoma::BORRAR_SELF | Genoma::GHOST_FLAG;
        let normalized = normalize_genoma(with_ghost);

        assert_eq!(normalized, Genoma::LEER_SELF | Genoma::BORRAR_SELF);
        assert!(!get_ghost(normalized));
    }

    #[test]
    fn parse_from_le_bytes_clears_ghost_flag() {
        let raw = set_ghost(Genoma::ESCRIBIR_SELF | Genoma::CLONAR, true);
        let parsed = parse_genoma_le_bytes(raw.to_le_bytes());

        assert_eq!(parsed, Genoma::ESCRIBIR_SELF | Genoma::CLONAR);
        assert!(!get_ghost(parsed));
    }
}