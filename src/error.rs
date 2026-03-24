// src/error.rs
use thiserror::Error;

#[derive(Error, Debug)]
pub enum OuroborosError {
    #[error("Error de entrada/salida en el disco: {0}")]
    Io(#[from] std::io::Error),

    #[error("Índice fuera de rango. Máximo permitido: {max}, solicitado: {requested}")]
    IndexOutOfBounds { requested: u32, max: u32 },

    #[error("Tamaño de datos incorrecto. Esperado: {expected}, recibido: {received}")]
    InvalidDataSize { expected: usize, received: usize },

    #[error("Archivo de metadatos faltante o corrupto. No se puede reabrir la base de datos.")]
    CorruptedMetadata,
    
    #[error("Error al cargar la configuración inicial: {0}")]
    ConfigError(String),
}

// Un alias de Result para no tener que escribir Result<T, OuroborosError> en todos lados
pub type Result<T> = std::result::Result<T, OuroborosError>;