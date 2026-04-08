use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use ouroboros::{Celula, Config, Genoma, OuroborosDb};

fn temp_dir() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time should be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!("ouroboros-multiprocess-{unique}"));
    fs::create_dir_all(&path).expect("temporary directory should be created");
    path
}

fn writer_probe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_writer_lock_probe")
}

fn reader_probe_bin() -> &'static str {
    env!("CARGO_BIN_EXE_reader_probe")
}

#[test]
fn rejects_second_writer_across_processes() {
    let directory = temp_dir();
    let config_path = directory.join(Config::DEFAULT_FILE_NAME);

    fs::write(&config_path, "data_path = \"ring.db\"\nmax_records = 4\n")
        .expect("config file should be written");

    let mut first_writer = Command::new(writer_probe_bin())
        .arg(config_path.to_string_lossy().as_ref())
        .arg("1200")
        .spawn()
        .expect("first writer probe should start");

    thread::sleep(Duration::from_millis(200));

    let second_writer = Command::new(writer_probe_bin())
        .arg(config_path.to_string_lossy().as_ref())
        .arg("10")
        .output()
        .expect("second writer probe should run");

    assert!(!second_writer.status.success());

    let first_status = first_writer
        .wait()
        .expect("first writer probe should finish");
    assert!(first_status.success());

    fs::remove_dir_all(directory).expect("temporary directory should be removed");
}

#[test]
fn reader_process_can_read_while_writer_process_holds_lock() {
    let directory = temp_dir();
    let config_path = directory.join(Config::DEFAULT_FILE_NAME);

    fs::write(&config_path, "data_path = \"ring.db\"\nmax_records = 4\n")
        .expect("config file should be written");

    {
        let mut db = OuroborosDb::open_from_path(&config_path).expect("db should open");
        let cell = Celula::with_secret([1u8; 16], b"clave", Genoma::LEER_SELF, 10, 20, 30);
        db.append(cell).expect("append should succeed");
    }

    let mut writer_holder = Command::new(writer_probe_bin())
        .arg(config_path.to_string_lossy().as_ref())
        .arg("1200")
        .spawn()
        .expect("writer holder should start");

    thread::sleep(Duration::from_millis(200));

    let reader = Command::new(reader_probe_bin())
        .arg(config_path.to_string_lossy().as_ref())
        .arg("0")
        .output()
        .expect("reader probe should run");

    assert!(reader.status.success());

    let writer_status = writer_holder
        .wait()
        .expect("writer holder should finish");
    assert!(writer_status.success());

    fs::remove_dir_all(directory).expect("temporary directory should be removed");
}
