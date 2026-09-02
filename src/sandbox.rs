//! Per-worker Linux sandboxing for the production grader.
//!
//! The trusted harness stays outside the sandbox so it can load and score the
//! hidden corpus. In grader mode, every individual `order()` invocation gets a
//! fresh bubblewrap PID and network namespace plus a minimal filesystem. This
//! is intentionally per worker: putting the parent and all workers in one PID
//! namespace would still expose the parent's `/proc` entries to contestant
//! code.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

pub const MODE_ENV: &str = "SSI_GRADER_SANDBOX";

/// Local-only escape hatch. Running the pushed-back winning submission's
/// `order()` unsandboxed is remote code execution on your own machine, so local
/// runs sandbox the worker by default and FAIL CLOSED when no native sandbox is
/// available. Setting this to
/// `1` opts back into direct host execution (with a loud warning) for users who
/// understand the risk or cannot install a sandbox. It is honored ONLY in local
/// mode; `--grader` ignores it entirely.
pub const ALLOW_UNSANDBOXED_ENV: &str = "SSI_ALLOW_UNSANDBOXED_WORKER";

/// Size cap for the worker's root tmpfs. bubblewrap mounts `/` as tmpfs, whose
/// pages are charged to RAM (page cache), not runner disk. Without a `--size`
/// bound a submission can memory-exhaust the whole grading job — trusted parent
/// included — by writing to `/`. The worker legitimately writes
/// nothing to `/`; its only writable path is the bound output file. 64 MiB
/// leaves slack for transient runtime files while making a fill attack fail
/// against this cap instead of the machine.
const ROOT_TMPFS_BYTES: u64 = 64 * 1024 * 1024;

/// Address-space cap for the worker (`RLIMIT_AS`). The competition budgets a
/// 2–4 GB per-matrix memory ceiling, which doubles as the anti-lookup-table cap;
/// the factorization that needs it runs in the trusted parent, so an
/// ordering-only worker never approaches this. Enforcing that ceiling here also
/// stops an in-process allocation OOM that the tmpfs `--size` cap does not
/// cover.
const ADDRESS_SPACE_BYTES: u64 = 4 * 1024 * 1024 * 1024;

/// Process cap for the worker (`RLIMIT_NPROC`). A legitimate ordering forks
/// nothing (rayon uses threads within one process); a fork bomb can otherwise
/// spawn tens of thousands. `RLIMIT_NPROC` is per-real-UID — it counts
/// the trusted parent's own tasks too — so it cannot be set as tight as a
/// per-cgroup `pids.max` (which would need cgroup delegation the unprivileged
/// runner user does not get). 4096 bounds a runaway well below the observed
/// ~24k while staying clear of any legitimate steady-state task count.
const MAX_PROCESSES: u64 = 4096;

/// Headroom added above the exact permutation size when capping the writable
/// output file, so the exact valid write always succeeds.
const OUTPUT_SLACK_BYTES: u64 = 4096;

/// Output-file cap for the sandbox self-checks, which write a short diagnostic
/// string rather than a permutation.
const SELF_CHECK_OUTPUT_BYTES: u64 = 64 * 1024;

/// Ceiling on the computed output cap. `8 + 8*n` is saturating, and a saturated
/// `u64::MAX` is exactly `RLIM_INFINITY` — the one value that would turn the cap
/// OFF rather than making it large. Clamping keeps `--fsize` finite for every
/// `n`. 1 GiB is ~134M columns, far above the corpus, so it never binds a
/// legitimate write.
const MAX_OUTPUT_CEILING_BYTES: u64 = 1024 * 1024 * 1024;

/// Per-worker resource limits applied in grader mode. `for_order` sizes the
/// writable output to the exact permutation this matrix produces so a runaway
/// write hits `SIGXFSZ` immediately; the other caps are fixed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WorkerLimits {
    root_tmpfs_bytes: u64,
    max_output_bytes: u64,
    address_space_bytes: u64,
    max_processes: u64,
}

impl WorkerLimits {
    /// Limits for a real `order()` invocation on a pattern of dimension `n`. A
    /// valid permutation output is exactly `8 + 8*n` bytes (see
    /// `ssi_worker_protocol::write_permutation`).
    pub fn for_order(n: usize) -> Self {
        let max_output_bytes = 8u64
            .saturating_add((n as u64).saturating_mul(8))
            .saturating_add(OUTPUT_SLACK_BYTES)
            .min(MAX_OUTPUT_CEILING_BYTES);
        Self {
            root_tmpfs_bytes: ROOT_TMPFS_BYTES,
            max_output_bytes,
            address_space_bytes: ADDRESS_SPACE_BYTES,
            max_processes: MAX_PROCESSES,
        }
    }

    /// Limits for the sandbox self-checks (diagnostic output, not a permutation).
    pub fn for_self_check() -> Self {
        Self {
            root_tmpfs_bytes: ROOT_TMPFS_BYTES,
            max_output_bytes: SELF_CHECK_OUTPUT_BYTES,
            address_space_bytes: ADDRESS_SPACE_BYTES,
            max_processes: MAX_PROCESSES,
        }
    }

