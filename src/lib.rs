// src/lib.rs

pub mod config;
pub mod engine;
pub mod error;
pub mod types;

// Re-exportamos los componentes principales para que los usuarios
// puedan usarlos directamente como: use ouroboros_db::{OuroborosDB, Result};
pub use engine::OuroborosDB;
pub use error::{OuroborosError, Result};
pub use types::{OuroborosConfig, RecordIndex};