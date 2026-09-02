//! `scan-tree <vendor_dir>` — run the transitive FFI/native scan over a
//! `cargo vendor` output dir. Exit 1 on the first rejection.
use std::path::Path;
use std::process::ExitCode;

fn main() -> ExitCode {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: scan-tree <vendor_dir>");
        return ExitCode::from(2);
    };
    match ssi_purity::scan_vendored_tree(Path::new(&dir)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("scan-tree: {e}");
            ExitCode::from(1)
        }
    }
}
