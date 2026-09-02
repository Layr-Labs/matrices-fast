//! Local purity & license gate — a thin delegator to the shared `ssi-purity`
//! crate. The scan logic lives in ONE place, and the grader runs
//! THIS SAME harness binary (Hilbert dispatches `cargo run --release`), so the
//! gate cannot drift between local and graded runs — they are byte-identical.
//!
//! Mode is `RequireDeny`: `cargo-deny` MUST be installed and pass — a missing
//! `cargo-deny` is a hard error, not a fallback. This is load-bearing now that
//! submissions may declare third-party crates (src/ordering/deps.toml): a
//! dependency can carry a non-permissive license, so the authoritative license
//! check must actually run. The grader container ships `cargo-deny` (see
//! benchmark.yml); local contestants install the pinned grader version with
//! `cargo install cargo-deny --version 0.20.2 --locked`.
//! (The older `FallbackAllowed` mode — skip the check when `cargo-deny` is
//! absent — was sound only under the retired stdlib-only policy, where a
//! submission added no crate and thus no new license to vet.)
//!
//! HARNESS FILE — do not modify. The contract (order signature, score, gates,
//! output formats) is unchanged; only the gate's enforcement mode changed.

use std::path::Path;

pub use ssi_purity::GateError;

/// Run the local Stage-A gate from the repo root (RequireDeny mode).
pub fn check(repo_root: &Path) -> Result<(), GateError> {
    // Crates are now allowed, so the license check is load-bearing: a dependency
    // may carry a non-permissive license. RequireDeny makes cargo-deny mandatory
    // (the grader installs 0.20.2; local runs need the same pinned version).
    ssi_purity::check(repo_root, ssi_purity::Mode::RequireDeny)
}
