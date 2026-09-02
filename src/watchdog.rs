//! Time-bounded child supervision for the local harness. Spawns a child
//! process, polls it, and SIGKILLs it if it exceeds the per-matrix time cap —
//! the same enforcement mechanism the grader uses, implemented `std`-only in
//! the public harness. Command-agnostic so it is
//! unit-testable with /bin/sleep. Trusted harness code.

use std::process::{Command, Stdio};
use std::thread::sleep;
use std::time::{Duration, Instant};

pub struct CapConfig {
    pub time_cap: Duration,
    pub poll: Duration,
}

impl Default for CapConfig {
    fn default() -> Self {
        CapConfig {
            time_cap: Duration::from_secs(2),
            poll: Duration::from_millis(10),
        }
    }
}

/// Outcome of one supervised child run.
#[derive(Debug, PartialEq, Eq)]
pub enum WorkerOutcome {
    /// Child exited 0.
    Ok,
    /// Child exceeded the time cap and was killed.
    Timeout,
    /// Child exited non-zero or could not be spawned/waited.
    Crashed(CrashDetail),
}

/// Why a supervised worker did not exit cleanly. `kind` is the redaction-safe
/// classification (OS/harness-determined bucket, safe to surface on the hidden
/// eval corpus); `detail` is the full human-readable reason and may embed a
/// worker-controlled exit code, so it is for the PUBLIC dev corpus only.
#[derive(Debug, PartialEq, Eq)]
pub struct CrashDetail {
    pub kind: CrashKind,
    pub detail: String,
}

/// Redaction-safe classification of a non-clean worker exit. The bucket itself
/// is OS/harness-determined; only the numeric exit code (folded into
/// `CrashDetail::detail`, never here) is worker-controlled.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CrashKind {
    /// Killed by a signal — e.g. SIGABRT from an allocation failure at the
    /// memory cap, or SIGSEGV. Signal delivery is OS-driven.
    Signal,
    /// Exited with a nonzero code — e.g. a panic in `order()` (code 101) or an
    /// explicit `std::process::exit(N)`. The code VALUE is worker-controlled,
    /// so only the fact of a nonzero exit is safe to surface, never the number.
    NonzeroExit,
    /// Harness-internal failure spawning or waiting on the worker; carries no
    /// worker-controlled data (the worker never ran).
    Harness,
}

/// Spawn `cmd` and supervise it under `cfg`. Kills the child on time breach.
///
/// On unix the worker is placed in its own process group (`process_group(0)`)
/// so a time-cap breach kills the WHOLE group, not just the direct worker pid.
/// Untrusted `order()` can spawn children; killing only the worker would
/// re-parent them to init and let them outlive the cap.
pub fn run_capped(cmd: &mut Command, cfg: &CapConfig) -> WorkerOutcome {
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        // pgid = 0 → new group whose id equals the worker pid.
        cmd.process_group(0);
    }
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            return WorkerOutcome::Crashed(CrashDetail {
                kind: CrashKind::Harness,
                detail: format!("could not spawn worker: {e}"),
            })
        }
    };
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                // Reap detached children on EVERY return, not just timeout: a
                // fast, successful order() must not leave a background process
                // behind (F1/F3). Best-effort: a setsid'd daemon escapes the
                // process group — which is why the scored step additionally
                // runs in a PID namespace (benchmark.yml), where escape is
                // impossible. This layer catches the common case even if the
                // namespace is ever lost, for one syscall.
                kill_group(&mut child);
                return if status.success() {
                    WorkerOutcome::Ok
                } else {
                    WorkerOutcome::Crashed(describe_crash(&status))
                };
            }
            Ok(None) => {}
            Err(e) => {
                return WorkerOutcome::Crashed(CrashDetail {
                    kind: CrashKind::Harness,
                    detail: format!("wait failed: {e}"),
                })
            }
        }
        if start.elapsed() > cfg.time_cap {
            kill_group(&mut child);
            let _ = child.wait();
            return WorkerOutcome::Timeout;
        }
        sleep(cfg.poll);
    }
}

/// SIGKILL the worker and every process it spawned. On unix the worker leads
/// its own process group (see `run_capped`), so `kill -KILL -- -<pgid>` reaps
/// the whole subtree; `pgid` equals the worker pid set with `process_group(0)`.
/// If no trusted kill utility is available or group signaling fails, falls
/// back to the direct child; non-unix platforms always use that fallback.
#[cfg(unix)]
fn kill_group(child: &mut std::process::Child) {
    let pid = child.id();
    let killed = trusted_kill_path()
        .map(|kill| {
            // `--` is load-bearing on GNU kill: without it, the negative PGID
            // is parsed as an option and can end up signalling pid -1.
            kill_group_command(kill, pid)
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
        })
        .unwrap_or(false);
    if !killed {
        // No trusted kill utility, or the group kill failed (e.g. worker died
        // before setpgid took effect): signal the direct worker pid via std.
        let _ = child.kill();
    }
}

#[cfg(unix)]
fn trusted_kill_path() -> Option<&'static std::path::Path> {
    ["/bin/kill", "/usr/bin/kill"]
        .into_iter()
        .map(std::path::Path::new)
        .find(|path| path.is_file())
}

