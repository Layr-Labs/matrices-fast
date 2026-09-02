//! `emit-deps <deps.toml>` — validate the submission's deps.toml against the
//! reviewed direct-dependency name/version allowlist and print `name=version`
//! per accepted dependency to stdout. Exit 1 on any rejection (message to
//! stderr). A missing file is empty (exit 0). Called by scripts/prepare-build.sh.

use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(path) = std::env::args().nth(1) else {
        eprintln!("usage: emit-deps <deps.toml>");
        return ExitCode::from(2);
    };
    let src = match std::fs::read_to_string(Path::new(&path)) {
        Ok(s) => s,
        Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(e) => {
            eprintln!("emit-deps: cannot read {path}: {e}");
            return ExitCode::from(1);
        }
    };
    match ssi_purity::parse_deps_toml(&src) {
        Ok(deps) => {
            for d in deps {
                println!("{}={}", d.name, d.version);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("emit-deps: {e}");
            ExitCode::from(1)
        }
    }
}
