use std::env;
use std::fs;
use std::process::ExitCode;

mod model;
mod parser;
mod printer;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let mut json_mode = false;
    let mut path: Option<String> = None;

    for arg in &args[1..] {
        match arg.as_str() {
            "--json" => json_mode = true,
            "-h" | "--help" => {
                print_usage();
                return ExitCode::SUCCESS;
            }
            other if other.starts_with('-') => {
                eprintln!("error: unknown option '{}'", other);
                print_usage();
                return ExitCode::FAILURE;
            }
            other => {
                if path.is_some() {
                    eprintln!("error: only one input file may be given");
                    return ExitCode::FAILURE;
                }
                path = Some(other.to_string());
            }
        }
    }

    let path = match path {
        Some(p) => p,
        None => {
            print_usage();
            return ExitCode::FAILURE;
        }
    };

    let src = match fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read '{}': {}", path, e);
            return ExitCode::FAILURE;
        }
    };

    match parser::parse(&src) {
        Ok(file) => {
            if json_mode {
                print!("{}", printer::to_json(&file));
            } else {
                print!("{}", printer::pretty_print(&file));
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {}:{}", path, e);
            ExitCode::FAILURE
        }
    }
}

fn print_usage() {
    eprintln!("usage: flagfmt [--json] <file>");
}