    #[cfg(test)]
    fn max_output_bytes(&self) -> u64 {
        self.max_output_bytes
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkerInvocation {
    Order,
    SandboxSelfCheck,
    SandboxTimeoutSelfCheck,
    /// Harness-owned time-cap regression mode: the candidate binary sleeps past
    /// the cap without ever calling `order()`, so the watchdog is exercised
    /// against a real child while contestant code never sees a test seam.
    TestTimeout,
}

impl WorkerInvocation {
    fn flag(self) -> Option<&'static str> {
        match self {
            Self::Order => None,
            Self::SandboxSelfCheck => Some("--sandbox-self-check"),
            Self::SandboxTimeoutSelfCheck => Some("--sandbox-timeout-self-check"),
            Self::TestTimeout => Some("--worker-test-timeout"),
        }
    }
}

#[derive(Debug)]
pub enum WorkerSandbox {
    /// Direct host execution — no isolation. Reachable locally ONLY via the
    /// explicit `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out; never the default.
    Disabled,
    /// Linux bubblewrap: fresh PID/network/mount namespaces per worker. Used by
    /// the grader and, by default, by local runs on Linux.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    Bubblewrap {
        executable: PathBuf,
        /// `prlimit` (util-linux) wraps bubblewrap to apply per-worker rlimits
        /// (file size, address space, process count). Kept as a resolved path
        /// so the worker command never depends on `PATH`.
        prlimit: PathBuf,
    },
    /// macOS `sandbox-exec` (Seatbelt): the default for local runs on macOS,
    /// where bubblewrap does not exist. Denies network and all filesystem writes
    /// except the single staged output file.
    #[cfg_attr(not(target_os = "macos"), allow(dead_code))]
    Seatbelt { executable: PathBuf },
}

impl WorkerSandbox {
    /// Select the sandbox from the trusted parent's environment.
    ///
    /// In `--grader` mode, `SSI_GRADER_SANDBOX` must be exactly `"bubblewrap"`;
    /// anything else is a hard error, so the production command cannot silently
    /// degrade to an unsandboxed worker.
    ///
    /// Local runs sandbox the worker BY DEFAULT: with no explicit
    /// `SSI_GRADER_SANDBOX`, the host-native sandbox is selected — bubblewrap on
    /// Linux, `sandbox-exec` (Seatbelt) on macOS. If no native sandbox is
    /// available the selection FAILS CLOSED rather than running untrusted code
    /// directly, unless `SSI_ALLOW_UNSANDBOXED_WORKER=1` is set as an explicit,
    /// at-your-own-risk opt-out.
    pub fn from_env(grader_mode: bool) -> Result<Self, String> {
        Self::from_value(grader_mode, std::env::var_os(MODE_ENV))
    }

    fn from_value(grader_mode: bool, value: Option<std::ffi::OsString>) -> Result<Self, String> {
        // The opt-out is honored only in local mode; --grader ignores it.
        let allow_unsandboxed = !grader_mode && env_is_one(ALLOW_UNSANDBOXED_ENV);
        Self::select(grader_mode, value, allow_unsandboxed)
    }

    /// Pure selection core (no environment reads beyond the injected values), so
    /// tests are deterministic across platforms.
    fn select(
        grader_mode: bool,
        value: Option<std::ffi::OsString>,
        allow_unsandboxed: bool,
    ) -> Result<Self, String> {
        // An explicit, non-empty selection is validated the same way in both
        // modes: only "bubblewrap" is accepted, and it is Linux-only.
        if let Some(value) = value.filter(|v| !v.is_empty()) {
            if value != "bubblewrap" {
                return Err(format!(
                    "{MODE_ENV} must be unset or exactly \"bubblewrap\""
                ));
            }
            return Self::bubblewrap_explicit();
        }

        // No explicit selection.
        if grader_mode {
            return Err(format!(
                "{MODE_ENV} must be exactly \"bubblewrap\" in --grader mode"
            ));
        }
        if allow_unsandboxed {
            return Ok(Self::Disabled);
        }
        Self::native_local()
    }

    /// `SSI_GRADER_SANDBOX=bubblewrap` was requested explicitly.
    fn bubblewrap_explicit() -> Result<Self, String> {
        #[cfg(not(target_os = "linux"))]
        {
            Err(format!(
                "{MODE_ENV}=bubblewrap is supported only on Linux; leave it unset for local development"
            ))
        }
        #[cfg(target_os = "linux")]
        {
            if effective_uid()? == 0 {
                return Err(
                    "grader parent is running as root; refusing to launch untrusted code"
                        .to_string(),
                );
            }
            let executable = find_tool(&["/usr/bin/bwrap", "/bin/bwrap"]).ok_or_else(|| {
                "bubblewrap requested but /usr/bin/bwrap and /bin/bwrap are absent".to_string()
            })?;
            let prlimit = find_tool(&["/usr/bin/prlimit", "/bin/prlimit"]).ok_or_else(|| {
                "per-worker resource limits require prlimit (util-linux) but \
                 /usr/bin/prlimit and /bin/prlimit are absent"
                    .to_string()
            })?;
            Ok(Self::Bubblewrap {
                executable,
                prlimit,
            })
        }
    }

    /// Default local selection: pick the host-native sandbox, fail closed if
    /// none is available.
    fn native_local() -> Result<Self, String> {
        #[cfg(target_os = "linux")]
        {
            if effective_uid()? == 0 {
                return Err(
                    "refusing to run the untrusted candidate worker as root; run local benchmarks \
                     as a normal user"
                        .to_string(),
                );
            }
            // Local Linux runs get the same per-worker resource caps as the
            // grader: bubblewrap for isolation, prlimit for the rlimits. Both are
            // required — fail closed if either is absent.
            let Some(executable) = find_tool(&["/usr/bin/bwrap", "/bin/bwrap"]) else {
                return Err(fail_closed_message(
                    "bubblewrap (bwrap)",
                    "install it, e.g. `apt-get install bubblewrap`",
                ));
            };
            let Some(prlimit) = find_tool(&["/usr/bin/prlimit", "/bin/prlimit"]) else {
                return Err(fail_closed_message(
                    "prlimit (util-linux)",
                    "install it, e.g. `apt-get install util-linux`",
                ));
            };
            return Ok(Self::Bubblewrap {
                executable,
                prlimit,
            });
        }
        #[cfg(target_os = "macos")]
        {
            return match find_tool(&["/usr/bin/sandbox-exec"]) {
                Some(executable) => Ok(Self::Seatbelt { executable }),
                None => Err(fail_closed_message(
                    "sandbox-exec",
                    "it ships with macOS; /usr/bin/sandbox-exec is expected to exist",
                )),
            };
        }
        #[cfg(not(any(target_os = "linux", target_os = "macos")))]
        {
            Err(fail_closed_message(
                "a built-in worker sandbox",
                "this platform has none",
            ))
        }
    }

