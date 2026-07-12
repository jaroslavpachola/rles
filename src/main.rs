mod ansi;
mod commands;
mod pager;
mod search;
mod source;
mod view;

use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use source::Source;

const USAGE: &str = "\
usage: rles [OPTIONS] [FILE...]

A small terminal pager. Reads FILEs, or standard input when no FILE is given.
With several files, switch between them with :n and :p.

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

    if files.is_empty() && io::stdin().is_terminal() {
        eprintln!("rles: missing filename (\"rles --help\" for help)");
        return ExitCode::FAILURE;
    }

    // Like less: when stdout is not a terminal, act as cat.
    if !io::stdout().is_terminal() {
        return match cat(&files) {
            Ok(()) => ExitCode::SUCCESS,
            Err(err) => {
                eprintln!("rles: {err}");
                ExitCode::FAILURE
            }
        };
    }

    let mut sources = Vec::new();
    for file in &files {
        match Source::from_file(file) {
            Ok(source) => sources.push(source),
            Err(err) => {
                eprintln!("rles: {err}");
                return ExitCode::FAILURE;
            }
        }
    }
    if sources.is_empty() {
        match Source::from_stdin() {
            Ok(source) => sources.push(source),
            Err(err) => {
                eprintln!("rles: {err}");
                return ExitCode::FAILURE;
            }
        }
    }

    match pager::run(sources, opts) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("rles: {err}");
            ExitCode::FAILURE
        }
    }
}

/// Copy inputs to stdout byte for byte (pipeline mode).
fn cat(files: &[String]) -> io::Result<()> {
    let mut out = io::stdout().lock();
    if files.is_empty() {
        io::copy(&mut io::stdin().lock(), &mut out)?;
        return Ok(());
    }
    for file in files {
        let bytes =
            std::fs::read(file).map_err(|e| io::Error::new(e.kind(), format!("{file}: {e}")))?;
        out.write_all(&bytes)?;
    }
    Ok(())
}
