//! Command line entry point (owned by task 01).

use std::fs;
use std::process::ExitCode;

pub fn run() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        None | Some("-h") | Some("--help") => {
            help();
            ExitCode::SUCCESS
        }
        Some("-V") | Some("--version") => {
            println!("soma {}", crate::version());
            ExitCode::SUCCESS
        }
        Some("parse") => parse_cmd(&args[1..]),
        Some("check") => check_cmd(&args[1..]),
        Some("run") => run_cmd(&args[1..]),
        Some(other) => {
            eprintln!("soma: unknown command `{other}`");
            help();
            ExitCode::FAILURE
        }
    }
}

fn help() {
    println!("soma {}", crate::version());
    println!();
    println!("usage:");
    println!("  soma parse <file> [--emit-psoma]   lex + parse a file");
    println!("  soma check <file>                  parse + resolve + type check");
    println!("  soma run <file>                    run a file");
    println!("  soma --version                     print version");
}

fn read(path: &str) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

fn parse_cmd(args: &[String]) -> ExitCode {
    let mut emit = false;
    let mut files = Vec::new();
    for a in args {
        if a == "--emit-psoma" {
            emit = true;
        } else {
            files.push(a.clone());
        }
    }
    let Some(path) = files.first() else {
        eprintln!("soma parse: missing file");
        return ExitCode::FAILURE;
    };
    let src = match read(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    match crate::driver::parse_source(path, &src) {
        Err(d) => {
            eprintln!("{}", crate::driver::render(&d, path, &src));
            ExitCode::FAILURE
        }
        Ok(parsed) => {
            if emit {
                print!("{}", crate::driver::emit_psoma(&parsed));
            } else {
                println!("{}: {} items", path, parsed.items.len());
            }
            ExitCode::SUCCESS
        }
    }
}

fn check_cmd(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("soma check: missing file");
        return ExitCode::FAILURE;
    };
    let src = match read(path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::FAILURE;
        }
    };
    let (_parsed, diags) = crate::driver::check_source(path, &src);
    if diags.is_empty() {
        println!("{path}: ok");
        ExitCode::SUCCESS
    } else {
        for d in &diags {
            eprintln!("{}", crate::driver::render(d, path, &src));
        }
        ExitCode::FAILURE
    }
}

fn run_cmd(args: &[String]) -> ExitCode {
    let Some(path) = args.first() else {
        eprintln!("soma run: missing file");
        return ExitCode::FAILURE;
    };
    match crate::interp::run_file(path) {
        Ok(out) => {
            print!("{}", out.stdout);
            ExitCode::SUCCESS
        }
        Err(d) => {
            eprintln!("{}", d.message);
            ExitCode::FAILURE
        }
    }
}
