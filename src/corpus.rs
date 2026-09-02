//! The development corpus loader.
//!
//! HARNESS FILE — do not modify the CONTRACT. The contestant ordering sees
//! `crate::Pattern` (re-exported from `ssi_scoring::Pattern` at the crate root,
//! in `main.rs`), so the contract signature
//! `order(pattern: &Pattern) -> Vec<usize>` is byte-identical to what the grader
//! scores. That type carries structure only — never values, never a right-hand
//! side (NARROW INPUT).
//!
//! ## One reader
//!
//! The corpus is a single `patterns.jsonl` produced by the `corpus-generation`
//! pipeline: one JSON object per line, each a full symmetrized CSC sparsity
//! pattern. Both this harness and the grader parse a line into a `Pattern`
//! through the SAME function, `ssi_scoring::pattern_from_jsonl_line` — there is
//! no second parser that can silently disagree, so a contestant's local
//! `Pattern` is identical to the graded `Pattern` at the parsing boundary too.

use std::path::PathBuf;

use crate::Pattern;

/// The shipped public development corpus: one JSONL file of CSC patterns.
pub const DEV_CORPUS_FILE: &str = "corpus/dev/patterns.jsonl";

/// Optional environment override for the corpus path. UNSET in every contestant
/// run, so local behavior is unchanged: the harness grades the public dev corpus
/// (`DEV_CORPUS_FILE`). The grading workflow sets this to the downloaded EVAL
/// corpus at a temp path *outside* the repo tree, so the eval bytes are never in
/// a tracked file and can never be committed. This is the corpus-path seam:
/// additive and default-preserving — it does not touch the contract
/// (signature/score/gates/output formats are unchanged).
pub const CORPUS_FILE_ENV: &str = "SSI_CORPUS_FILE";

/// Resolve the corpus path from a (possibly absent) env-var value, defaulting to
/// the public dev corpus when unset or blank. Split out from the env read so it
/// is testable without mutating process-global state.
fn resolve_corpus_path_from(env_value: Option<String>) -> PathBuf {
    match env_value {
        Some(v) if !v.trim().is_empty() => PathBuf::from(v),
        _ => PathBuf::from(DEV_CORPUS_FILE),
    }
}

/// The corpus path this run grades: `$SSI_CORPUS_FILE` if set and non-blank,
/// else the public dev corpus.
pub fn corpus_path() -> PathBuf {
    resolve_corpus_path_from(std::env::var(CORPUS_FILE_ENV).ok())
}

/// A Git LFS pointer file (what a clone without `git-lfs` leaves at a tracked
/// path) begins with this line. Detecting it lets the harness print an
/// actionable message instead of an opaque JSON parse error.
fn is_lfs_pointer(text: &str) -> bool {
    text.trim_start()
        .starts_with("version https://git-lfs.github.com/spec/")
}

/// Read at most the first ~256 bytes of `path` (a Git LFS pointer is well under
/// that). Returns an empty string if the file is absent/unreadable, so the
/// caller treats "no file" as "not a pointer" and falls through to the normal
/// load path. Avoids slurping a ~99 MB corpus just to check its first line.
fn read_prefix(path: &std::path::Path) -> String {
    use std::io::Read as _;
    let Ok(mut f) = std::fs::File::open(path) else {
        return String::new();
    };
    let mut buf = [0u8; 256];
    let n = f.read(&mut buf).unwrap_or(0);
    String::from_utf8_lossy(&buf[..n]).into_owned()
}

