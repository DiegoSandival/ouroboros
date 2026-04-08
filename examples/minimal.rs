use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use ouroboros::{Celula, Config, Genoma, OuroborosDb, Result};

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ouroboros-example-{unique}"));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn main() -> Result<()> {
    let directory = temp_dir();
    let config_path = directory.join(Config::DEFAULT_FILE_NAME);
    let data_path = directory.join("example.db");
    let lock_path = directory.join("example.writer.lock");

    fs::write(&config_path, "data_path = \"example.db\"\nmax_records = 8\n")?;

    let mut db = OuroborosDb::open_from_path(&config_path)?;
    let secret = b"demo-secret";
    let salt = [1u8; 16];
    let hash = Celula::hash_secret(&salt, secret);

    let initial = Celula::new(hash, salt, Genoma::LEER_SELF | Genoma::ESCRIBIR_SELF, 10, 20, 30);

    let index = db.append(initial)?;
    println!("append -> index={index}");

    let read_plain = db.read(index)?;
    println!(
        "read -> genoma={}, x={}, y={}, z={}",
        read_plain.genoma, read_plain.x, read_plain.y, read_plain.z
    );

    let read_auth = db.read_auth(index, secret)?;
    println!(
        "read_auth -> genoma={}, x={}, y={}, z={}",
        read_auth.genoma, read_auth.x, read_auth.y, read_auth.z
    );

    db.update(index, Genoma::LEER_SELF, 100, 200, 300)?;
    let updated = db.read(index)?;
    println!(
        "update -> genoma={}, x={}, y={}, z={}",
        updated.genoma, updated.x, updated.y, updated.z
    );

    db.update_auth(index, secret, Genoma::LEER_SELF | Genoma::BORRAR_SELF, 7, 8, 9)?;
    let updated_auth = db.read_auth(index, secret)?;
    println!(
        "update_auth -> genoma={}, x={}, y={}, z={}",
        updated_auth.genoma, updated_auth.x, updated_auth.y, updated_auth.z
    );

    drop(db);

    fs::remove_file(data_path)?;
    fs::remove_file(config_path)?;
    fs::remove_file(lock_path)?;
    fs::remove_dir(directory)?;

    Ok(())
}