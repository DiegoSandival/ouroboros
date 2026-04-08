use std::thread;
use std::time::Duration;

use ouroboros::OuroborosDb;

fn main() {
    let mut args = std::env::args().skip(1);
    let config_path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: writer_lock_probe <config_path> <hold_ms>");
            std::process::exit(2);
        }
    };

    let hold_ms: u64 = match args.next().and_then(|value| value.parse::<u64>().ok()) {
        Some(value) => value,
        None => {
            eprintln!("usage: writer_lock_probe <config_path> <hold_ms>");
            std::process::exit(2);
        }
    };

    let db = match OuroborosDb::open_from_path(&config_path) {
        Ok(db) => db,
        Err(error) => {
            eprintln!("writer-open-error: {error}");
            std::process::exit(1);
        }
    };

    let _keep_alive = db;
    thread::sleep(Duration::from_millis(hold_ms));
}
