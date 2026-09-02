//! Trusted publication of run artifacts after all contestant workers exit.
//!
//! A worker shares the harness UID and can address the repository by an
//! absolute path even though its CWD is empty. Treat `score.json` and
//! `results.tsv` as untrusted until the parent has finished supervising every
//! worker, then replace them from parent-owned state.

use std::cell::Cell;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

const RESULTS_FILE: &str = "results.tsv";
const SCORE_FILE: &str = "score.json";
static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(0);

/// Parent-owned artifact state captured before any contestant worker runs.
pub struct RunArtifacts {
    results_path: PathBuf,
    score_path: PathBuf,
    original_results: Vec<u8>,
    score_is_authoritative: Cell<bool>,
}

impl RunArtifacts {
    pub fn capture(repo_root: &Path) -> io::Result<Self> {
        let results_path = repo_root.join(RESULTS_FILE);
        let original_results = match fs::read(&results_path) {
            Ok(bytes) => bytes,
            Err(e) if e.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(e) => return Err(e),
        };
        Ok(Self {
            results_path,
            score_path: repo_root.join(SCORE_FILE),
            original_results,
            score_is_authoritative: Cell::new(false),
        })
    }

    /// Publish a failed run: no score may remain, and any worker-written result
    /// rows are replaced by the pre-run bytes plus this one trusted row.
    pub fn finish_failure(&self, row: &str) -> io::Result<()> {
        self.score_is_authoritative.set(false);
        let score_result = remove_any(&self.score_path);
        let results_result = self.rewrite_results(row);
        score_result.and(results_result)
    }

    /// Publish a successful run while preserving the existing score JSON
    /// payload byte-for-byte. Both files are replaced only after workers exit.
    pub fn finish_success(&self, row: &str, score_json: &str) -> io::Result<()> {
        self.score_is_authoritative.set(false);
        remove_any(&self.score_path)?;
        self.rewrite_results(row)?;
        if let Err(e) = atomic_write(&self.score_path, score_json.as_bytes()) {
            let _ = remove_any(&self.score_path);
            return Err(e);
        }
        self.score_is_authoritative.set(true);
        Ok(())
    }

    fn rewrite_results(&self, row: &str) -> io::Result<()> {
        if row.contains(['\r', '\n']) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trusted result row must contain exactly one line",
            ));
        }

        let mut contents = self.original_results.clone();
        if !contents.is_empty() && !contents.ends_with(b"\n") {
            contents.push(b'\n');
        }
        contents.extend_from_slice(row.as_bytes());
        contents.push(b'\n');
        atomic_write(&self.results_path, &contents)
    }
}

impl Drop for RunArtifacts {
    fn drop(&mut self) {
        if !self.score_is_authoritative.get() {
            let _ = remove_any(&self.score_path);
        }
    }
}

/// Best-effort fail-closed cleanup for the rare case where artifact-state
/// capture itself fails before a `RunArtifacts` value can be constructed.
pub fn clear_score(repo_root: &Path) -> io::Result<()> {
    remove_any(&repo_root.join(SCORE_FILE))
}

fn atomic_write(path: &Path, contents: &[u8]) -> io::Result<()> {
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("artifact");

    let temp_path = loop {
        let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
        let candidate = parent.join(format!(
            ".{file_name}.trusted-{}-{id}.tmp",
            std::process::id()
        ));
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&candidate)
        {
            Ok(mut file) => {
                if let Err(e) = file.write_all(contents).and_then(|_| file.sync_all()) {
                    let _ = fs::remove_file(&candidate);
                    return Err(e);
                }
                drop(file);
                break candidate;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e),
        }
    };

    // A worker may replace an artifact with a directory, which rename(2) cannot
    // overwrite. Regular files and symlinks are atomically replaced directly.
    if fs::symlink_metadata(path)
        .map(|meta| meta.file_type().is_dir())
        .unwrap_or(false)
    {
        if let Err(e) = fs::remove_dir_all(path) {
            let _ = fs::remove_file(&temp_path);
            return Err(e);
        }
    }

    if let Err(e) = fs::rename(&temp_path, path) {
        let _ = fs::remove_file(&temp_path);
        return Err(e);
    }
    Ok(())
}

