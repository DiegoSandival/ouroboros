use std::fs;
use std::path::PathBuf;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use ouroboros::{Celula, Config, Genoma, OuroborosDb, OuroborosReader, Result};

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ouroboros-swmr-example-{unique}"));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn main() -> Result<()> {
    let directory = temp_dir();
    let config_path = directory.join(Config::DEFAULT_FILE_NAME);
    let data_path = directory.join("swmr.db");
    let lock_path = directory.join("swmr.writer.lock");

    fs::write(&config_path, "data_path = \"swmr.db\"\nmax_records = 16\n")?;

    let mut writer = OuroborosDb::open_from_path(&config_path)?;

    let second_writer = OuroborosDb::open_from_path(&config_path);
    if second_writer.is_ok() {
        return Err(std::io::Error::other("second writer should not open").into());
    }

    let secret = b"swmr-secret";
    let mut written = Vec::new();
    for i in 0..4u32 {
        let salt = [i as u8; 16];
        let hash = Celula::hash_secret(&salt, secret);
        let cell = Celula::new(hash, salt, Genoma::LEER_SELF | Genoma::ESCRIBIR_SELF, i, i + 10, i + 20);
        let index = writer.append(cell)?;
        written.push(index);
    }

    let reader_a = writer.reader()?;
    let reader_b = writer.reader()?;
    let reader_c = OuroborosReader::open_from_path(&config_path)?;

    let h1 = spawn_reader("A", reader_a, written.clone());
    let h2 = spawn_reader("B", reader_b, written.clone());
    let h3 = spawn_reader("C", reader_c, written.clone());

    h1.join().expect("reader A thread should not panic")?;
    h2.join().expect("reader B thread should not panic")?;
    h3.join().expect("reader C thread should not panic")?;

    drop(writer);

    fs::remove_file(data_path)?;
    fs::remove_file(config_path)?;
    fs::remove_file(lock_path)?;
    fs::remove_dir(directory)?;

    println!("SWMR example completed successfully");
    Ok(())
}

fn spawn_reader(name: &'static str, reader: OuroborosReader, indices: Vec<u32>) -> thread::JoinHandle<Result<()>> {
    thread::spawn(move || {
        for index in indices {
            let cell = reader.read(index)?;
            println!(
                "reader {name} -> index={index}, genoma={}, x={}, y={}, z={}",
                cell.genoma, cell.x, cell.y, cell.z
            );
        }
        Ok(())
    })
}