#[cfg(unix)]
fn kill_group_command(kill: &std::path::Path, pgid: u32) -> Command {
    let mut command = Command::new(kill);
    command.arg("-KILL").arg("--").arg(format!("-{pgid}"));
    // The group kill is fired on EVERY return, including a fast, successful
    // worker exit whose process group is already gone. In that (normal) case
    // `kill` writes "No such process" (ESRCH) to stderr. We already discard the
    // command's result (see `kill_group`) and fall back to `child.kill()`, so
    // that line is pure log noise on every graded matrix — silence both streams.
    command.stdout(Stdio::null()).stderr(Stdio::null());
    command
}

#[cfg(not(unix))]
fn kill_group(child: &mut std::process::Child) {
    let _ = child.kill();
}

#[cfg(unix)]
fn describe_crash(status: &std::process::ExitStatus) -> CrashDetail {
    use std::os::unix::process::ExitStatusExt;
    if let Some(sig) = status.signal() {
        CrashDetail {
            kind: CrashKind::Signal,
            detail: format!("worker killed by signal {sig}"),
        }
    } else if let Some(code) = status.code() {
        CrashDetail {
            kind: CrashKind::NonzeroExit,
            detail: format!("worker exited with code {code}"),
        }
    } else {
        CrashDetail {
            kind: CrashKind::NonzeroExit,
            detail: "worker exited abnormally".to_string(),
        }
    }
}

#[cfg(not(unix))]
fn describe_crash(status: &std::process::ExitStatus) -> CrashDetail {
    CrashDetail {
        kind: CrashKind::NonzeroExit,
        detail: format!("worker exited: {status}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fast_command_is_ok() {
        let mut cmd = Command::new("true");
        let cfg = CapConfig {
            time_cap: Duration::from_secs(2),
            poll: Duration::from_millis(5),
        };
        assert_eq!(run_capped(&mut cmd, &cfg), WorkerOutcome::Ok);
    }

    #[test]
    fn slow_command_times_out_promptly() {
        // On Linux this catches ambiguous GNU kill parsing: `-<pgid>` without
        // a preceding `--` is treated as a signal option and sleep runs 10s.
        let mut cmd = Command::new("sleep");
        cmd.arg("10");
        let cfg = CapConfig {
            time_cap: Duration::from_millis(200),
            poll: Duration::from_millis(5),
        };
        let start = Instant::now();
        let outcome = run_capped(&mut cmd, &cfg);
        assert_eq!(outcome, WorkerOutcome::Timeout);
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "watchdog did not kill promptly: {:?}",
            start.elapsed()
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn linux_group_kill_uses_trusted_path_and_option_terminator() {
        let kill = trusted_kill_path().expect("Linux runner has /bin/kill or /usr/bin/kill");
        assert!(
            kill.is_absolute(),
            "kill path must not depend on PATH: {kill:?}"
        );

        let command = kill_group_command(kill, 12345);
        assert_eq!(command.get_program(), kill.as_os_str());
        let args: Vec<_> = command
            .get_args()
            .map(|arg| arg.to_str().expect("kill arguments are ASCII"))
            .collect();
        assert_eq!(
            args,
            ["-KILL", "--", "-12345"],
            "`--` must terminate options before the negative process-group ID"
        );
    }

    #[cfg(unix)]
    #[test]
    fn timeout_kills_spawned_grandchildren() {
        // The worker (sh) backgrounds a long sleep, records its pid, then blocks
        // past the cap. Before the fix the watchdog SIGKILLs only sh; the
        // backgrounded sleep is re-parented to init and outlives the cap. The fix
        // runs the worker in its own process group and kills the whole group.
        let pidfile = std::env::temp_dir().join(format!("ssi-wd-{}.pid", std::process::id()));
        let _ = std::fs::remove_file(&pidfile);
        let mut cmd = Command::new("sh");
        cmd.arg("-c").arg(format!(
            "sleep 30 & echo $! > {}; sleep 30",
            pidfile.display()
        ));
        let cfg = CapConfig {
            time_cap: Duration::from_millis(200),
            poll: Duration::from_millis(5),
        };
        assert_eq!(run_capped(&mut cmd, &cfg), WorkerOutcome::Timeout);

        // Read the grandchild pid the worker recorded before it was killed.
        let mut pid_str = String::new();
        for _ in 0..50 {
            if let Ok(s) = std::fs::read_to_string(&pidfile) {
                if !s.trim().is_empty() {
                    pid_str = s;
                    break;
                }
            }
            sleep(Duration::from_millis(10));
        }
        let _ = std::fs::remove_file(&pidfile);
        let gpid: i32 = pid_str.trim().parse().expect("grandchild pid recorded");

        // The grandchild must be gone shortly after the cap kill. `kill -0`
        // probes existence without signaling.
        let mut alive = true;
        for _ in 0..100 {
            let ok = Command::new("kill")
                .arg("-0")
                .arg(gpid.to_string())
                .status()
                .map(|s| s.success())
                .unwrap_or(false);
            if !ok {
                alive = false;
                break;
            }
            sleep(Duration::from_millis(10));
        }
        if alive {
            // Reap the orphan so a red test doesn't leave it lingering.
            let _ = Command::new("kill")
                .arg("-KILL")
                .arg(gpid.to_string())
                .status();
        }
        assert!(!alive, "grandchild {gpid} survived the time cap (orphaned)");
    }

    #[test]
    fn nonzero_exit_is_crashed() {
        let mut cmd = Command::new("false");
        let cfg = CapConfig {
            time_cap: Duration::from_secs(2),
            poll: Duration::from_millis(5),
        };
        assert!(matches!(
            run_capped(&mut cmd, &cfg),
            WorkerOutcome::Crashed(CrashDetail {
                kind: CrashKind::NonzeroExit,
                ..
            })
        ));
    }
}
