mod vero_lite;

use std::path::PathBuf;

use vero_ir::VeroProgram;
use vero_lite::parse_vero_lite;

fn main() {
    let args = std::env::args().collect::<Vec<_>>();
    if args.len() < 2 {
        eprintln!("usage:");
        eprintln!("  vero_cli compile <in.vero> --out <out.json>");
        std::process::exit(2);
    }
    match args[1].as_str() {
        "--version" | "-V" => {
            println!("vero {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        "compile" => {
            let in_path = args.get(2).map(PathBuf::from).unwrap_or_default();
            if in_path.as_os_str().is_empty() {
                eprintln!("missing input path");
                std::process::exit(2);
            }
            let out_path = find_arg_value(&args, "--out").map(PathBuf::from).unwrap_or_else(|| {
                let mut p = in_path.clone();
                p.set_extension("json");
                p
            });

            let src = std::fs::read_to_string(&in_path).unwrap_or_else(|e| {
                eprintln!("failed to read {}: {}", in_path.display(), e);
                std::process::exit(2);
            });
            let program: VeroProgram = match parse_vero_lite(&src) {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("parse error at line {}: {}", e.line, e.msg);
                    std::process::exit(2);
                }
            };
            if let Err(errs) = vero_ir::check_program(&program) {
                for e in &errs {
                    eprintln!("error: {e}");
                }
                std::process::exit(1);
            }
            let json = serde_json::to_string_pretty(&program).unwrap();
            if let Err(e) = std::fs::write(&out_path, json) {
                eprintln!("failed to write {}: {}", out_path.display(), e);
                std::process::exit(2);
            }
            eprintln!("wrote {}", out_path.display());
        }
        _ => {
            eprintln!("unknown command: {}", args[1]);
            std::process::exit(2);
        }
    }
}

fn find_arg_value(args: &[String], key: &str) -> Option<String> {
    let mut i = 0usize;
    while i + 1 < args.len() {
        if args[i] == key {
            return Some(args[i + 1].clone());
        }
        i += 1;
    }
    None
}
