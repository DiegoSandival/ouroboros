// src/config.rs
use crate::error::{OuroborosError, Result};
use crate::types::OuroborosConfig;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs::{ File};
use std::io::{Read, Write};
use std::path::Path;

/// Estructura interna para serializar los metadatos al archivo sidecar
#[derive(Serialize, Deserialize, Debug)]
struct MetadataFile {
    pub data_size: usize,
    pub max_records: u32,
    pub version: u8,
}

impl OuroborosConfig {
    /// Carga la configuracion de una base existente o inicializa una nueva.
    ///
    /// Comportamiento:
    ///
    /// - Si existe `<db_path>.meta`, la configuracion se lee desde ahi.
    /// - Si no existe, se intenta construir la configuracion desde variables de entorno
    ///   y se persiste inmediatamente en el sidecar `.meta`.
    ///
    /// Esto garantiza que la forma de interpretar el archivo principal no cambie entre
    /// ejecuciones aunque el entorno de la aplicacion sea distinto.
    pub fn load_or_init(db_path: &str) -> Result<Self> {
        let meta_path = format!("{}.meta", db_path);

        if Path::new(&meta_path).exists() {
            // 1. LA BASE DE DATOS EXISTE: Leer el archivo sidecar inmutable
            Self::load_from_meta(&meta_path)
        } else {
            // 2. ES NUEVA: Leer del .env y congelar
            let config = Self::from_env()?;
            config.save_to_meta(&meta_path)?;
            Ok(config)
        }
    }

    /// Lee estrictamente el sidecar de metadata e ignora el entorno.
    fn load_from_meta(meta_path: &str) -> Result<Self> {
        let mut file = File::open(meta_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        let meta: MetadataFile = serde_json::from_str(&contents)
            .map_err(|_| OuroborosError::CorruptedMetadata)?;

        Ok(OuroborosConfig {
            data_size: meta.data_size,
            max_records: meta.max_records,
        })
    }

    /// Construye una configuracion nueva a partir del entorno.
    ///
    /// Variables esperadas:
    ///
    /// - `OUROBOROS_DATA_SIZE`
    /// - `OUROBOROS_MAX_RECORDS`
    fn from_env() -> Result<Self> {
        // Intentamos cargar el .env, pero si no existe no fallamos (podrían estar exportadas en el SO)
        let _ = dotenvy::dotenv();

        let data_size_str = env::var("OUROBOROS_DATA_SIZE")
            .map_err(|_| OuroborosError::ConfigError("OUROBOROS_DATA_SIZE no definido".into()))?;
        let max_records_str = env::var("OUROBOROS_MAX_RECORDS")
            .map_err(|_| OuroborosError::ConfigError("OUROBOROS_MAX_RECORDS no definido".into()))?;

        let data_size = data_size_str.parse::<usize>()
            .map_err(|_| OuroborosError::ConfigError("DATA_SIZE debe ser un número entero".into()))?;
        let max_records = max_records_str.parse::<u32>()
            .map_err(|_| OuroborosError::ConfigError("MAX_RECORDS debe ser un número entero".into()))?;

        Ok(OuroborosConfig {
            data_size,
            max_records,
        })
    }

    /// Persiste la configuracion en el sidecar JSON `.meta`.
    fn save_to_meta(&self, meta_path: &str) -> Result<()> {
        let meta = MetadataFile {
            data_size: self.data_size,
            max_records: self.max_records,
            version: 1, // Preparado para el futuro por si cambiamos el formato
        };

        let json = serde_json::to_string_pretty(&meta)
            .map_err(|_| OuroborosError::ConfigError("Error serializando metadatos".into()))?;

        let mut file = File::create(meta_path)?;
        file.write_all(json.as_bytes())?;
        
        Ok(())
    }
}