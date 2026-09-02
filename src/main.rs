//! Harness entry point — THE CONTRACT. Do not modify.
//!
//! `cargo run --release -- --note "what I tried"` does:
//!
//!   0. run the local purity & license gate (Stage A analog) over
//!      src/ordering/ — a stdlib-only, license-clean submission passes; any
//!      foreign-code escape or extra dependency FAILs the run before scoring;
//!
//! then, per matrix in the development corpus (the real dev patterns in
//! corpus/dev/patterns.jsonl, loaded with the shared ssi-scoring JSONL reader):
//!
//!   1. run the AMD baseline (feral_amd::amd_order) and score it through the
//!      trusted scoring wrapper;
//!   2. run YOUR ordering (src/ordering/) twice — both runs must agree
//!      (determinism gate, Stage E analog) and finish under the time cap;
//!   3. validate the permutation as a bijection of 0..n (Stage C analog);
//!   4. recompute predicted flops and nnz(L) from the permutation with the
//!      trusted scoring wrapper (Stage D) — your code never reports a number;
//!
//! then, for the public development corpus, prints a per-matrix table and a
//! per-bucket breakdown; all runs compute
//!
//!     score = weighted mean over size buckets of
//!             geomean_within_bucket( flops(yours) / flops(AMD) )
//!             (lower is better)
//!
//! Matrices are bucketed by dimension n — lt_1k (n<1000), 1k_10k
//! (1000≤n<10000), gt_10k (n≥10000) — with weights 0.30 / 0.30 / 0.40. Empty
//! buckets are renormalized out (weights rescaled over populated buckets), so on
//! a corpus that only populates one bucket the score is just that bucket's
//! geomean. The tiebreak is the same weighted scheme over the fill ratio
//! nnz(L)(yours)/nnz(L)(AMD).
//!
//! ONE SCORING CODE PATH: the baseline and your ordering are both
//! scored by `ssi_scoring::score`, the same function the private grader calls,
//! and the aggregation lives in ssi_scoring::aggregate, shared with the reference-line tools.
//! The per-matrix score is a pure function of (pattern, permutation), so the number
//! printed here is IDENTICAL to the number the grader computes for the same ordering
//! on the same matrices.
//!
//! Any invalid permutation, panic, nondeterminism, cap violation, or
//! purity/license failure makes the whole run FAIL — no partial credit, no
//! silent fallback. A panic in the trusted in-process baseline/score path
//! (feral internal error or an oversized pattern) is caught and recorded as a
//! FAIL row whose note carries the reason; the scratch dir is disposed on
//! every exit path.

mod artifacts;
mod corpus;
mod failsafe;
mod purity;
mod sandbox;
mod watchdog;

use std::fmt::Write as _;
use std::fs::OpenOptions;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use ssi_scoring::{
    combine, geomean, score, size_bucket, validate_permutation, BucketAcc, BUCKETS, BUCKET_KEYS,
    BUCKET_WEIGHTS,
};

pub use ssi_scoring::Pattern;

/// Per-matrix time cap, ENFORCED: order() runs in a child process that is
/// SIGKILLed at this bound (see watchdog). 2 s is the strict end of the 2–5 s
/// band the competition budgets per matrix. The grader runs THIS SAME binary
/// (Hilbert dispatches `cargo run --release` in the repo's own Actions), so the
/// cap that gates a submission on the server is exactly this constant — local
/// and graded runs use the identical 2 s cap by construction.
const TIME_CAP_PER_MATRIX: Duration = Duration::from_secs(2);
const CANDIDATE_WORKER_ENV: &str = "SSI_CANDIDATE_WORKER";

/// Parent-only smoke/CI knob: when set to a positive integer, the harness scores
/// only the corpus matrices whose dimension `n` is at most this bound. It lets a
/// fast regression test exercise the full public-dev output format over the small
/// matrices WITHOUT depending on `order()` beating the per-matrix cap on the
/// largest ones (whose wall-clock time is runner-speed variable). Read from the
/// trusted PARENT's environment only: the grader never
/// sets it, and contestant worker code cannot reach the parent env, so graded
/// eval runs always score the whole corpus. It narrows the scored set only —
/// the score DEFINITION, the census gate, and the hidden-corpus privacy path
/// are all unchanged.
const MAX_MATRIX_N_ENV: &str = "SSI_MAX_MATRIX_N";

/// Parent-only regression flag, accepted from the time-cap integration test:
/// every worker is launched in the candidate binary's harness-owned
/// `--worker-test-timeout` mode, which sleeps past the cap without ever calling
/// `order()`. Production workers receive neither this flag nor any test
/// environment variable, so contestant code cannot observe the test seam.
const TEST_TIME_CAP_FLAG: &str = "--test-time-cap";

