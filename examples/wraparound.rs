use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ouroboros::{get_ghost, Celula, Config, Genoma, OuroborosDb, Result};

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ouroboros-wraparound-example-{unique}"));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn main() -> Result<()> {
    let directory = temp_dir();
    let config_path = directory.join(Config::DEFAULT_FILE_NAME);
    let data_path = directory.join("wrap.db");
    let lock_path = directory.join("wrap.writer.lock");

    fs::write(&config_path, "data_path = \"wrap.db\"\nmax_records = 3\n")?;

    let mut db = OuroborosDb::open_from_path(&config_path)?;
    let secret = b"wrap-secret";

    println!("max_records={}", db.max_records());
    println!("escribiendo 5 celdas en capacidad 3");

    for i in 0..5u32 {
        let salt = [i as u8; 16];
        let hash = Celula::hash_secret(&salt, secret);
        let cell = Celula::new(hash, salt, Genoma::LEER_SELF, i, i + 100, i + 200);
        let slot = db.append(cell)?;
        println!(
            "append #{i} -> slot={slot}, cursor={}, phase={}",
            db.cursor(),
            db.phase()
        );
    }

    println!("estado final: cursor={}, phase={}", db.cursor(), db.phase());
    println!("contenido actual de los 3 slots:");

    for slot in 0..db.max_records() {
        let cell = db.read(slot)?;
        println!(
            "slot={slot} -> x={}, y={}, z={}, ghost={}",
            cell.x,
            cell.y,
            cell.z,
            get_ghost(cell.genoma)
        );
    }

    drop(db);

    fs::remove_file(data_path)?;
    fs::remove_file(config_path)?;
    fs::remove_file(lock_path)?;
    fs::remove_dir(directory)?;

    Ok(())
}