    pub fn is_enabled(&self) -> bool {
        matches!(self, Self::Bubblewrap { .. } | Self::Seatbelt { .. })
    }

    /// Construct one worker command. The caller must create `output` first so
    /// bubblewrap can bind that single writable file into an otherwise
    /// read-only/minimal filesystem.
    pub fn worker_command(
        &self,
        candidate_exe: &Path,
        invocation: WorkerInvocation,
        input: &Path,
        output: &Path,
        local_cwd: &Path,
        limits: &WorkerLimits,
    ) -> Result<Command, String> {
        match self {
            Self::Disabled => {
                // Local development runs contestant code directly; it is
                // arbitrary-code-by-design and gets no sandbox, so the grader
                // resource limits do not apply here.
                let _ = limits;
                let mut cmd = Command::new(candidate_exe);
                if let Some(flag) = invocation.flag() {
                    cmd.arg(flag);
                }
                // The worker runs contestant code; hand it NOTHING beyond its
                // pattern/output arguments. No environment entry — test control
                // included — ever reaches order() (F2).
                cmd.arg(input)
                    .arg(output)
                    .env_clear()
                    .current_dir(local_cwd);
                null_stdio(&mut cmd);
                Ok(cmd)
            }
            Self::Bubblewrap {
                executable,
                prlimit,
            } => {
                let candidate_exe = canonical_file(candidate_exe, "candidate executable")?;
                let input = canonical_file(input, "worker input")?;
                let output = canonical_file(output, "worker output")?;

                // Wrap bubblewrap in prlimit so per-worker rlimits are set on
                // the process that execs bwrap and inherited across it into the
                // worker. bubblewrap provides namespace/mount isolation but no
                // resource limits; these caps close the "missing per-worker
                // resource limits" family — runaway writes, fork bombs, and the
                // in-process memory vector — that isolation alone does not cover:
                //   --fsize caps the single writable output file,
                //   --nproc bounds a fork bomb,
                //   --as    enforces the documented per-matrix memory cap,
                //   --core  suppresses core dumps that would fill the scratch.
                let mut cmd = Command::new(prlimit);
                cmd.env_clear()
                    .arg(format!("--nproc={}", limits.max_processes))
                    .arg(format!("--fsize={}", limits.max_output_bytes))
                    .arg(format!("--as={}", limits.address_space_bytes))
                    .arg("--core=0")
                    .arg("--")
                    .arg(executable);

                cmd.args([
                    "--unshare-pid",
                    "--unshare-net",
                    "--unshare-ipc",
                    "--unshare-uts",
                    "--new-session",
                    "--die-with-parent",
                    "--as-pid-1",
                    "--cap-drop",
                    "ALL",
                    "--clearenv",
                ]);
                // Size-cap the root tmpfs so a submission cannot exhaust RAM by
                // writing to `/`. `--size` applies to the tmpfs mounted
                // immediately after it.
                cmd.arg("--size")
                    .arg(limits.root_tmpfs_bytes.to_string())
                    .args(["--tmpfs", "/", "--proc", "/proc", "--dev", "/dev"]);
                // `--dev` mounts a SECOND tmpfs, which is also RAM-charged. It
                // cannot be size-capped the same way: bubblewrap consumes
                // `--size` only in its `--tmpfs` handler and dies with "--size
                // must be followed by --tmpfs" otherwise (bubblewrap.c,
                // `next_size_arg`). Left alone it is an unbounded RAM sink that
                // the per-file RLIMIT_FSIZE does not cover, because many small
                // files each stay under the per-file cap while the total does
                // not. Remount it read-only instead — zero writable bytes is a
                // strictly tighter bound than any size cap. The remount is
                // non-recursive and applies to the /dev tmpfs itself, so the
                // device nodes bubblewrap bind-mounted under it (/dev/null,
                // /dev/urandom, ...) stay readable and writable as devices.
                cmd.args(["--remount-ro", "/dev"]);

                // The Rust binary is normally dynamically linked to libc and
                // libgcc. Expose only runtime-library trees, read-only; do not
                // expose /usr/bin, the repository, HOME, or runner temp files.
                for runtime_dir in ["/lib", "/lib64", "/usr/lib"] {
                    if Path::new(runtime_dir).exists() {
                        cmd.arg("--ro-bind").arg(runtime_dir).arg(runtime_dir);
                    }
                }

                cmd.arg("--ro-bind")
                    .arg(&candidate_exe)
                    .arg("/candidate-worker")
                    .arg("--ro-bind")
                    .arg(&input)
                    .arg("/input/pattern.bin")
                    .arg("--bind")
                    .arg(&output)
                    .arg("/output/permutation.bin")
                    .args(["--chdir", "/", "--", "/candidate-worker"]);
                if let Some(flag) = invocation.flag() {
                    cmd.arg(flag);
                }
                cmd.args(["/input/pattern.bin", "/output/permutation.bin"]);
                null_stdio(&mut cmd);
                Ok(cmd)
            }
            Self::Seatbelt { executable } => {
                // macOS/Darwin does not enforce RLIMIT_AS (setrlimit is refused
                // and large allocations are not bounded), and there is no
                // unprivileged memory-cgroup equivalent, so the per-worker
                // resource caps cannot be applied here. macOS local runs rely on
                // Seatbelt's network-deny + write-confinement and the per-matrix
                // time cap; full resource isolation is deferred to the managed
                // grading sandbox. Only the Linux/grader bubblewrap path enforces
                // the prlimit caps.
                let _ = limits;
                // Resolve symlinks: Seatbelt matches on the canonical path, and
                // macOS temp/output paths route through /var -> /private/var, so
                // an unresolved literal in the write-allow rule would never match.
                let candidate_exe = canonical_file(candidate_exe, "candidate executable")?;
                let input = canonical_file(input, "worker input")?;
                let output = canonical_file(output, "worker output")?;
                let profile = seatbelt_profile(&candidate_exe, &input, &output);

                let mut cmd = Command::new(executable);
                cmd.arg("-p").arg(&profile).arg(&candidate_exe);
                if let Some(flag) = invocation.flag() {
                    cmd.arg(flag);
                }
                cmd.arg(&input)
                    .arg(&output)
                    // Hand contestant code NOTHING beyond its two path arguments,
                    // matching the Disabled/Bubblewrap arms.
                    .env_clear()
                    .current_dir(local_cwd);
                null_stdio(&mut cmd);
                Ok(cmd)
            }
        }
    }
}

