use std::path::PathBuf;

mod db;
mod server;
mod short_code;

fn main() {
    let mut exit_code = exitcode::OK;
    let arg = std::env::args().nth(1).unwrap_or("./k0r.db".to_string());

    if arg == "-h" || arg == "--help" {
        println!("k0r [/path/to/k0r.db]\tDatabase name defaults to ./k0r.db");
        exit_code = exitcode::USAGE;
    }

    let mut path = PathBuf::from(arg);

    // append k0r.db as filename if path is a directory
    if path.is_dir() {
        path.push("k0r.db");
    }

    if !path.is_file() {
        println!("DB path not found and cannot be created: {:?}", path);
        exit_code = exitcode::CANTCREAT;
    }

    if exit_code == exitcode::OK {
        let _ = server::start(path);
    } else {
        std::process::exit(exit_code);
    }
}