fn remove_any(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Ok(meta) if meta.file_type().is_dir() => fs::remove_dir_all(path),
        Ok(_) => fs::remove_file(path),
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(e),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static NEXT_TEST_ID: AtomicU64 = AtomicU64::new(0);

    fn test_dir(label: &str) -> PathBuf {
        let id = NEXT_TEST_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("ssi-artifacts-{label}-{}-{id}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn assert_no_parent_temp_files(dir: &Path) {
        let names: Vec<_> = fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().file_name())
            .collect();
        assert!(
            names
                .iter()
                .all(|name| !name.to_string_lossy().contains(".trusted-")),
            "parent temp file existed while worker state was being simulated: {names:?}"
        );
    }

    #[test]
    fn failure_discards_forged_score_and_result_rows() {
        let dir = test_dir("failure");
        let original = b"# header\n100\tOK\t1.000000\t1.000000\told\n";
        fs::write(dir.join(RESULTS_FILE), original).unwrap();
        fs::write(dir.join(SCORE_FILE), r#"{ "score": 1.0 }"#).unwrap();

        let artifacts = RunArtifacts::capture(&dir).unwrap();
        assert_no_parent_temp_files(&dir);

        // Simulate writes made through an absolute repository path by a worker.
        fs::write(dir.join(RESULTS_FILE), b"forged\nforged-again\n").unwrap();
        fs::write(dir.join(SCORE_FILE), r#"{ "score": -999.0 }"#).unwrap();
        assert_no_parent_temp_files(&dir);

        let final_row = "200\tFAIL\tNaN\tNaN\tauthoritative failure";
        artifacts.finish_failure(final_row).unwrap();

        assert!(!dir.join(SCORE_FILE).exists());
        let mut expected = original.to_vec();
        expected.extend_from_slice(final_row.as_bytes());
        expected.push(b'\n');
        assert_eq!(fs::read(dir.join(RESULTS_FILE)).unwrap(), expected);
        assert_no_parent_temp_files(&dir);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn success_replaces_forged_artifacts_without_changing_json() {
        let dir = test_dir("success");
        let original = b"# header\n";
        fs::write(dir.join(RESULTS_FILE), original).unwrap();
        let artifacts = RunArtifacts::capture(&dir).unwrap();

        fs::write(dir.join(RESULTS_FILE), b"forged\n").unwrap();
        fs::write(dir.join(SCORE_FILE), b"forged score").unwrap();

        let score_json = "{ \"score\": 0.900000, \"metrics\": { \"geomean_flop_ratio\": 0.900000, \"geomean_fill_ratio\": 0.950000, \"matrices\": 1, \"weights\": { \"lt_1k\": 0.30, \"1k_10k\": 0.30, \"gt_10k\": 0.40 }, \"buckets\": { \"lt_1k\": { \"count\": 1, \"geomean_flop_ratio\": 0.900000, \"geomean_fill_ratio\": 0.950000 },\"1k_10k\": { \"count\": 0, \"geomean_flop_ratio\": null, \"geomean_fill_ratio\": null },\"gt_10k\": { \"count\": 0, \"geomean_flop_ratio\": null, \"geomean_fill_ratio\": null } } } }\n";
        let final_row = "200\tOK\t0.900000\t0.950000\ttrusted";
        artifacts.finish_success(final_row, score_json).unwrap();

        assert_eq!(
            fs::read_to_string(dir.join(SCORE_FILE)).unwrap(),
            score_json
        );
        assert_eq!(
            fs::read(dir.join(RESULTS_FILE)).unwrap(),
            b"# header\n200\tOK\t0.900000\t0.950000\ttrusted\n"
        );
        assert_no_parent_temp_files(&dir);
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn unpublished_state_clears_score_during_unwind_or_early_exit() {
        let dir = test_dir("drop");
        fs::write(dir.join(RESULTS_FILE), b"# header\n").unwrap();
        let artifacts = RunArtifacts::capture(&dir).unwrap();
        fs::write(dir.join(SCORE_FILE), b"stale or worker-forged").unwrap();

        drop(artifacts);

        assert!(!dir.join(SCORE_FILE).exists());
        fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn workflow_upload_requires_successful_benchmark() {
        let workflow = include_str!("../.github/workflows/benchmark.yml");
        let upload = workflow
            .split("      - name: Upload score")
            .nth(1)
            .expect("Upload score step");

        assert!(workflow.contains("      - name: Benchmark\n        id: benchmark"));
        assert!(upload.contains("if: ${{ success() && steps.benchmark.outcome == 'success' }}"));
        assert!(!upload.contains("if: always()"));
    }
}
