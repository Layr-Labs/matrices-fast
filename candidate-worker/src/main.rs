//! Untrusted candidate process.
//!
//! This package is the only package that includes `src/ordering` and the
//! dependencies declared in `src/ordering/deps.toml`. The trusted parent starts
//! this executable inside bubblewrap before any of its Rust initialization or
//! contestant dependency code can run.

#[path = "../../src/ordering/mod.rs"]
mod ordering;

#[cfg(test)]
mod corpus {
    pub fn corpus() -> Vec<(String, crate::Pattern)> {
        let path =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../corpus/dev/patterns.jsonl");
        ssi_scoring::load_corpus_jsonl(&path).expect("load development corpus")
    }
}

use std::path::Path;
use std::process::{Command, ExitCode, Stdio};
use std::time::Duration;

pub use ssi_scoring::Pattern;

pub const CANDIDATE_ENTRYPOINT_MARKER: &str =
    "ssi-candidate-worker-entrypoint-v1:src/ordering::order";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let code = match args.first().map(String::as_str) {
        Some("--identity") => {
            println!("{CANDIDATE_ENTRYPOINT_MARKER}");
            0
        }
        Some("--sandbox-self-check") => sandbox_self_check(&args[1..]),
        Some("--sandbox-timeout-self-check") => timeout_self_check(&args[1..]),
        Some("--sandbox-timeout-child") => timeout_child(),
        Some("--worker-test-timeout") => test_timeout_worker(),
        _ => run_ordering(&args),
    };
    ExitCode::from(code as u8)
}

fn run_ordering(args: &[String]) -> i32 {
    let (Some(pattern_file), Some(output_file)) = (args.first(), args.get(1)) else {
        eprintln!("usage: ssi-candidate-worker <pattern_file> <output_file>");
        return 2;
    };
    if args.len() != 2 {
        eprintln!("candidate worker expected exactly two paths");
        return 2;
    }

    let pattern = match ssi_worker_protocol::read_pattern(Path::new(pattern_file)) {
        Ok(pattern) => pattern,
        Err(error) => {
            eprintln!("candidate worker failed to read pattern: {error}");
            return 3;
        }
    };
    let permutation = ordering::order(&pattern);
    match ssi_worker_protocol::write_permutation(Path::new(output_file), &permutation) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("candidate worker failed to write permutation: {error}");
            4
        }
    }
}

fn sandbox_self_check(args: &[String]) -> i32 {
    let (Some(input), Some(output)) = (args.first(), args.get(1)) else {
        return 2;
    };
    let check = || -> Result<(), String> {
        if std::process::id() != 1 {
            return Err(format!(
                "worker is PID {}, expected PID 1 in its own namespace",
                std::process::id()
            ));
        }
        // The parent env_clears and bwrap runs with --clearenv, so nothing is
        // inherited. bubblewrap itself then sets PWD to its --chdir target ("/")
        // after clearing; that single variable is injected by the sandbox, not
        // leaked from the parent, so allow it and reject anything else.
        for (name, _) in std::env::vars_os() {
            if name != "PWD" {
                return Err(format!("worker inherited environment variable {name:?}"));
            }
        }
        for hidden in ["/workspace", "/home", "/root", "/tmp"] {
            if Path::new(hidden).exists() {
                return Err(format!("unexpected host path is visible: {hidden}"));
            }
        }
        if std::fs::write(input, b"overwrite").is_ok() {
            return Err("pattern input is writable".to_string());
        }
        for fd in 0..=2 {
            let target = std::fs::read_link(format!("/proc/self/fd/{fd}"))
                .map_err(|error| format!("inspect fd {fd}: {error}"))?;
            if target != Path::new("/dev/null") {
                return Err(format!("fd {fd} is inherited ({})", target.display()));
            }
        }
        let visible_pids = std::fs::read_dir("/proc")
            .map_err(|error| format!("read private /proc: {error}"))?
            .filter_map(Result::ok)
            .filter(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .chars()
                    .all(|character| character.is_ascii_digit())
            })
            .count();
        if visible_pids != 1 {
            return Err(format!(
                "private /proc exposes {visible_pids} processes, expected one"
            ));
        }
        let routes = std::fs::read_to_string("/proc/net/route")
            .map_err(|error| format!("read network namespace routes: {error}"))?;
        if routes.lines().skip(1).any(|line| {
            line.split_whitespace()
                .nth(1)
                .is_some_and(|destination| destination == "00000000")
        }) {
            return Err("network namespace has a default route".to_string());
        }
        // Per-worker resource limits must be live, not "unlimited": file size,
        // process count, and address space. Assert the caps are present and sane
        // without pinning exact bytes, so the check proves the mechanism
        // regardless of tuning.
        let limits = std::fs::read_to_string("/proc/self/limits")
            .map_err(|error| format!("read /proc/self/limits: {error}"))?;
        check_soft_limit(&limits, "Max file size", 1024 * 1024)?;
        check_soft_limit(&limits, "Max processes", 8192)?;
        check_soft_limit(&limits, "Max address space", 8u64 * 1024 * 1024 * 1024)?;
        // The root tmpfs must be size-bounded: an unbounded `/` tmpfs charges
        // RAM without limit.
        let mounts = std::fs::read_to_string("/proc/mounts")
            .map_err(|error| format!("read /proc/mounts: {error}"))?;
        let root = mounts
            .lines()
            .find(|line| line.split_whitespace().nth(1) == Some("/"))
            .ok_or_else(|| "no root mount in /proc/mounts".to_string())?;
        let mut fields = root.split_whitespace();
        let fs_type = fields.nth(2).unwrap_or("");
        let options = fields.next().unwrap_or("");
        if fs_type != "tmpfs" || !options.split(',').any(|opt| opt.starts_with("size=")) {
            return Err(format!(
                "root filesystem is not a size-capped tmpfs: {root}"
            ));
        }
        // `--dev` mounts a second RAM-charged tmpfs that bubblewrap's `--size`
        // cannot cap, so it is held read-only instead. Assert both
        // the mount flag and the behaviour: a writable /dev is an unbounded
        // memory sink that per-file RLIMIT_FSIZE does not bound, because many
        // small files each stay under the per-file cap.
        let dev = mounts
            .lines()
            .find(|line| line.split_whitespace().nth(1) == Some("/dev"))
            .ok_or_else(|| "no /dev mount in /proc/mounts".to_string())?;
        if !dev
            .split_whitespace()
            .nth(3)
            .unwrap_or("")
            .split(',')
            .any(|opt| opt == "ro")
        {
            return Err(format!("/dev is not mounted read-only: {dev}"));
        }
        let probe = "/dev/ssi-sandbox-write-probe";
        if std::fs::write(probe, b"x").is_ok() {
            let _ = std::fs::remove_file(probe);
            return Err("/dev accepted a new file; its tmpfs is an uncapped RAM sink".to_string());
        }
        std::fs::write(output, b"ok\n").map_err(|error| format!("write permitted output: {error}"))
    };

    match check() {
        Ok(()) => 0,
        Err(error) => {
            let _ = std::fs::write(output, format!("error: {error}\n"));
            1
        }
    }
}

