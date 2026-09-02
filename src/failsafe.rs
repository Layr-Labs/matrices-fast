//! Failure-containment seams for the trusted harness path.
//!
//! The per-matrix AMD baseline and scoring run in-process in the parent. A
//! panic there (feral internal error, or an i32-overflow-sized pattern) must
//! become a recorded FAIL — not a process abort that leaks the scratch dir.
//! These are trusted harness helpers, kept in one small module so each seam is
//! unit-testable without spawning the whole binary.

use std::any::Any;
use std::path::{Path, PathBuf};

/// Run a trusted-path closure, converting a panic into `Err(message)`. Mirrors
/// the containment the contestant `order()` path already gets from its
/// subprocess+watchdog, for the in-process baseline/score path.
pub fn catch<T>(f: impl FnOnce() -> T + std::panic::UnwindSafe) -> Result<T, String> {
    std::panic::catch_unwind(f).map_err(panic_message)
}

/// Best-effort extraction of a panic payload's message.
fn panic_message(payload: Box<dyn Any + Send>) -> String {
    if let Some(s) = payload.downcast_ref::<&str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic (non-string payload)".to_string()
    }
}

/// Strip tab/CR/newline so a value is safe as a single TSV field. Every char
/// that `artifacts::rewrite_results` rejects in a row (`\t`, `\r`, `\n`) must be
/// removed here so a note (e.g. pasted with CRLF endings) cannot break artifact
/// publication.
pub fn sanitize(s: &str) -> String {
    s.replace(['\t', '\r', '\n'], " ")
}

/// Compose the `results.tsv` note column for a FAIL row: the failure reason,
/// then ` | ` + the user's note when one was given. Both are TSV-sanitized.
pub fn compose_note(reason: &str, user_note: &str) -> String {
    let reason = sanitize(reason);
    if user_note.is_empty() {
        reason
    } else {
        format!("{reason} | {}", sanitize(user_note))
    }
}

/// Owns the harness scratch dir and removes it on drop, so cleanup runs on
/// every exit path (OK, FAIL, or a stray unwind) once `main` stops calling
/// `std::process::exit`.
pub struct ScratchDir(pub PathBuf);

impl ScratchDir {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for ScratchDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Owns one staged worker-input file and unlinks it on every scope exit.
///
/// The whole scratch directory also has a drop guard, but pattern files contain
/// one matrix each and should not remain readable until the end of a corpus run.
/// Construct this guard before writing so even a partial write is cleaned up.
pub struct StagedFile(PathBuf);

impl StagedFile {
    pub fn new(path: PathBuf) -> Self {
        let _ = std::fs::remove_file(&path);
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }

    /// Unlink the staged input as soon as both workers have finished.
    ///
    /// The drop implementation remains armed, so an error here is retried while
    /// unwinding the loop iteration and by the enclosing scratch-dir guard.
    pub fn remove_now(&self) -> std::io::Result<()> {
        match std::fs::remove_file(&self.0) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl Drop for StagedFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catch_returns_ok_for_non_panicking_closure() {
        assert_eq!(catch(|| 21 * 2), Ok(42));
    }

    #[test]
    fn catch_converts_str_panic_to_err_and_survives() {
        let r = catch(|| -> i32 { panic!("boom") });
        assert_eq!(r, Err("boom".to_string()));
        // reaching here proves the process was not aborted
    }

    #[test]
    fn catch_converts_string_panic_to_err() {
        let r = catch(|| -> i32 { panic!("{}", format!("code {}", 7)) });
        assert_eq!(r, Err("code 7".to_string()));
    }

    #[test]
    fn panic_message_handles_non_string_payload() {
        let payload: Box<dyn Any + Send> = Box::new(123_u32);
        assert_eq!(panic_message(payload), "unknown panic (non-string payload)");
    }

    #[test]
    fn compose_note_without_user_note_is_reason_only() {
        assert_eq!(compose_note("matrix too large", ""), "matrix too large");
    }

    #[test]
    fn compose_note_with_user_note_joins_with_pipe() {
        assert_eq!(
            compose_note("matrix too large", "retry knob"),
            "matrix too large | retry knob"
        );
    }

    #[test]
    fn compose_note_strips_tabs_and_newlines() {
        assert_eq!(compose_note("a\tb", "c\nd"), "a b | c d");
    }

    #[test]
    fn sanitize_strips_carriage_returns() {
        // A '\r' must not survive: artifacts::rewrite_results rejects any row
        // containing '\r' or '\n'.
        let out = sanitize("line one\r\nline two");
        assert!(!out.contains('\r'));
        assert!(!out.contains('\n'));
        assert_eq!(out, "line one  line two");
    }

    #[test]
    fn compose_note_strips_carriage_returns() {
        // A note pasted with CRLF line endings must round-trip into a single
        // sanitized TSV field, not blow up artifact publication.
        let note = compose_note("matrix too large", "pasted\r\nwith crlf");
        assert!(!note.contains('\r'));
        assert!(!note.contains('\n'));
        assert_eq!(note, "matrix too large | pasted  with crlf");
    }

    #[test]
    fn scratch_dir_removed_on_drop() {
        let dir = std::env::temp_dir().join("ssi-failsafe-scratch-test");
        std::fs::create_dir_all(&dir).unwrap();
        {
            let _guard = ScratchDir(dir.clone());
            assert!(dir.exists());
        }
        assert!(!dir.exists());
    }

    #[test]
    fn scratch_dir_drop_is_noop_when_missing() {
        let dir = std::env::temp_dir().join("ssi-failsafe-scratch-missing");
        let _ = std::fs::remove_dir_all(&dir);
        // dropping a guard over a non-existent dir must not panic
        drop(ScratchDir(dir));
    }

    #[test]
    fn staged_file_removed_on_drop() {
        let path =
            std::env::temp_dir().join(format!("ssi-failsafe-staged-file-{}", std::process::id()));
        let guard = StagedFile::new(path.clone());
        std::fs::write(guard.path(), b"pattern bytes").unwrap();
        assert!(path.exists());
        drop(guard);
        assert!(!path.exists());
    }

    #[test]
    fn staged_file_can_be_removed_promptly() {
        let path = std::env::temp_dir().join(format!(
            "ssi-failsafe-prompt-staged-file-{}",
            std::process::id()
        ));
        let guard = StagedFile::new(path.clone());
        std::fs::write(guard.path(), b"pattern bytes").unwrap();
        guard.remove_now().unwrap();
        assert!(!path.exists());
        // Drop is intentionally still safe after prompt removal.
        drop(guard);
    }
}
