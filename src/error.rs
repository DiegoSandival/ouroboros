// src/error.rs
use thiserror::Error;

/// Errores publicos devueltos por OuroborosDB.
#[derive(Error, Debug)]
pub enum OuroborosError {
    /// Error de I/O al leer, escribir o abrir archivos.
    #[error("Error de entrada/salida en el disco: {0}")]
    Io(#[from] std::io::Error),

    /// Se solicito un slot fuera del rango fisico del anillo.
    #[error("Índice fuera de rango. Máximo permitido: {max}, solicitado: {requested}")]
    IndexOutOfBounds { requested: u32, max: u32 },

    /// El payload no coincide con el tamano configurado.
    #[error("Tamaño de datos incorrecto. Esperado: {expected}, recibido: {received}")]
    InvalidDataSize { expected: usize, received: usize },

    /// El sidecar `.meta` falta o no se puede interpretar.
    #[error("Archivo de metadatos faltante o corrupto. No se puede reabrir la base de datos.")]
    CorruptedMetadata,
    
    /// Error de inicializacion o incompatibilidad estructural.
    #[error("Error al cargar la configuración inicial: {0}")]
    ConfigError(String),
}

/// Alias de resultado del crate.
pub type Result<T> = std::result::Result<T, OuroborosError>;