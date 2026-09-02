#!/usr/bin/env bash
# Regenerate the trusted parent and candidate-worker manifests. Contestant
# dependencies are written only to candidate-worker/Cargo.toml.
# Runs in BOTH the local harness and the grader, so both build identically.
# Exit non-zero on any validation failure. Only reviewed exact direct dependencies are emitted;
# the full transitive-tree license/native scan runs after `cargo vendor`.
set -euo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

DEPS_TOML="src/ordering/deps.toml"
TEMPLATE="Cargo.toml.in"
OUT="Cargo.toml"
CANDIDATE_TEMPLATE="candidate-worker/Cargo.toml.in"
CANDIDATE_OUT="candidate-worker/Cargo.toml"

[ -f "$TEMPLATE" ] || { echo "prepare-build: missing $TEMPLATE" >&2; exit 2; }
[ -f "$CANDIDATE_TEMPLATE" ] || {
  echo "prepare-build: missing $CANDIDATE_TEMPLATE" >&2
  exit 2
}

# Write the candidate manifest up to its generated-dependency marker, then add
# the validated dependency declarations. The trusted parent manifest is copied
# byte-for-byte and never receives contestant-controlled dependency entries.
write_candidate_manifest() {
  local gen="$1"
  awk '1; /=== GENERATED CANDIDATE DEPS BELOW/ {exit}' \
    "$CANDIDATE_TEMPLATE" > "$CANDIDATE_OUT"
  if [ -n "$gen" ]; then
    while IFS='=' read -r name version; do
      [ -z "$name" ] && continue
      printf '%s = "%s"\n' "$name" "$version" >> "$CANDIDATE_OUT"
    done <<< "$gen"
  fi
}

# Both manifests are generated and git-ignored. Bootstrap the candidate with no
# declared dependencies so Cargo can load the workspace before emit-deps runs.
cp "$TEMPLATE" "$OUT"
write_candidate_manifest ""

# Validate deps.toml against ssi-purity's trusted exact-version allowlist (the
# ONE parser/policy gate). Emits one `name=version` line per accepted dependency
# to stdout, or exits non-zero. This completes before any vendor or build step.
GEN="$(cargo run --quiet -p ssi-purity --bin emit-deps -- "$DEPS_TOML")" || {
  echo "prepare-build: deps.toml rejected (see error above)" >&2
  exit 1
}

# Rewrite only the untrusted candidate package with validated declared deps.
write_candidate_manifest "$GEN"
echo "prepare-build: wrote $OUT and $CANDIDATE_OUT"

# Vendor the full transitive tree. The COMMITTED Cargo.lock is authoritative:
# `cargo vendor` honors its existing pins while resolving an approved direct
# dependency not yet represented in the baseline lock. The trusted parser above
# enforces each direct name/version pair exactly. This step needs network to
# fetch crate archives; the later build is offline and locked.
mkdir -p .cargo
CARGO_NET_OFFLINE=false cargo vendor vendor > .cargo/vendor-source.toml
cat .cargo/config.base.toml .cargo/vendor-source.toml > .cargo/config.toml

# Scan the vendored tree for native/FFI escapes BEFORE any build.
cargo run --quiet -p ssi-purity --bin scan-tree -- vendor || {
  echo "prepare-build: vendored dependency tree failed the FFI/native scan" >&2
  exit 1
}
echo "prepare-build: vendored tree scanned clean"