/// Load `path` as `(source, Pattern)` pairs through the shared JSONL reader.
///
/// Returns an empty vec when the file is absent. Parse/LFS errors are returned
/// so the harness can redact details for a hidden corpus before writing logs.
pub fn load(path: &std::path::Path) -> Result<Vec<(String, Pattern)>, String> {
    // If the corpus is an unresolved Git LFS pointer (clone without git-lfs),
    // fail loudly with a fix, not a confusing parse error. Only the first line
    // matters (the pointer's `version` marker), so read a small prefix rather
    // than slurping the whole corpus (~99 MB) just to check it.
    if is_lfs_pointer(&read_prefix(path)) {
        return Err(format!(
            "{} is an unresolved Git LFS pointer, not the corpus.\n\
             Install git-lfs and fetch the real file:\n\
             \x20   git lfs install && git lfs pull",
            path.display()
        ));
    }
    match ssi_scoring::load_corpus_jsonl(path) {
        Ok(c) => Ok(c),
        Err(e) => {
            if path.exists() {
                Err(format!("failed to load {}: {e}", path.display()))
            } else {
                Ok(Vec::new())
            }
        }
    }
}

/// Load the resolved corpus with fail-loud behavior, for tests.
///
/// The main harness uses [`load`] directly so hidden-corpus errors can be
/// redacted. Developer probes live in the candidate-worker package, which
/// carries its own test-only corpus shim.
#[cfg(test)]
pub fn corpus() -> Vec<(String, Pattern)> {
    let path = corpus_path();
    load(&path).unwrap_or_else(|e| panic!("{e}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn corpus_path_defaults_to_dev_when_env_unset() {
        // Contestants never set SSI_CORPUS_FILE; the default must be the public
        // dev corpus so local runs are unchanged (contract-invisible seam).
        assert_eq!(
            resolve_corpus_path_from(None),
            PathBuf::from(DEV_CORPUS_FILE)
        );
    }

    #[test]
    fn corpus_path_honors_env_override() {
        // The grader workflow sets SSI_CORPUS_FILE to the downloaded eval corpus
        // at a temp path outside the repo tree, so eval bytes never sit at a
        // tracked path.
        let eval = "/tmp/eval.jsonl";
        assert_eq!(
            resolve_corpus_path_from(Some(eval.to_string())),
            PathBuf::from(eval)
        );
    }

    #[test]
    fn corpus_path_treats_blank_env_as_unset() {
        // A defined-but-empty env var must not silently point the harness at "".
        assert_eq!(
            resolve_corpus_path_from(Some(String::new())),
            PathBuf::from(DEV_CORPUS_FILE)
        );
        assert_eq!(
            resolve_corpus_path_from(Some("   ".to_string())),
            PathBuf::from(DEV_CORPUS_FILE)
        );
    }

    #[test]
    fn corpus_matches_shared_jsonl_reader() {
        // The harness corpus loader must yield exactly what the shared
        // ssi_scoring reader parses, in the same order — one parser.
        let path = std::path::Path::new(DEV_CORPUS_FILE);
        let shared = ssi_scoring::load_corpus_jsonl(path).expect("load dev corpus");
        let ours = corpus();
        assert_eq!(ours.len(), shared.len());
        for ((src_a, pat_a), (src_b, pat_b)) in ours.iter().zip(shared.iter()) {
            assert_eq!(src_a, src_b);
            assert_eq!(pat_a.n, pat_b.n);
            assert_eq!(pat_a.col_ptr, pat_b.col_ptr);
            assert_eq!(pat_a.row_idx, pat_b.row_idx);
        }
    }

    #[test]
    fn load_returns_parse_error_instead_of_panicking() {
        let dir = std::env::temp_dir().join("ssi-corpus-load-error-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("malformed.jsonl");
        std::fs::write(&path, r#"{"source":"SECRET_NAME","n":"bad"}"#).unwrap();
        let error = load(&path).unwrap_err();
        assert!(error.contains("SECRET_NAME"));
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn detects_git_lfs_pointer_text() {
        let pointer = "version https://git-lfs.github.com/spec/v1\n\
                       oid sha256:abc123\n\
                       size 103879806\n";
        assert!(is_lfs_pointer(pointer));
        // A real JSONL corpus line is not a pointer.
        assert!(!is_lfs_pointer(r#"{"n":4,"indptr":[0],"indices":[]}"#));
        assert!(!is_lfs_pointer(""));
    }
}
