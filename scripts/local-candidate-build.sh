#!/usr/bin/env bash
# Sandbox the LOCAL build of the untrusted candidate worker.
#
# The candidate package is the only one that compiles `src/ordering/` and the
# contestant's declared dependencies. A pushed-back winning submission is
# untrusted code; the execute phase is already sandboxed per worker
# by the trusted parent, and this script gives the *build* phase the same
# containment locally — no network, and writes confined to the build/cache
# directories. It builds `ssi-candidate-worker` into the normal `target/release`
# so the usual `cargo run --release` finds it as a sibling.
#
# Platforms:
#   - Linux: reuse the audited bubblewrap build boundary
#     (`scripts/grader-sandbox.sh candidate-build`), then stage the artifact.
#   - macOS: run cargo under `sandbox-exec` (Seatbelt) with network denied.
#
# Escape hatch: SSI_ALLOW_UNSANDBOXED_WORKER=1 builds directly (loud warning),
# mirroring the execute-phase opt-out.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd -P)"
cd "$ROOT"

PACKAGE="ssi-candidate-worker"
BUILD=(cargo build --release -p "$PACKAGE" --offline --locked)

if [[ "${SSI_ALLOW_UNSANDBOXED_WORKER:-}" == "1" ]]; then
  echo "WARNING: SSI_ALLOW_UNSANDBOXED_WORKER=1 — building the untrusted candidate" \
       "worker WITHOUT a sandbox." >&2
  exec "${BUILD[@]}"
fi

OS="$(uname -s)"
case "$OS" in
  Darwin)
    SANDBOX_EXEC="/usr/bin/sandbox-exec"
    if [[ ! -x "$SANDBOX_EXEC" ]]; then
      echo "local-candidate-build: $SANDBOX_EXEC not found; cannot sandbox the build." \
           "Set SSI_ALLOW_UNSANDBOXED_WORKER=1 to build unsandboxed at your own risk." >&2
      exit 1
    fi
    # Canonicalize the writable roots: Seatbelt matches resolved paths, and
    # macOS temp/cargo dirs route through /private and symlinks.
    canon() { (cd "$1" 2>/dev/null && pwd -P); }
    TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
    mkdir -p "$TARGET_DIR"
    CARGO_DIR="${CARGO_HOME:-$HOME/.cargo}"
    TMP_DIR="${TMPDIR:-/tmp}"
    TARGET_C="$(canon "$TARGET_DIR")"
    CARGO_C="$(canon "$CARGO_DIR")"
    TMP_C="$(canon "$TMP_DIR")"
    # deny everything, then re-allow what a Rust build needs: exec/fork, broad
    # reads (toolchain, sources, vendored registry), sysctl/mach for the loader.
    # Deny ALL network. Permit writes ONLY under the build/cache/temp roots.
    PROFILE="(version 1)
(deny default)
(allow process-fork)
(allow process-exec)
(allow sysctl-read)
(allow sysctl-write)
(allow mach-lookup)
(allow file-read*)
(allow file-ioctl)
(deny network*)
(allow file-write-data (literal \"/dev/null\"))
(allow file-write* (subpath \"$TARGET_C\"))
(allow file-write* (subpath \"$CARGO_C\"))
(allow file-write* (subpath \"$TMP_C\"))"
    echo "local-candidate-build: building $PACKAGE under sandbox-exec (network denied)" >&2
    exec "$SANDBOX_EXEC" -p "$PROFILE" "${BUILD[@]}"
    ;;
  Linux)
    if ! command -v bwrap >/dev/null 2>&1; then
      echo "local-candidate-build: bubblewrap (bwrap) not found; cannot sandbox the build." \
           "Install it (e.g. apt-get install bubblewrap), or set" \
           "SSI_ALLOW_UNSANDBOXED_WORKER=1 to build unsandboxed at your own risk." >&2
      exit 1
    fi
    # Reuse the audited build boundary; it builds into target/candidate.
    echo "local-candidate-build: building $PACKAGE under bubblewrap (network denied)" >&2
    bash "$ROOT/scripts/grader-sandbox.sh" candidate-build
    # Stage the artifact where the local parent discovers it as a sibling.
    SRC="$ROOT/target/candidate/release/$PACKAGE"
    DST_DIR="$ROOT/target/release"
    mkdir -p "$DST_DIR"
    cp -f "$SRC" "$DST_DIR/$PACKAGE"
    echo "local-candidate-build: staged $DST_DIR/$PACKAGE" >&2
    ;;
  *)
    echo "local-candidate-build: unsupported platform $OS; no built-in build sandbox." \
         "Set SSI_ALLOW_UNSANDBOXED_WORKER=1 to build unsandboxed at your own risk." >&2
    exit 1
    ;;
esac
