// examples/basico.rs

use ouroboros_db::{OuroborosConfig, OuroborosDB, RecordIndex, Result};
use std::env;
use std::fs;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let db_path = "demo_ouroboros.db";

    // Limpiamos archivos de pruebas anteriores si existen (solo para esta demo)
    let _ = fs::remove_file(db_path);
    let _ = fs::remove_file(format!("{}.meta", db_path));

    // Simulamos las variables de entorno
    unsafe {
        env::set_var("OUROBOROS_DATA_SIZE", "96");
        env::set_var("OUROBOROS_MAX_RECORDS", "5");
    }

    println!("🐍 Iniciando demostración de OuroborosDB Concurrente 🐍\n");

    // Bloque 1: Creación y primera escritura (Igual que antes)
    {
        println!("--- 1. Creando base de datos (Secuencial) ---");
        let config = OuroborosConfig::load_or_init(db_path)?;
        let mut db = OuroborosDB::open(db_path, config)?;

        for i in 0..5 {
            let mut data = vec![0u8; 96];
            data[0] = i as u8; 
            db.append(&data)?;
        }
        
        let mut data = vec![0u8; 96];
        data[0] = 99; // Sobrescribe el índice 0
        db.append(&data)?;

        let mut updated_data = vec![0u8; 96];
        updated_data[0] = 42; 
        db.update(RecordIndex(2), &updated_data)?;

        println!("Servidor apagado repentinamente.\n");
    }

    // Bloque 2 y 3: Recuperación y Multi-Threading
    {
        println!("--- 2. Recuperando el sistema ---");
        let config = OuroborosConfig::load_or_init(db_path)?;
        let db = OuroborosDB::open(db_path, config)?;
        println!("Base de datos reabierta en O(log N).");

        println!("\n--- 3. Demostración de Concurrencia (Multi-Thread) ---");
        // Envolvemos la base de datos en el patrón MRSW (Arc + RwLock)
        let db_compartida = Arc::new(RwLock::new(db));
        
        // Un vector para guardar los "mangos" de nuestros hilos
        let mut hilos = vec![];

        // 1. Simulamos 3 LECTORES (ej. usuarios de una API web)
        for id_lector in 1..=3 {
            let db_clon = Arc::clone(&db_compartida);
            
            let hilo_lector = thread::spawn(move || {
                // Adquirimos candado de lectura. ¡Múltiples hilos pueden hacer esto a la vez!
                let lock_lectura = db_clon.read().unwrap();
                
                println!("📖 Lector {} inició. Leyendo índice 2...", id_lector);
                // Usamos la nueva función read() que solo requiere &self
                let datos = lock_lectura.read(RecordIndex(2)).unwrap();
                
                // Simulamos que el lector tarda un poco en procesar los datos
                thread::sleep(Duration::from_millis(100));
                
                println!("   ✅ Lector {} terminó. Dato leído: {}", id_lector, datos[0]);
            });
            hilos.push(hilo_lector);
        }

        // 2. Simulamos 1 ESCRITOR (ej. un sensor de telemetría)
        let db_clon_escritor = Arc::clone(&db_compartida);
        let hilo_escritor = thread::spawn(move || {
            // Damos tiempo a que los lectores empiecen primero
            thread::sleep(Duration::from_millis(20)); 
            
            println!("✍️  Escritor intentando guardar nuevo dato...");
            // Adquirimos candado exclusivo. Esto esperará a que los lectores actuales terminen.
            let mut lock_escritura = db_clon_escritor.write().unwrap();
            
            println!("   🔒 Escritor obtuvo acceso exclusivo. Escribiendo...");
            let mut nuevos_datos = vec![0u8; 96];
            nuevos_datos[0] = 77; // Nuevo dato a guardar
            lock_escritura.append(&nuevos_datos).unwrap();
            
            println!("   ✅ Escritor terminó de guardar (dato 77).");
        });
        hilos.push(hilo_escritor);

        // Esperamos a que todos los hilos (lectores y escritor) terminen su trabajo
        for hilo in hilos {
            hilo.join().unwrap();
        }

        // Comprobación final
        println!("\nComprobación final tras la concurrencia:");
        let lock_final = db_compartida.read().unwrap();
        let dato_nuevo = lock_final.read(RecordIndex(1)).unwrap(); // Se debió escribir en el índice 1
        println!("Dato en índice 1 (escrito por el hilo): {}", dato_nuevo[0]);
    }

    println!("\n✅ Demostración Multi-Thread finalizada con éxito.");
    Ok(())
}