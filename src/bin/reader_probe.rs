use ouroboros::OuroborosReader;

fn main() {
    let mut args = std::env::args().skip(1);
    let config_path = match args.next() {
        Some(path) => path,
        None => {
            eprintln!("usage: reader_probe <config_path> <index>");
            std::process::exit(2);
        }
    };

    let index: u32 = match args.next().and_then(|value| value.parse::<u32>().ok()) {
        Some(value) => value,
        None => {
            eprintln!("usage: reader_probe <config_path> <index>");
            std::process::exit(2);
        }
    };

    let reader = match OuroborosReader::open_from_path(&config_path) {
        Ok(reader) => reader,
        Err(error) => {
            eprintln!("reader-open-error: {error}");
            std::process::exit(1);
        }
    };

    let cell = match reader.read(index) {
        Ok(cell) => cell,
        Err(error) => {
            eprintln!("reader-read-error: {error}");
            std::process::exit(1);
        }
    };

    println!("ok {} {} {}", cell.x, cell.y, cell.z);
}