/// Assert a `/proc/self/limits` row has a finite soft limit at or below
/// `max_allowed`. The resource name may contain spaces, so match it as a line
/// prefix and read the soft limit as the first token that follows.
fn check_soft_limit(limits: &str, name: &str, max_allowed: u64) -> Result<(), String> {
    let line = limits
        .lines()
        .find(|line| line.starts_with(name))
        .ok_or_else(|| format!("/proc/self/limits missing {name:?}"))?;
    let soft = line[name.len()..]
        .split_whitespace()
        .next()
        .ok_or_else(|| format!("{name:?} row has no soft limit"))?;
    if soft == "unlimited" {
        return Err(format!("{name} is unlimited; resource cap not applied"));
    }
    let value: u64 = soft
        .parse()
        .map_err(|_| format!("{name} soft limit is not a number: {soft:?}"))?;
    if value > max_allowed {
        return Err(format!(
            "{name} soft limit {value} exceeds expected cap {max_allowed}"
        ));
    }
    Ok(())
}

fn timeout_self_check(args: &[String]) -> i32 {
    let Some(output) = args.get(1) else {
        return 2;
    };
    if std::fs::write(output, b"candidate started\n").is_err() {
        return 3;
    }
    let child = Command::new("/candidate-worker")
        .arg("--sandbox-timeout-child")
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn();
    if child.is_err() {
        let _ = std::fs::write(output, b"candidate child spawn failed\n");
        return 4;
    }
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

fn timeout_child() -> i32 {
    loop {
        std::thread::sleep(Duration::from_secs(60));
    }
}

#[cfg(test)]
mod limit_tests {
    use super::check_soft_limit;

    const SAMPLE: &str =
        "Limit                     Soft Limit           Hard Limit           Units\n\
        Max file size             65536                65536                bytes\n\
        Max processes             4096                 4096                 processes\n\
        Max address space         unlimited            unlimited            bytes\n";

    #[test]
    fn finite_limit_within_bound_passes() {
        assert!(check_soft_limit(SAMPLE, "Max file size", 1024 * 1024).is_ok());
    }

    #[test]
    fn limit_over_bound_fails() {
        assert!(check_soft_limit(SAMPLE, "Max file size", 1024).is_err());
    }

    #[test]
    fn unlimited_soft_limit_fails() {
        let err = check_soft_limit(SAMPLE, "Max address space", u64::MAX).unwrap_err();
        assert!(err.contains("unlimited"), "{err}");
    }

    #[test]
    fn missing_row_fails() {
        assert!(check_soft_limit(SAMPLE, "Max stack size", u64::MAX).is_err());
    }
}

/// Harness-owned time-cap regression mode (parent `--test-time-cap` runs):
/// sleep far past the 2 s cap without ever calling `ordering::order()`, so the
/// watchdog is exercised against a real worker while contestant code cannot
/// observe any test flag or environment variable.
fn test_timeout_worker() -> i32 {
    std::thread::sleep(Duration::from_secs(30));
    0
}