/// Home directory subtrees whose reads are denied to the worker.
/// Every credential class the POC exfiltrated — SSH keys, cloud credentials,
/// GnuPG rings, login/keychain data, browser cookies — lives under a user home
/// (`/Users/<u>` on macOS, `/home/<u>` on Linux layouts) or root's home
/// (`/var/root`, canonically `/private/var/root`). Denying these removes the
/// material there is any incentive to encode into the scored permutation.
const SEATBELT_DENIED_READ_SUBTREES: &[&str] =
    &["/Users", "/home", "/var/root", "/private/var/root"];

/// The read-deny subtree list for one worker: the static home prefixes above,
/// plus the *canonicalized* `$HOME` when it falls outside them.
///
/// Seatbelt matches `subpath` rules on the realpath (symlinks resolved), so a
/// home relocated onto another volume by symlink (e.g. `/Users/bob` ->
/// `/Volumes/data/bob`, common for network or oversized homes) would slip past
/// the bare `/Users` prefix and reopen the exfiltration channel. Resolving
/// `$HOME` and denying its real location closes that. (Only the home *root* can
/// be resolved this way; a home whose individual subdirectory is symlinked
/// off-tree cannot be enumerated here.)
///
/// Pure over `canonical_home` so it can be unit-tested without a symlinked home;
/// the caller resolves `$HOME` to its realpath and passes the result.
fn seatbelt_denied_read_subtrees(canonical_home: Option<&str>) -> Vec<String> {
    let mut subtrees: Vec<String> = SEATBELT_DENIED_READ_SUBTREES
        .iter()
        .map(|s| (*s).to_string())
        .collect();
    if let Some(home) = canonical_home {
        let covered = subtrees
            .iter()
            .any(|s| home == s || home.starts_with(&format!("{s}/")));
        if !covered {
            subtrees.push(home.to_string());
        }
    }
    subtrees
}

/// Build the Seatbelt profile for one worker: deny everything, re-allow only
/// what a dynamically linked Rust binary needs to run (exec/fork, sysctl, mach
/// lookups) plus broad reads, then CARVE OUT the home directories, deny all
/// network, and permit writes to exactly the one staged output file plus
/// /dev/null.
///
/// The home-directory read carve-out is what closes the exfiltration channel. A
/// blanket `(allow file-read*)` let a winning submission's `order()` read any
/// host file (`~/.ssh`, `~/.aws`, the keychain, browser data) and encode what it
/// found into its returned permutation — an attacker-controlled channel that
/// modulates the committed `score.json`/`results.tsv`, turning every
/// contestant's local run into a fleet-scale exfiltration node. Network-deny and
/// write-confinement alone do NOT close it, because the score itself is a
/// low-bandwidth covert channel to a public artifact. Denying reads of the home
/// directories removes the secrets there is any incentive to encode.
///
/// Why a deny-list here rather than the Linux side's tight allow-list: on macOS,
/// dyld's shared-cache discovery reads across a set of version-dependent paths
/// (the cryptex-split cache under `/System/Volumes/Preboot/Cryptexes/OS/...`,
/// dyld closures, and fallbacks) that a minimal allow-list cannot pin without
/// breaking the worker before `order()` runs. This was verified empirically on
/// macOS 26.x / arm64: a deny-default profile allowing only `/usr/lib`,
/// `/System`, and `/private/var/db/dyld` aborts in `dyld4::CacheFinder`, and even
/// allowing every standard system tree while omitting `/Users` still aborts;
/// keeping broad read while denying the home trees runs cleanly. The
/// `seatbelt_worker_produces_valid_permutation` e2e test pins the positive
/// direction (the worker runs) and `seatbelt_denies_home_reads_but_runs` pins the
/// negative (a `$HOME` read is blocked), so a future macOS release that regresses
/// either direction fails loudly.
///
/// Seatbelt matches the LAST rule that applies, so the exact candidate binary and
/// input are re-allowed by literal AFTER the home-directory denies: the worker
/// still loads and reads its pattern even when the clone (hence the binary and
/// staged input) lives under `$HOME` — the case a bare home-dir deny would
/// otherwise break. Output writes use `file-write*`, which the read denies do not
/// touch.
///
/// All three paths MUST already be canonicalized by the caller (Seatbelt matches
/// on canonical paths). `$HOME` is resolved here so that a home symlinked onto
/// another volume is still denied — see `seatbelt_denied_read_subtrees`.
fn seatbelt_profile(candidate_exe: &Path, input: &Path, output: &Path) -> String {
    let candidate_exe = seatbelt_quote(&candidate_exe.to_string_lossy());
    let input = seatbelt_quote(&input.to_string_lossy());
    let output = seatbelt_quote(&output.to_string_lossy());
    let canonical_home = std::env::var_os("HOME")
        .and_then(|home| std::fs::canonicalize(home).ok())
        .map(|home| home.to_string_lossy().into_owned());
    let deny_home_reads: String = seatbelt_denied_read_subtrees(canonical_home.as_deref())
        .iter()
        .map(|subtree| {
            let subtree = seatbelt_quote(subtree);
            format!("(deny file-read* (subpath \"{subtree}\"))")
        })
        .collect();
    format!(
        "(version 1)\
         (deny default)\
         (allow process-fork)\
         (allow process-exec)\
         (allow sysctl-read)\
         (allow mach-lookup)\
         (allow file-read*)\
         {deny_home_reads}\
         (allow file-read* (literal \"{candidate_exe}\"))\
         (allow file-read* (literal \"{input}\"))\
         (deny network*)\
         (allow file-write-data (literal \"/dev/null\"))\
         (allow file-write* (literal \"{output}\"))"
    )
}

