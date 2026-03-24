# 🐍 OuroborosDB

![OuroborosDB Logo](images/ouroboros_db_logo.png)

**OuroborosDB** es un motor de base de datos circular (Ring Buffer) de alto rendimiento escrito en Rust. Inspirado en el mito del Ouroboros, este sistema gestiona un ciclo continuo de vida y muerte de datos: cuando se alcanza la capacidad máxima, los registros más antiguos son sobrescritos pacíficamente por los nuevos.

Diseñado para sistemas de telemetría, logs de alta disponibilidad y almacenamiento en dispositivos de recursos limitados.

## ✨ Características Principales

* **Recuperación Instantánea O(log N):** Gracias a un algoritmo de "Bit de Fase", la base de datos recupera su estado y posición del cursor casi instantáneamente tras un apagón, sin necesidad de escanear el archivo completo.
* **Concurrencia MRSW (Multiple Readers, Single Writer):** Implementa I/O posicional, permitiendo infinitas lecturas simultáneas sin bloqueos entre hilos.
* **Configuración Inmutable:** Los parámetros de creación (tamaño de celda y capacidad) se sellan en un archivo `.meta` independiente para garantizar la integridad estructural de por vida.
* **Agnóstica y Eficiente:** Trata los datos como bloques de bytes puros. Ideal para estructuras binarias, JSON serializado o métricas crudas.
* **Cero dependencias pesadas:** Construido sobre la biblioteca estándar de Rust, con un manejo de errores robusto mediante `thiserror`.

## 🛠 Instalación

Agrega OuroborosDB a tu `Cargo.toml` directamente desde tu GitHub:

    [dependencies]
    ouroboros_db = { git = "https://github.com/TU_USUARIO/ouroboros_db.git" }

## 🚀 Uso Rápido

### Configuración vía `.env`
Crea un archivo `.env` en la raíz de tu proyecto para definir las dimensiones de tu base de datos:

    OUROBOROS_DATA_SIZE=96
    OUROBOROS_MAX_RECORDS=1000000

### Ejemplo de código

    use ouroboros_db::{OuroborosDB, OuroborosConfig, RecordIndex};
    use std::sync::{Arc, RwLock};

    fn main() -> ouroboros_db::Result<()> {
        // 1. Cargar o inicializar configuración
        let config = OuroborosConfig::load_or_init("mis_datos.db")?;
        
        // 2. Abrir el motor
        let db = OuroborosDB::open("mis_datos.db", config)?;
        
        // 3. Preparar para multi-hilo
        let shared_db = Arc::new(RwLock::new(db));

        // Escritura (Append) - Requiere acceso exclusivo
        {
            let mut db_writer = shared_db.write().unwrap();
            db_writer.append(&[0u8; 96])?;
        }

        // Lectura (Read) - ¡Puede ejecutarse en múltiples hilos a la vez!
        {
            let db_reader = shared_db.read().unwrap();
            let data = db_reader.read(RecordIndex(0))?;
            println!("Dato recuperado: {:?}", data);
        }

        Ok(())
    }

## 📐 Estructura de la Célula

Cada registro en el disco tiene la siguiente estructura física:
`[ 1 byte: Bit de Fase | N bytes: Datos de usuario ]`

La fase alterna entre `0` y `1` cada vez que la base de datos completa una vuelta al archivo, permitiendo la búsqueda binaria del punto de ruptura cronológico.

## 🧪 Testing

El motor incluye pruebas automatizadas para validar la persistencia, la rotación circular de datos y la recuperación ante fallos:

    cargo test

---
Desarrollado con ❤️ en Rust.
