//! OuroborosDB es un almacenamiento circular persistente para payloads de tamano fijo.
//!
//! La API publica expone tres piezas principales:
//!
//! - [`OuroborosConfig`]: define el tamano de payload y la capacidad del anillo.
//! - [`OuroborosDB`]: abre el archivo, recupera el cursor y permite `append`, `read` y `update`.
//! - [`RecordIndex`]: identifica slots fisicos dentro del anillo.
//!
//! Flujo de uso tipico:
//!
//! 1. Cargar o inicializar configuracion con [`OuroborosConfig::load_or_init`].
//! 2. Abrir la base con [`OuroborosDB::open`].
//! 3. Escribir con [`OuroborosDB::append`].
//! 4. Leer o actualizar slots usando [`RecordIndex`].
//!
//! `RecordIndex` representa una posicion fisica, no un identificador cronologico global.
//! Cuando el cursor completa una vuelta, los datos mas antiguos pueden ser sobrescritos.

pub mod config;
pub mod engine;
pub mod error;
pub mod types;

/// Motor principal del anillo persistente.
pub use engine::OuroborosDB;
/// Tipos de error y alias de resultado del crate.
pub use error::{OuroborosError, Result};
/// Configuracion publica y tipo para referenciar slots fisicos.
pub use types::{OuroborosConfig, RecordIndex};