/// Escape a path for embedding inside a Seatbelt string literal.
fn seatbelt_quote(path: &str) -> String {
    path.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg_attr(not(any(target_os = "linux", target_os = "macos")), allow(dead_code))]
fn find_tool(candidates: &[&str]) -> Option<PathBuf> {
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

fn env_is_one(name: &str) -> bool {
    std::env::var_os(name).is_some_and(|value| value == "1")
}

fn fail_closed_message(tool: &str, hint: &str) -> String {
    format!(
        "refusing to run the untrusted candidate worker without a sandbox: {tool} was not found \
         ({hint}). Local runs sandbox order() by default so a malicious pushed-back submission \
         cannot reach your network or files. Install the sandbox, or set \
         {ALLOW_UNSANDBOXED_ENV}=1 to run the worker directly on your host at your own risk."
    )
}

fn canonical_file(path: &Path, description: &str) -> Result<PathBuf, String> {
    let canonical = fs::canonicalize(path)
        .map_err(|e| format!("cannot resolve {description} {}: {e}", path.display()))?;
    if !canonical.is_file() {
        return Err(format!(
            "{description} is not a regular file: {}",
            canonical.display()
        ));
    }
    Ok(canonical)
}

fn null_stdio(cmd: &mut Command) {
    cmd.stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
}

#[cfg(target_os = "linux")]
fn effective_uid() -> Result<u32, String> {
    let status = fs::read_to_string("/proc/self/status")
        .map_err(|e| format!("cannot determine grader uid from /proc/self/status: {e}"))?;
    parse_effective_uid(&status)
        .ok_or_else(|| "cannot parse effective uid from /proc/self/status".to_string())
}

#[cfg(target_os = "linux")]
fn parse_effective_uid(status: &str) -> Option<u32> {
    let uid_line = status.lines().find(|line| line.starts_with("Uid:"))?;
    uid_line.split_whitespace().nth(2)?.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::OsStr;

    #[test]
    fn disabled_command_clears_environment_and_uses_host_paths() {
        let sandbox = WorkerSandbox::Disabled;
        let cmd = sandbox
            .worker_command(
                Path::new("/candidate"),
                WorkerInvocation::Order,
                Path::new("/pattern"),
                Path::new("/permutation"),
                Path::new("/empty"),
                &WorkerLimits::for_order(4),
            )
            .unwrap();

        assert_eq!(cmd.get_program(), OsStr::new("/candidate"));
        assert_eq!(
            cmd.get_args().collect::<Vec<_>>(),
            [OsStr::new("/pattern"), OsStr::new("/permutation")]
        );
        assert!(cmd.get_envs().all(|(_, value)| value.is_none()));
        assert_eq!(cmd.get_current_dir(), Some(Path::new("/empty")));
    }

    #[test]
    fn denied_read_subtrees_cover_relocated_home() {
        // The static prefixes are always present.
        let base = seatbelt_denied_read_subtrees(None);
        for subtree in SEATBELT_DENIED_READ_SUBTREES {
            assert!(base.iter().any(|s| s == subtree));
        }

        // A home already under a static prefix adds nothing (no duplicate).
        let under_users = seatbelt_denied_read_subtrees(Some("/Users/alice"));
        assert_eq!(under_users, base);

        // Symlink bypass: a home whose realpath is on another volume
        // is NOT under any static prefix, so it must get its own deny — Seatbelt
        // resolves symlinks before matching, so `/Users` alone would miss it.
        let relocated = seatbelt_denied_read_subtrees(Some("/Volumes/data/bob"));
        assert!(relocated.iter().any(|s| s == "/Volumes/data/bob"));
    }

    #[test]
    fn grader_mode_rejects_missing_or_empty_sandbox_selection() {
        // In grader mode the opt-out is irrelevant: a missing/empty selection is
        // always a hard error.
        assert!(WorkerSandbox::select(true, None, true)
            .unwrap_err()
            .contains("must be exactly \"bubblewrap\""));
        assert!(WorkerSandbox::select(true, Some("".into()), true)
            .unwrap_err()
            .contains("must be exactly \"bubblewrap\""));
    }

    #[test]
    fn local_opt_out_yields_disabled_only_when_explicit() {
        // The explicit opt-out downgrades to direct execution...
        assert!(matches!(
            WorkerSandbox::select(false, None, true).unwrap(),
            WorkerSandbox::Disabled
        ));
        // ...but WITHOUT it, a local run never silently returns Disabled: it
        // either selects the host-native sandbox or fails closed.
        match WorkerSandbox::select(false, None, false) {
            Ok(WorkerSandbox::Disabled) => panic!("local default must not be unsandboxed"),
            Ok(_) => {} // native sandbox available on this host
            Err(message) => {
                assert!(message.contains(ALLOW_UNSANDBOXED_ENV));
                // The message must say WHY, not just refuse.
                assert!(message.contains("sandbox order() by default"));
            }
        }
    }

    #[test]
    fn rejects_unknown_explicit_selection() {
        assert!(WorkerSandbox::select(false, Some("firejail".into()), false)
            .unwrap_err()
            .contains("must be unset or exactly \"bubblewrap\""));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_local_default_selects_seatbelt() {
        assert!(matches!(
            WorkerSandbox::select(false, None, false).unwrap(),
            WorkerSandbox::Seatbelt { .. }
        ));
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn seatbelt_command_denies_network_and_allows_only_the_output() {
        let dir = std::env::temp_dir().join(format!("ssi-seatbelt-command-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let candidate = dir.join("candidate");
        let input = dir.join("input");
        let output = dir.join("output");
        for path in [&candidate, &input, &output] {
            std::fs::write(path, b"x").unwrap();
        }
        let sandbox = WorkerSandbox::Seatbelt {
            executable: PathBuf::from("/usr/bin/sandbox-exec"),
        };
        let cmd = sandbox
            .worker_command(
                &candidate,
                WorkerInvocation::Order,
                &input,
                &output,
                &dir,
                &WorkerLimits::for_order(4),
            )
            .unwrap();
        assert_eq!(cmd.get_program(), OsStr::new("/usr/bin/sandbox-exec"));

        let args: Vec<String> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        assert_eq!(args[0], "-p");
        let profile = &args[1];
        assert!(profile.contains("(deny default)"));
        assert!(profile.contains("(deny network*)"));

        // Home directories — where ~/.ssh, ~/.aws, the keychain and
        // browser data live — must be read-denied so order() has nothing worth
        // encoding into the scored permutation.
        for subtree in SEATBELT_DENIED_READ_SUBTREES {
            assert!(
                profile.contains(&format!("(deny file-read* (subpath \"{subtree}\"))")),
                "home read carve-out for {subtree} missing: {profile}"
            );
        }
        // The home denies must precede the candidate/input re-allows: Seatbelt
        // takes the LAST matching rule, so the literal re-allows below only win
        // (keeping the worker runnable from a clone under $HOME) if they come
        // after the denies.
        let first_home_deny = profile.find("(deny file-read* (subpath").unwrap();

        // The write-allow literal must be the CANONICAL output path (so the
        // /var -> /private/var symlink does not defeat the match), and the
        // candidate/input/output paths follow the profile, canonicalized.
        let canonical_output = std::fs::canonicalize(&output).unwrap();
        assert!(profile.contains(&format!(
            "(allow file-write* (literal \"{}\"))",
            canonical_output.display()
        )));
        let canonical_candidate = std::fs::canonicalize(&candidate)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        // The candidate binary and input are re-allowed by their exact canonical
        // literals AFTER the home denies, so the worker still loads and reads its
        // input even when the clone (and thus the binary/input) lives under
        // $HOME — the case a bare `(deny subpath "/Users")` would break.
        let candidate_allow = format!("(allow file-read* (literal \"{canonical_candidate}\"))");
        assert!(profile.contains(&candidate_allow));
        assert!(
            profile.find(&candidate_allow).unwrap() > first_home_deny,
            "candidate re-allow must follow the home denies to win last-match: {profile}"
        );
        let canonical_input = std::fs::canonicalize(&input)
            .unwrap()
            .to_string_lossy()
            .into_owned();
        assert!(profile.contains(&format!(
            "(allow file-read* (literal \"{canonical_input}\"))"
        )));
        assert_eq!(args[2], canonical_candidate);
        assert_eq!(args.last().unwrap(), &canonical_output.to_string_lossy());

        // Environment is cleared, exactly like the other worker arms.
        assert!(cmd.get_envs().all(|(_, value)| value.is_none()));
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn seatbelt_profile_escapes_quotes_in_output_path() {
        let profile = seatbelt_profile(
            Path::new("/tmp/cand"),
            Path::new("/tmp/in"),
            Path::new("/tmp/we\"ird/out.bin"),
        );
        assert!(profile.contains("/tmp/we\\\"ird/out.bin"));
    }

    /// End-to-end proof of the home-read carve-out: under the real Seatbelt
    /// profile, a read of a file under `$HOME` is DENIED while a read of a system
    /// file (dyld/runtime health) still SUCCEEDS. Uses `/bin/cat` as a stand-in
    /// for a malicious `order()`, so it needs no built candidate — but it writes
    /// a throwaway probe under the real home directory, so it is ignored during
    /// routine `cargo test` and run explicitly to re-verify per macOS release.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "writes a probe under $HOME; run explicitly to re-verify the home-read carve-out"]
    fn seatbelt_denies_home_reads_but_runs() {
        let home = PathBuf::from(std::env::var_os("HOME").expect("HOME must be set"));
        assert!(
            home.starts_with("/Users") || home.starts_with("/home") || home.starts_with("/var"),
            "test assumes HOME lives under a denied subtree: {}",
            home.display()
        );
        let probe_dir = home.join(format!(".ssi63-carveout-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&probe_dir);
        std::fs::create_dir_all(&probe_dir).unwrap();
        let secret = probe_dir.join("stolen_secret");
        std::fs::write(&secret, b"top secret\n").unwrap();

        // A system file that must stay readable, and a scratch output file.
        let system_file = Path::new("/System/Library/CoreServices/SystemVersion.plist");
        assert!(system_file.exists(), "expected a readable system file");
        let output = probe_dir.join("out");
        std::fs::File::create(&output).unwrap();

        // Build the exact production profile. `input` is a SYSTEM file (not the
        // probe), so the probe is never re-allowed by the literal carve-back.
        let cat = canonical_file(Path::new("/bin/cat"), "cat").unwrap();
        let profile = seatbelt_profile(
            &cat,
            &canonical_file(system_file, "system file").unwrap(),
            &canonical_file(&output, "output").unwrap(),
        );

        let run = |target: &Path| {
            std::process::Command::new("/usr/bin/sandbox-exec")
                .arg("-p")
                .arg(&profile)
                .arg("/bin/cat")
                .arg(target)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status()
                .unwrap()
        };

        // Sanity: unsandboxed, the "secret" is readable — proving the deny below
        // is what blocks it, not a missing file.
        assert!(std::process::Command::new("/bin/cat")
            .arg(&secret)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .unwrap()
            .success());

        assert!(
            !run(&secret).success(),
            "a $HOME read must be blocked under the Seatbelt profile"
        );
        assert!(
            run(system_file).success(),
            "a system read must still succeed (dyld/runtime health)"
        );

        std::fs::remove_dir_all(&probe_dir).unwrap();
    }

    /// End-to-end proof that the Seatbelt profile does not break a genuine
    /// `order()` run: the built worker, launched under `sandbox-exec`, reads a
    /// real pattern and writes a valid permutation. Ignored unless
    /// `SSI_CANDIDATE_WORKER` names the built candidate binary.
    #[cfg(target_os = "macos")]
    #[test]
    #[ignore = "requires a built candidate worker (set SSI_CANDIDATE_WORKER)"]
    fn seatbelt_worker_produces_valid_permutation() {
        let candidate = std::env::var_os("SSI_CANDIDATE_WORKER")
            .map(PathBuf::from)
            .expect("SSI_CANDIDATE_WORKER must name the built candidate");
        let sandbox = WorkerSandbox::select(false, None, false).unwrap();
        assert!(matches!(sandbox, WorkerSandbox::Seatbelt { .. }));

        let dir = std::env::temp_dir().join(format!("ssi-seatbelt-e2e-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("pattern.bin");
        let output = dir.join("perm.bin");
        let cwd = dir.join("cwd");
        std::fs::create_dir(&cwd).unwrap();

        // A small path graph 0-1-2-3-4-5.
        let n = 6;
        let edges: Vec<(usize, usize)> = (0..n - 1).map(|i| (i, i + 1)).collect();
        let pattern = ssi_scoring::Pattern::from_edges(n, &edges);
        ssi_worker_protocol::write_pattern(&input, &pattern).unwrap();
        std::fs::File::create(&output).unwrap();

        let mut cmd = sandbox
            .worker_command(
                &candidate,
                WorkerInvocation::Order,
                &input,
                &output,
                &cwd,
                &WorkerLimits::for_order(n),
            )
            .unwrap();
        let outcome = crate::watchdog::run_capped(
            &mut cmd,
            &crate::watchdog::CapConfig {
                time_cap: std::time::Duration::from_secs(2),
                poll: std::time::Duration::from_millis(10),
            },
        );
        assert_eq!(outcome, crate::watchdog::WorkerOutcome::Ok);

        let perm = ssi_worker_protocol::read_permutation(&output, n).unwrap();
        ssi_scoring::validate_permutation(&perm, n).expect("worker returned a valid bijection");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn bubblewrap_command_executes_candidate_binary_from_process_start() {
        let dir = std::env::temp_dir().join(format!("ssi-sandbox-command-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let candidate = dir.join("candidate");
        let input = dir.join("input");
        let output = dir.join("output");
        for path in [&candidate, &input, &output] {
            std::fs::write(path, b"x").unwrap();
        }
        let sandbox = WorkerSandbox::Bubblewrap {
            executable: PathBuf::from("/test/bwrap"),
            prlimit: PathBuf::from("/test/prlimit"),
        };
        let cmd = sandbox
            .worker_command(
                &candidate,
                WorkerInvocation::Order,
                &input,
                &output,
                &dir,
                &WorkerLimits::for_order(4),
            )
            .unwrap();
        let args: Vec<_> = cmd
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let candidate = std::fs::canonicalize(candidate)
            .unwrap()
            .to_string_lossy()
            .into_owned();

        // The worker is now launched through prlimit, which execs bwrap after
        // its option terminator; bwrap in turn execs the candidate.
        assert_eq!(cmd.get_program(), OsStr::new("/test/prlimit"));
        assert!(args
            .windows(2)
            .any(|window| window == ["--", "/test/bwrap"]));
        assert!(args
            .windows(3)
            .any(|window| window == ["--ro-bind", &candidate, "/candidate-worker"]));
        assert!(args
            .windows(2)
            .any(|window| window == ["--", "/candidate-worker"]));
        assert!(!args.iter().any(|arg| arg == "--worker"));
    }

    fn bubblewrap_order_command(n: usize) -> Vec<String> {
        let dir = std::env::temp_dir().join(format!("ssi-sandbox-limits-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let candidate = dir.join("candidate");
        let input = dir.join("input");
        let output = dir.join("output");
        for path in [&candidate, &input, &output] {
            std::fs::write(path, b"x").unwrap();
        }
        let sandbox = WorkerSandbox::Bubblewrap {
            executable: PathBuf::from("/test/bwrap"),
            prlimit: PathBuf::from("/test/prlimit"),
        };
        let cmd = sandbox
            .worker_command(
                &candidate,
                WorkerInvocation::Order,
                &input,
                &output,
                &dir,
                &WorkerLimits::for_order(n),
            )
            .unwrap();
        std::iter::once(cmd.get_program())
            .chain(cmd.get_args())
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect()
    }

    #[test]
    fn worker_limits_for_order_caps_output_to_permutation_size() {
        // A valid permutation of 0..n serializes to exactly 8 + 8*n bytes.
        assert_eq!(WorkerLimits::for_order(4).max_output_bytes(), 8 + 32 + 4096);
        // Huge n must neither overflow the arithmetic nor saturate to
        // u64::MAX, which prlimit would read as RLIM_INFINITY — an unbounded
        // output file, i.e. the cap silently off. It clamps to a finite ceiling.
        assert_eq!(
            WorkerLimits::for_order(usize::MAX).max_output_bytes(),
            MAX_OUTPUT_CEILING_BYTES
        );
    }

    #[test]
    fn bubblewrap_command_caps_root_tmpfs_size() {
        let argv = bubblewrap_order_command(4);
        assert!(
            argv.windows(4)
                .any(|w| w == ["--size", "67108864", "--tmpfs", "/"]),
            "root tmpfs must be size-capped before it is mounted: {argv:?}"
        );
    }

    #[test]
    fn bubblewrap_command_makes_dev_read_only() {
        let argv = bubblewrap_order_command(4);
        // `--dev /dev` mounts a SECOND tmpfs, and bubblewrap's `--size` composes
        // only with `--tmpfs` (`--size N --dev /dev` dies with "--size must be
        // followed by --tmpfs"), so that tmpfs cannot be size-capped the way `/`
        // is. Remount it read-only instead: zero writable bytes is a strictly
        // tighter bound than any size cap, and it closes the many-small-files
        // RAM fill that the per-file RLIMIT_FSIZE does not cover.
        let dev = argv
            .iter()
            .position(|arg| arg == "--dev")
            .unwrap_or_else(|| panic!("no --dev mount to harden: {argv:?}"));
        let remount = argv
            .windows(2)
            .position(|w| w == ["--remount-ro", "/dev"])
            .unwrap_or_else(|| panic!("/dev tmpfs must be remounted read-only: {argv:?}"));
        assert!(
            remount > dev,
            "--remount-ro /dev must follow the --dev mount it hardens: {argv:?}"
        );
    }

    #[test]
    fn bubblewrap_command_wraps_bwrap_in_prlimit_with_resource_caps() {
        let argv = bubblewrap_order_command(4);
        assert_eq!(
            argv[0], "/test/prlimit",
            "worker must be wrapped in prlimit"
        );
        assert!(
            argv.contains(&"--nproc=4096".to_string()),
            "fork-bomb cap missing: {argv:?}"
        );
        assert!(
            argv.contains(&"--fsize=4136".to_string()),
            "output-file size cap missing or not sized to 8+8n+slack: {argv:?}"
        );
        assert!(
            argv.contains(&"--as=4294967296".to_string()),
            "address-space cap missing: {argv:?}"
        );
        // prlimit's option terminator must precede the bwrap program.
        let term = argv.iter().position(|a| a == "--").expect("prlimit --");
        assert_eq!(
            argv[term + 1],
            "/test/bwrap",
            "bwrap must follow prlimit's `--`: {argv:?}"
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn parses_effective_uid_not_real_uid() {
        let status = "Name:\ttest\nUid:\t1000\t1001\t1002\t1003\n";
        assert_eq!(parse_effective_uid(status), Some(1001));
        assert_eq!(parse_effective_uid("Name:\ttest\n"), None);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[ignore = "requires bubblewrap user namespaces and a built candidate worker"]
    fn bubblewrap_timeout_tears_down_candidate_descendants() {
        let candidate = std::env::var_os("SSI_CANDIDATE_WORKER")
            .map(PathBuf::from)
            .expect("SSI_CANDIDATE_WORKER must name the built candidate");
        let sandbox = WorkerSandbox::from_value(true, Some("bubblewrap".into())).unwrap();
        let dir =
            std::env::temp_dir().join(format!("ssi-bwrap-timeout-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let input = dir.join("input");
        let output = dir.join("output");
        let cwd = dir.join("cwd");
        std::fs::write(&input, b"sentinel").unwrap();
        std::fs::write(&output, b"").unwrap();
        std::fs::create_dir(&cwd).unwrap();

        let mut cmd = sandbox
            .worker_command(
                &candidate,
                WorkerInvocation::SandboxTimeoutSelfCheck,
                &input,
                &output,
                &cwd,
                &WorkerLimits::for_self_check(),
            )
            .unwrap();
        let outcome = crate::watchdog::run_capped(
            &mut cmd,
            &crate::watchdog::CapConfig {
                time_cap: std::time::Duration::from_millis(200),
                poll: std::time::Duration::from_millis(5),
            },
        );
        assert_eq!(outcome, crate::watchdog::WorkerOutcome::Timeout);
        assert_eq!(
            std::fs::read_to_string(&output).unwrap(),
            "candidate started\n"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }
}