fn main() -> ExitCode {
    let raw_args: Vec<String> = std::env::args().collect();
    let grader_mode = raw_args.iter().any(|arg| arg == "--grader");
    let worker_sandbox = match sandbox::WorkerSandbox::from_env(grader_mode) {
        Ok(sandbox) => sandbox,
        Err(e) => {
            eprintln!("RUN FAILED (worker sandbox: {e})");
            return ExitCode::FAILURE;
        }
    };
    // A local run reaches Disabled only through the explicit opt-out; make that
    // choice loud so it is never a silent regression to unsandboxed execution.
    if !grader_mode && !worker_sandbox.is_enabled() {
        eprintln!(
            "WARNING: {}=1 — running the candidate worker WITHOUT a sandbox. \
             A malicious pushed-back submission can reach your network and files. \
             Unset it to sandbox order().",
            sandbox::ALLOW_UNSANDBOXED_ENV
        );
    }
    let candidate_worker = match candidate_worker_path(grader_mode) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("RUN FAILED (candidate worker: {e})");
            return ExitCode::FAILURE;
        }
    };
    if raw_args.iter().any(|arg| arg == "--sandbox-self-check") {
        return match sandbox_self_check_parent(&worker_sandbox, &candidate_worker) {
            Ok(()) => {
                println!("per-worker bubblewrap self-check passed");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("sandbox self-check failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    if raw_args
        .iter()
        .any(|arg| arg == "--sandbox-timeout-self-check")
    {
        return match sandbox_timeout_self_check_parent(&worker_sandbox, &candidate_worker) {
            Ok(()) => {
                println!("bubblewrap timeout teardown self-check passed");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("sandbox timeout self-check failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    let test_time_cap = raw_args.iter().any(|arg| arg == TEST_TIME_CAP_FLAG);

    let note = parse_note();
    let repo_root = repo_root();
    // Preserve the trusted pre-run log before any contestant worker can write
    // through an absolute path. Artifact publication happens only after all
    // workers finish and discards any bytes they may have forged.
    let run_artifacts = match artifacts::RunArtifacts::capture(&repo_root) {
        Ok(state) => state,
        Err(e) => {
            let reason = format!("could not preserve pre-run artifact state: {e}");
            println!("RUN FAILED: {reason}");
            if let Err(clear_error) = artifacts::clear_score(&repo_root) {
                eprintln!("could not clear score.json after failure: {clear_error}");
            }
            return ExitCode::FAILURE;
        }
    };

    // --- Stage A analog: purity & license gate, before any scoring. ---
    if let Err(e) = purity::check(&repo_root) {
        let reason = format!("Stage A — purity/license: {e}");
        return finish_failed_run(&run_artifacts, &reason, &note);
    }

    let corpus_file = corpus::corpus_path();
    let show_matrix_census = is_public_dev_corpus(&repo_root, &corpus_file);
    let corpus = match corpus::load(&corpus_file) {
        Ok(corpus) => corpus,
        Err(detail) => {
            // A malformed hidden corpus must neither remain on disk nor leak
            // matrix names through the parse error into public logs.
            if !show_matrix_census {
                let _ = std::fs::remove_file(&corpus_file);
            }
            let reason = if show_matrix_census {
                detail
            } else {
                "failed to load hidden corpus".to_string()
            };
            return finish_failed_run(&run_artifacts, &reason, &note);
        }
    };
    // The corpus is fully resident in memory now and no worker has run yet.
    // If this is a grader-supplied hidden eval corpus, unlink it so worker
    // children cannot read the eval set from disk (F2). The default dev
    // corpus lives in the repo tree, is public, and is left alone — matched
    // by canonical path so an absolute spelling of it is never deleted.
    if !show_matrix_census {
        let _ = std::fs::remove_file(&corpus_file);
    }
    // Optional parent-only size filter (see MAX_MATRIX_N_ENV). Applied after the
    // hidden-corpus unlink so it never changes the privacy path — only which of
    // the already-loaded matrices get scored. Gated on `show_matrix_census`
    // (the public-dev path) so a leaked env var can NEVER narrow a graded /
    // hidden-corpus run: the "grader always scores the whole corpus" promise is
    // enforced in code here, not merely by the grader declining to set the var.
    let corpus = match (show_matrix_census, max_matrix_n()) {
        (true, Some(max_n)) => corpus
            .into_iter()
            .filter(|(_, pat)| pat.n <= max_n)
            .collect(),
        _ => corpus,
    };
    if corpus.is_empty() {
        let reason = format!(
            "no patterns found at {}. Run from the repo root.",
            corpus_file.display()
        );
        return finish_failed_run(&run_artifacts, &reason, &note);
    }

    if show_matrix_census {
        println!(
            "{:<28} {:>8} {:>10} {:>14} {:>14} {:>8} {:>9}",
            "matrix", "n", "nnz(A)", "flops(base)", "flops(yours)", "ratio", "time"
        );
    }

    let mut buckets = [BucketAcc::default(); BUCKETS];
    let mut failed: Option<String> = None;
    let mut table = String::new();

    // Create a scratch dir for worker protocol files.
    let scratch = failsafe::ScratchDir(
        std::env::temp_dir().join(format!("ssi-harness-{}", std::process::id())),
    );
    std::fs::create_dir_all(scratch.path()).expect("create scratch dir");
    let cap = watchdog::CapConfig {
        time_cap: TIME_CAP_PER_MATRIX,
        poll: std::time::Duration::from_millis(10),
    };

    for (seq, (name, pat)) in corpus.iter().enumerate() {
        // --- AMD baseline (trusted, in-process) — guarded so a feral panic
        // or an i32-overflow-sized pattern becomes a recorded FAIL, not a
        // process abort that leaks scratch. ---
        let base = match failsafe::catch(std::panic::AssertUnwindSafe(|| {
            let base_perm = ssi_scoring::amd_baseline(pat);
            score(pat, &base_perm)
        })) {
            Ok(b) => b,
            Err(msg) => {
                failed = Some(matrix_failure(
                    show_matrix_census,
                    name,
                    &format!("trusted baseline/score panicked — {msg}"),
                    "trusted baseline/score panicked",
                ));
                break;
            }
        };

        // Serialize THIS pattern once to the scratch dir; both determinism runs
        // read it. Written by the trusted parent OUTSIDE the timed window, so its
        // cost never counts against the per-matrix cap.
        // The per-file guard unlinks it after both runs and on every earlier
        // break/unwind; the whole-directory guard remains a final fallback.
        let pat_file = failsafe::StagedFile::new(scratch.path().join(format!("{seq}-pat.bin")));
        if let Err(e) = ssi_worker_protocol::write_pattern(pat_file.path(), pat) {
            failed = Some(matrix_failure(
                show_matrix_census,
                name,
                &format!("failed to stage pattern for worker: {e}"),
                "failed to stage pattern for worker",
            ));
            break;
        }

        // --- contestant ordering: capped subprocess, run twice ---
        let run_once = |tag: &str| -> Result<Vec<usize>, WorkerFailure> {
            let out_perm = scratch.path().join(format!("{seq}-{tag}.bin"));
            let _ = std::fs::remove_file(&out_perm);
            // Pre-create the output so grader mode can bind precisely one
            // writable file. No directory is writable in the worker sandbox.
            OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&out_perm)
                .map_err(|e| format!("failed to stage worker output: {e}"))?;
            // Local mode keeps the historical private empty CWD. Grader mode
            // replaces the entire filesystem and does not expose this path.
            let worker_cwd = scratch.path().join(format!("{seq}-{tag}-cwd"));
            std::fs::create_dir_all(&worker_cwd)
                .map_err(|e| format!("failed to stage worker cwd: {e}"))?;
            let invocation = if test_time_cap {
                // Test-only harness-owned worker mode: it sleeps without ever
                // invoking order(), so the watchdog is exercised against a real
                // child while contestant code cannot inspect the test flag.
                sandbox::WorkerInvocation::TestTimeout
            } else {
                sandbox::WorkerInvocation::Order
            };
            let mut cmd = worker_sandbox.worker_command(
                &candidate_worker,
                invocation,
                pat_file.path(),
                &out_perm,
                &worker_cwd,
                &sandbox::WorkerLimits::for_order(pat.n),
            )?;
            let t0 = Instant::now();
            match watchdog::run_capped(&mut cmd, &cap) {
                watchdog::WorkerOutcome::Ok => {
                    ssi_worker_protocol::read_permutation(&out_perm, pat.n).map_err(|e| {
                        WorkerFailure {
                            // The parse error can echo bytes from the worker-written
                            // output file, so keep it off the hidden path.
                            detailed: format!("worker produced no readable permutation: {e}"),
                            redacted: "worker produced no readable permutation".to_string(),
                        }
                    })
                }
                other => Err(classify_worker_failure(
                    other,
                    t0.elapsed(),
                    pat.n,
                    pat.nnz(),
                )),
            }
        };

        let perm1 = match run_once("a") {
            Ok(p) => p,
            Err(e) => {
                failed = Some(matrix_failure(
                    show_matrix_census,
                    name,
                    &e.detailed,
                    &e.redacted,
                ));
                break;
            }
        };
        let perm2 = match run_once("b") {
            Ok(p) => p,
            Err(e) => {
                failed = Some(matrix_failure(
                    show_matrix_census,
                    name,
                    &e.detailed,
                    &e.redacted,
                ));
                break;
            }
        };
        // Neither worker needs the staged input now. Unlink immediately instead
        // of retaining this matrix until whole-run scratch cleanup. Any failure
        // aborts before another worker can observe the leftover file; the guard
        // retries on this loop exit.
        if let Err(e) = pat_file.remove_now() {
            failed = Some(matrix_failure(
                show_matrix_census,
                name,
                &format!("failed to remove staged worker pattern: {e}"),
                "failed to remove staged worker input",
            ));
            break;
        }
        if perm1 != perm2 {
            failed = Some(matrix_failure(
                show_matrix_census,
                name,
                "nondeterministic ordering (two runs differ)",
                "nondeterministic ordering (two runs differ)",
            ));
            break;
        }
        if let Err(e) = validate_permutation(&perm1, pat.n) {
            failed = Some(matrix_failure(
                show_matrix_census,
                name,
                &format!("invalid permutation — {e}"),
                "invalid permutation",
            ));
            break;
        }

        // --- trusted scoring (Stage D), same path as the grader — guarded ---
        let yours = match failsafe::catch(std::panic::AssertUnwindSafe(|| score(pat, &perm1))) {
            Ok(s) => s,
            Err(msg) => {
                failed = Some(matrix_failure(
                    show_matrix_census,
                    name,
                    &format!("trusted scoring panicked — {msg}"),
                    "trusted scoring panicked",
                ));
                break;
            }
        };
        let ratio = yours.flops as f64 / base.flops as f64;
        let fill_ratio = yours.nnz_l as f64 / base.nnz_l as f64;
        let b = size_bucket(pat.n);
        buckets[b].log_ratio_sum += ratio.ln();
        buckets[b].log_fill_sum += fill_ratio.ln();
        buckets[b].count += 1;

        if show_matrix_census {
            let line = format!(
                "{:<28} {:>8} {:>10} {:>14} {:>14} {:>8.3} {:>9}",
                name,
                pat.n,
                pat.nnz(),
                base.flops,
                yours.flops,
                ratio,
                "(capped)"
            );
            println!("{line}");
            let _ = writeln!(table, "{line}");
        }
    }

    let timestamp = now();

    match failed {
        Some(reason) => finish_failed_run_at(&run_artifacts, timestamp, &reason, &note),
        None => {
            // Per-bucket geomeans (None for an empty bucket).
            let flop_gms: [Option<f64>; BUCKETS] =
                std::array::from_fn(|i| geomean(buckets[i].log_ratio_sum, buckets[i].count));
            let fill_gms: [Option<f64>; BUCKETS] =
                std::array::from_fn(|i| geomean(buckets[i].log_fill_sum, buckets[i].count));

            let score_val = combine(&flop_gms, &BUCKET_WEIGHTS);
            let fill = combine(&fill_gms, &BUCKET_WEIGHTS);

            // Bucket-level diagnostics disclose both occupancy and relative
            // performance, so they are emitted only for the public dev corpus.
            if show_matrix_census {
                println!("\nper-bucket (geomean of ratio vs AMD, within bucket):");
                println!(
                    "{:<8} {:>6} {:>16} {:>16}",
                    "bucket", "count", "flop_geomean", "fill_geomean"
                );
                for i in 0..BUCKETS {
                    let fmt =
                        |g: Option<f64>| g.map_or_else(|| "—".to_string(), |v| format!("{v:.4}"));
                    println!(
                        "{:<8} {:>6} {:>16} {:>16}",
                        BUCKET_KEYS[i],
                        buckets[i].count,
                        fmt(flop_gms[i]),
                        fmt(fill_gms[i]),
                    );
                }
            }

            println!(
                "\nscore (weighted mean of per-bucket geomean flop ratios, lower is better): {score_val:.4}"
            );
            println!("tiebreak (weighted mean of per-bucket geomean fill ratios):                    {fill:.4}");

            // score.json — top-level `score` is what the grader ranks on;
            // `metrics` is passthrough detail (Hilbert captures it whole and
            // shows it in the PR report). Preserve the public-dev schema, but
            // hidden artifacts expose aggregate score/tiebreak metrics only.
            let json = if show_matrix_census {
                let mut buckets_json = String::new();
                for i in 0..BUCKETS {
                    let jf = |g: Option<f64>| {
                        g.map_or_else(|| "null".to_string(), |v| format!("{v:.6}"))
                    };
                    let sep = if i + 1 < BUCKETS { "," } else { "" };
                    let _ = write!(
                        buckets_json,
                        "\"{}\": {{ \"count\": {}, \"geomean_flop_ratio\": {}, \"geomean_fill_ratio\": {} }}{}",
                        BUCKET_KEYS[i],
                        buckets[i].count,
                        jf(flop_gms[i]),
                        jf(fill_gms[i]),
                        sep,
                    );
                }
                let total: usize = buckets.iter().map(|b| b.count).sum();
                format!(
                    "{{ \"score\": {score_val:.6}, \"metrics\": {{ \
                     \"geomean_flop_ratio\": {score_val:.6}, \
                     \"geomean_fill_ratio\": {fill:.6}, \
                     \"matrices\": {total}, \
                     \"weights\": {{ \"lt_1k\": {:.2}, \"1k_10k\": {:.2}, \"gt_10k\": {:.2} }}, \
                     \"buckets\": {{ {buckets_json} }} }} }}\n",
                    BUCKET_WEIGHTS[0], BUCKET_WEIGHTS[1], BUCKET_WEIGHTS[2],
                )
            } else {
                format!(
                    "{{ \"score\": {score_val:.6}, \"metrics\": {{ \
                     \"geomean_flop_ratio\": {score_val:.6}, \
                     \"geomean_fill_ratio\": {fill:.6} }} }}\n"
                )
            };
            let row = result_row(timestamp, "OK", score_val, fill, &note);
            match run_artifacts.finish_success(&row, &json) {
                Ok(()) => ExitCode::SUCCESS,
                Err(e) => {
                    let reason = format!("could not publish trusted run artifacts: {e}");
                    println!("\nRUN FAILED: {reason}");
                    let fail_row = result_row(
                        timestamp,
                        "FAIL",
                        f64::NAN,
                        f64::NAN,
                        &failsafe::compose_note(&reason, &note),
                    );
                    if let Err(record_error) = run_artifacts.finish_failure(&fail_row) {
                        eprintln!(
                            "could not clear/rewrite artifacts after publication failure: \
                             {record_error}"
                        );
                    }
                    ExitCode::FAILURE
                }
            }
        }
    }
}

/// Exercise the exact per-worker bubblewrap command used by grading. CI runs
/// this after the sandboxed build, before downloading the hidden corpus.
fn sandbox_self_check_parent(
    worker_sandbox: &sandbox::WorkerSandbox,
    candidate_worker: &Path,
) -> Result<(), String> {
    if !worker_sandbox.is_enabled() {
        return Err(format!(
            "{}=bubblewrap is required for this check",
            sandbox::MODE_ENV
        ));
    }
    let scratch = failsafe::ScratchDir(
        std::env::temp_dir().join(format!("ssi-sandbox-check-{}", std::process::id())),
    );
    std::fs::create_dir_all(scratch.path())
        .map_err(|e| format!("create self-check scratch dir: {e}"))?;
    let input = scratch.path().join("input");
    let output = scratch.path().join("output");
    let cwd = scratch.path().join("cwd");
    std::fs::write(&input, b"read-only sentinel")
        .map_err(|e| format!("create self-check input: {e}"))?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|e| format!("create self-check output: {e}"))?;
    std::fs::create_dir(&cwd).map_err(|e| format!("create self-check cwd: {e}"))?;

    let mut cmd = worker_sandbox.worker_command(
        candidate_worker,
        sandbox::WorkerInvocation::SandboxSelfCheck,
        &input,
        &output,
        &cwd,
        &sandbox::WorkerLimits::for_self_check(),
    )?;
    let cfg = watchdog::CapConfig {
        time_cap: Duration::from_secs(5),
        poll: Duration::from_millis(10),
    };
    let outcome = watchdog::run_capped(&mut cmd, &cfg);
    let report = std::fs::read_to_string(&output).unwrap_or_default();
    match outcome {
        watchdog::WorkerOutcome::Ok if report == "ok\n" => Ok(()),
        other => Err(format!("worker reported {other:?}: {}", report.trim())),
    }
}

fn sandbox_timeout_self_check_parent(
    worker_sandbox: &sandbox::WorkerSandbox,
    candidate_worker: &Path,
) -> Result<(), String> {
    if !worker_sandbox.is_enabled() {
        return Err(format!(
            "{}=bubblewrap is required for this check",
            sandbox::MODE_ENV
        ));
    }
    let scratch = failsafe::ScratchDir(
        std::env::temp_dir().join(format!("ssi-sandbox-timeout-{}", std::process::id())),
    );
    std::fs::create_dir_all(scratch.path())
        .map_err(|e| format!("create timeout-check scratch dir: {e}"))?;
    let input = scratch.path().join("input");
    let output = scratch.path().join("output");
    let cwd = scratch.path().join("cwd");
    std::fs::write(&input, b"read-only sentinel")
        .map_err(|e| format!("create timeout-check input: {e}"))?;
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&output)
        .map_err(|e| format!("create timeout-check output: {e}"))?;
    std::fs::create_dir(&cwd).map_err(|e| format!("create timeout-check cwd: {e}"))?;

    let mut cmd = worker_sandbox.worker_command(
        candidate_worker,
        sandbox::WorkerInvocation::SandboxTimeoutSelfCheck,
        &input,
        &output,
        &cwd,
        &sandbox::WorkerLimits::for_self_check(),
    )?;
    let cfg = watchdog::CapConfig {
        time_cap: Duration::from_millis(200),
        poll: Duration::from_millis(5),
    };
    let outcome = watchdog::run_capped(&mut cmd, &cfg);
    let report = std::fs::read_to_string(&output).unwrap_or_default();
    match outcome {
        watchdog::WorkerOutcome::Timeout if report == "candidate started\n" => Ok(()),
        other => Err(format!("worker reported {other:?}: {}", report.trim())),
    }
}

fn candidate_worker_path(grader_mode: bool) -> Result<PathBuf, String> {
    if let Some(path) = std::env::var_os(CANDIDATE_WORKER_ENV) {
        if path.is_empty() {
            return Err(format!("{CANDIDATE_WORKER_ENV} is set but empty"));
        }
        return require_candidate_worker(PathBuf::from(path));
    }
    if grader_mode {
        return Err(format!(
            "{CANDIDATE_WORKER_ENV} must name the separately built candidate binary in --grader mode"
        ));
    }

    let mut path =
        std::env::current_exe().map_err(|e| format!("locate trusted parent executable: {e}"))?;
    path.set_file_name(format!(
        "ssi-candidate-worker{}",
        std::env::consts::EXE_SUFFIX
    ));
    require_candidate_worker(path)
}

fn require_candidate_worker(path: PathBuf) -> Result<PathBuf, String> {
    let canonical = std::fs::canonicalize(&path).map_err(|error| {
        format!(
            "candidate worker not found at {}: {error}. Build the untrusted candidate worker \
             (sandboxed) with `bash scripts/local-candidate-build.sh` before `cargo run`, and \
             rebuild `ssi-candidate-worker` after ordering edits",
            path.display()
        )
    })?;
    if !canonical.is_file() {
        return Err(format!(
            "candidate worker is not a regular file: {}. Build the untrusted candidate worker \
             (sandboxed) with `bash scripts/local-candidate-build.sh` before `cargo run`",
            canonical.display()
        ));
    }
    Ok(canonical)
}

/// Resolve the repo root so the gate finds src/ordering/ and deny.toml whether
/// the binary is launched from the repo root or elsewhere. Prefer the package
/// directory embedded by Cargo, then walk up from the executable and current
/// directory. The latter is required for grader builds: the build sandbox
/// deliberately exposes the checkout as `/workspace`, which does not exist
/// after the resulting trusted binary returns to the host runner.
fn repo_root() -> std::path::PathBuf {
    let compiled = Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = std::env::current_exe().ok();
    let cwd = std::env::current_dir().ok();
    resolve_repo_root(compiled, executable.as_deref(), cwd.as_deref())
}

fn resolve_repo_root(compiled: &Path, executable: Option<&Path>, cwd: Option<&Path>) -> PathBuf {
    find_repo_root(compiled)
        .or_else(|| executable.and_then(find_repo_root))
        .or_else(|| cwd.and_then(find_repo_root))
        .unwrap_or_else(|| compiled.to_path_buf())
}

fn find_repo_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|path| path.join("deny.toml").is_file() && path.join("src/ordering").is_dir())
        .map(Path::to_path_buf)
}

fn is_public_dev_corpus(repo_root: &Path, path: &Path) -> bool {
    // Canonical comparison recognizes absolute paths and symlinked aliases to
    // the shipped public corpus without trusting a matching relative spelling
    // in some other working directory. Any resolution failure stays
    // fail-closed: an unrecognized path is treated as hidden.
    let repo_dev = repo_root.join(corpus::DEV_CORPUS_FILE);
    match (std::fs::canonicalize(path), std::fs::canonicalize(repo_dev)) {
        (Ok(candidate), Ok(public_dev)) => candidate == public_dev,
        _ => false,
    }
}

/// Parse the parent-only `SSI_MAX_MATRIX_N` bound (see `MAX_MATRIX_N_ENV`).
/// Returns `None` — score everything — when the variable is unset, blank,
/// not a valid `usize`, or `0`, so a mistyped value can never silently score
/// zero matrices. Graded runs never set it.
fn max_matrix_n() -> Option<usize> {
    parse_max_matrix_n(std::env::var(MAX_MATRIX_N_ENV).ok())
}

/// Pure parser behind [`max_matrix_n`], split out so its fail-open behavior is
/// unit-testable without mutating the process environment. A bound of `0` would
/// filter out every matrix (all `n >= 1`) and surface as a misleading "no
/// patterns found" error, so it is treated as "no limit" like any other
/// unusable value.
fn parse_max_matrix_n(raw: Option<String>) -> Option<usize> {
    raw.and_then(|v| v.trim().parse::<usize>().ok())
        .filter(|&n| n > 0)
}

/// A worker failure carries two renderings of the same event: `detailed` (for
/// the PUBLIC dev corpus, includes the matrix census) and `redacted` (for the
/// HIDDEN eval corpus, names the failure category but never the census). Both
/// are corpus-neutral — the matrix name and its n/nnz are added/withheld by
/// [`matrix_failure`], not here.
struct WorkerFailure {
    detailed: String,
    redacted: String,
}

impl From<String> for WorkerFailure {
    /// Harness-internal staging/sandbox errors carry no matrix census, so the
    /// same message is safe on both the public and hidden paths.
    fn from(msg: String) -> Self {
        WorkerFailure {
            redacted: msg.clone(),
            detailed: msg,
        }
    }
}

/// Classify a non-success worker outcome into a [`WorkerFailure`]. The redacted
/// rendering distinguishes a time-cap kill from an abnormal exit — the
/// actionable bit for a contestant — while withholding the n/nnz census that
/// would fingerprint a hidden eval matrix. `describe_status`-derived crash
/// detail is harness-generated (exit code/signal), never worker output, so it
/// is safe on both paths.
fn classify_worker_failure(
    outcome: watchdog::WorkerOutcome,
    elapsed: Duration,
    n: usize,
    nnz: usize,
) -> WorkerFailure {
    let cap = TIME_CAP_PER_MATRIX.as_secs_f64();
    match outcome {
        watchdog::WorkerOutcome::Timeout => WorkerFailure {
            detailed: format!(
                "order() exceeded the {:.1}s per-matrix cap and was killed (took ≥ {:.1}s). \
                 Your ordering must return within {:.0}s on every matrix. This matrix is \
                 n={}, nnz={}, nnz/n≈{} — if it is dense, the cost is in order() itself; \
                 gate expensive paths by BOTH n and nnz.",
                cap,
                elapsed.as_secs_f64(),
                cap,
                n,
                nnz,
                nnz / n.max(1)
            ),
            redacted: format!("order() exceeded the {cap:.1}s per-matrix cap and was killed"),
        },
        watchdog::WorkerOutcome::Crashed(crash) => {
            // The redacted category is keyed off the OS/harness-determined
            // CrashKind, never off `crash.detail` — a nonzero exit code is
            // worker-chosen (`std::process::exit(f(n))`), so echoing it on the
            // hidden path would be a covert channel for the eval matrix. The
            // signal-vs-exit split is the actionable bit (memory cap / segfault
            // vs a panic) and leaks at most one OS-determined bit, no number.
            let redacted = match crash.kind {
                watchdog::CrashKind::Signal => {
                    "order() worker was killed by a signal (e.g. it hit the memory cap or segfaulted)"
                }
                watchdog::CrashKind::NonzeroExit => {
                    "order() worker exited nonzero (e.g. order() panicked)"
                }
                watchdog::CrashKind::Harness => "order() worker could not be run",
            };
            WorkerFailure {
                detailed: format!("order() worker exited abnormally: {}", crash.detail),
                redacted: redacted.to_string(),
            }
        }
        watchdog::WorkerOutcome::Ok => {
            unreachable!("classify_worker_failure called on a successful worker outcome")
        }
    }
}

fn matrix_failure(show_matrix_census: bool, name: &str, detailed: &str, redacted: &str) -> String {
    if show_matrix_census {
        format!("{name}: {detailed}")
    } else {
        format!("hidden matrix: {redacted}")
    }
}

#[cfg(test)]
mod max_matrix_n_tests {
    use super::parse_max_matrix_n;

    #[test]
    fn unset_scores_everything() {
        assert_eq!(parse_max_matrix_n(None), None);
    }

    #[test]
    fn blank_or_invalid_fails_open_to_no_limit() {
        // A mistyped bound must never silently narrow the corpus to nothing.
        assert_eq!(parse_max_matrix_n(Some(String::new())), None);
        assert_eq!(parse_max_matrix_n(Some("  ".to_string())), None);
        assert_eq!(parse_max_matrix_n(Some("2k".to_string())), None);
        assert_eq!(parse_max_matrix_n(Some("-5".to_string())), None);
        // 0 would filter out every matrix (all n >= 1); treat it as no limit
        // rather than fail with a misleading "no patterns found" error.
        assert_eq!(parse_max_matrix_n(Some("0".to_string())), None);
        assert_eq!(parse_max_matrix_n(Some(" 0 ".to_string())), None);
    }

    #[test]
    fn valid_bound_parses_and_tolerates_whitespace() {
        assert_eq!(parse_max_matrix_n(Some("2000".to_string())), Some(2000));
        assert_eq!(parse_max_matrix_n(Some(" 1 ".to_string())), Some(1));
    }
}

#[cfg(test)]
mod repo_root_tests {
    use super::*;

    const MISSING_SANDBOX_ROOT: &str = "/__ssi_missing_build_sandbox_workspace__";

    #[test]
    fn uses_valid_compiled_manifest_directory() {
        let expected = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(resolve_repo_root(expected, None, None), expected);
    }

    #[test]
    fn falls_back_from_build_sandbox_path_to_executable_location() {
        let expected = Path::new(env!("CARGO_MANIFEST_DIR"));
        let executable = expected.join("target/trusted/release/matrices-fast");
        assert_eq!(
            resolve_repo_root(
                Path::new(MISSING_SANDBOX_ROOT),
                Some(&executable),
                Some(Path::new("/")),
            ),
            expected
        );
    }

    #[test]
    fn falls_back_to_current_directory_when_executable_is_external() {
        let expected = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert_eq!(
            resolve_repo_root(
                Path::new(MISSING_SANDBOX_ROOT),
                Some(Path::new("/tmp/matrices-fast")),
                Some(expected),
            ),
            expected
        );
    }
}

#[cfg(test)]
mod census_tests {
    use super::*;

    #[test]
    fn public_dev_corpus_recognizes_relative_and_canonical_paths() {
        let root = repo_root();
        let relative = Path::new(corpus::DEV_CORPUS_FILE);
        assert!(is_public_dev_corpus(&root, relative));

        let canonical = std::fs::canonicalize(root.join(relative))
            .expect("canonicalize shipped public dev corpus");
        assert!(is_public_dev_corpus(&root, &canonical));
        assert!(!is_public_dev_corpus(
            &root,
            &std::env::temp_dir().join("hidden-eval.jsonl")
        ));
    }

    #[test]
    fn hidden_matrix_failures_redact_names_and_details() {
        let reason = matrix_failure(
            false,
            "secret-matrix-name",
            "timeout at n=1234, nnz=5678",
            "ordering timed out",
        );
        assert_eq!(reason, "hidden matrix: ordering timed out");
        assert!(!reason.contains("secret-matrix-name"));
        assert!(!reason.contains("1234"));
        assert!(!reason.contains("5678"));
    }

    #[test]
    fn public_dev_matrix_failures_keep_actionable_details() {
        let reason = matrix_failure(
            true,
            "dev-matrix",
            "timeout at n=12, nnz=34",
            "ordering timed out",
        );
        assert_eq!(reason, "dev-matrix: timeout at n=12, nnz=34");
    }

    #[test]
    fn timeout_failure_names_the_cap_and_withholds_census() {
        let f = classify_worker_failure(
            watchdog::WorkerOutcome::Timeout,
            Duration::from_secs_f64(2.5),
            1234,
            5678,
        );
        // Shown on the hidden eval corpus: names the time cap, not the old
        // opaque "worker ordering failed", and never leaks the census.
        assert_ne!(f.redacted, "worker ordering failed");
        assert!(f.redacted.contains("cap"), "redacted: {}", f.redacted);
        assert!(!f.redacted.contains("1234"));
        assert!(!f.redacted.contains("5678"));
        // Public dev corpus keeps the actionable n/nnz census.
        assert!(f.detailed.contains("1234"));
        assert!(f.detailed.contains("5678"));
    }

    #[test]
    fn crash_redacted_never_carries_the_worker_controlled_exit_code() {
        // A nonzero exit code is worker-chosen (`std::process::exit(f(n))`), so
        // it must never reach the hidden-corpus log — only the fact of a crash.
        let f = classify_worker_failure(
            watchdog::WorkerOutcome::Crashed(watchdog::CrashDetail {
                kind: watchdog::CrashKind::NonzeroExit,
                detail: "worker exited with code 17".to_string(),
            }),
            Duration::ZERO,
            357,
            1080,
        );
        // Distinct from a time-cap kill, and free of the worker-chosen number.
        assert!(!f.redacted.contains("cap"), "redacted: {}", f.redacted);
        assert!(
            !f.redacted.contains("17"),
            "redacted leaked code: {}",
            f.redacted
        );
        // The full detail (public dev corpus only) may keep the code.
        assert!(f.detailed.contains("17"));
    }

    #[test]
    fn crash_redacted_distinguishes_signal_from_nonzero_exit() {
        let signal = classify_worker_failure(
            watchdog::WorkerOutcome::Crashed(watchdog::CrashDetail {
                kind: watchdog::CrashKind::Signal,
                detail: "worker killed by signal 6".to_string(),
            }),
            Duration::ZERO,
            10,
            20,
        );
        let exited = classify_worker_failure(
            watchdog::WorkerOutcome::Crashed(watchdog::CrashDetail {
                kind: watchdog::CrashKind::NonzeroExit,
                detail: "worker exited with code 101".to_string(),
            }),
            Duration::ZERO,
            10,
            20,
        );
        // A signal (memory cap / segfault) reads differently from a nonzero
        // exit (panic / explicit exit) — the actionable distinction for a
        // contestant — but neither carries the signal or code NUMBER.
        assert!(signal.redacted.contains("signal"), "{}", signal.redacted);
        assert!(!signal.redacted.contains('6'));
        assert!(!exited.redacted.contains("101"));
        assert_ne!(signal.redacted, exited.redacted);
    }
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn finish_failed_run(artifacts: &artifacts::RunArtifacts, reason: &str, note: &str) -> ExitCode {
    finish_failed_run_at(artifacts, now(), reason, note)
}

fn finish_failed_run_at(
    artifacts: &artifacts::RunArtifacts,
    timestamp: u64,
    reason: &str,
    note: &str,
) -> ExitCode {
    println!("\nRUN FAILED: {reason}");
    let row = result_row(
        timestamp,
        "FAIL",
        f64::NAN,
        f64::NAN,
        &failsafe::compose_note(reason, note),
    );
    if let Err(e) = artifacts.finish_failure(&row) {
        eprintln!("could not clear/rewrite failed-run artifacts: {e}");
    }
    ExitCode::FAILURE
}

fn result_row(ts: u64, status: &str, score: f64, fill: f64, note: &str) -> String {
    format!("{ts}\t{status}\t{score:.6}\t{fill:.6}\t{note}")
}

fn parse_note() -> String {
    let args: Vec<String> = std::env::args().collect();
    for i in 0..args.len() {
        if args[i] == "--note" && i + 1 < args.len() {
            return failsafe::sanitize(&args[i + 1]);
        }
    }
    String::new()
}
