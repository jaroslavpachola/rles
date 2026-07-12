mod commands;
mod pager;
mod search;
mod view;

use std::io::{self, IsTerminal, Read, Write};
use std::process::ExitCode;

const USAGE: &str = "\
usage: rles [OPTIONS] [FILE]

A small terminal pager. Reads FILE, or standard input when no FILE is given.

options:
  -N, --line-numbers    show line numbers (toggle at runtime with -N)
  -h, --help            print this help and exit
  -V, --version         print version and exit
";

fn main() -> ExitCode {
    let mut files: Vec<String> = Vec::new();
    let mut opts = pager::Options::default();
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print!("{USAGE}");
                return ExitCode::SUCCESS;
            }
            "-V" | "--version" => {
                println!("rles {}", env!("CARGO_PKG_VERSION"));
                return ExitCode::SUCCESS;
            }
            "-N" | "--line-numbers" => opts.line_numbers = true,
            _ if arg.starts_with('-') && arg.len() > 1 => {
                eprintln!("rles: unknown option: {arg}");
                return ExitCode::FAILURE;
            }
            _ => files.push(arg),
        }
    }

    let (name, content) = match read_input(files.first().map(String::as_str)) {
        Ok(input) => input,
        Err(err) => {
            eprintln!("rles: {err}");
            return ExitCode::FAILURE;
        }
    };

    // Like less: when stdout is not a terminal, act as cat.
    if !io::stdout().is_terminal() {
        let mut out = io::stdout().lock();
        return match out.write_all(content.as_bytes()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("rles: {err}");
                ExitCode::FAILURE
            }
        };
    }

    let lines: Vec<String> = content.lines().map(str::to_owned).collect();
    match pager::run(&name, &lines, opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("rles: {err}");
            ExitCode::FAILURE
        }
    }
}

fn read_input(path: Option<&str>) -> io::Result<(String, String)> {
    match path {
        Some(path) => {
            let bytes = std::fs::read(path)
                .map_err(|e| io::Error::new(e.kind(), format!("{path}: {e}")))?;
            Ok((
                path.to_owned(),
                String::from_utf8_lossy(&bytes).into_owned(),
            ))
        }
        None => {
            let stdin = io::stdin();
            if stdin.is_terminal() {
                return Err(io::Error::other(
                    "missing filename (\"rles --help\" for help)",
                ));
            }
            let mut buf = Vec::new();
            stdin.lock().read_to_end(&mut buf)?;
            Ok((
                "(stdin)".to_owned(),
                String::from_utf8_lossy(&buf).into_owned(),
            ))
        }
    }
}
