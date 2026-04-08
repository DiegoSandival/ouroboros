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

pub fn get_ghost(genoma: u32) -> bool {
    (genoma & Genoma::GHOST_FLAG) != 0
}

pub fn set_ghost(genoma: u32, phase: bool) -> u32 {
    if phase {
        genoma | Genoma::GHOST_FLAG
    } else {
        genoma & !Genoma::GHOST_FLAG
    }
}