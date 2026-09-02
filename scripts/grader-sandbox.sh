#!/usr/bin/env bash
# Bubblewrap build boundary. Trusted parent and untrusted candidate are built
# as separate packages into disjoint target directories.
set -euo pipefail

MODE="${1:-}"
if [[ "$MODE" != "trusted-build" && "$MODE" != "candidate-build" && "$MODE" != "self-check" ]]; then
  echo "usage: $0 {trusted-build|candidate-build|self-check}" >&2
  exit 2
fi
if [[ "$(uname -s)" != "Linux" ]]; then
  echo "grader-sandbox: bubblewrap grader mode requires Linux" >&2
  exit 1
fi
if [[ "$(id -u)" -eq 0 ]]; then
  echo "grader-sandbox: trusted parent must not run as root" >&2
  exit 1
fi
if ! command -v bwrap >/dev/null 2>&1; then
  echo "grader-sandbox: bubblewrap is required" >&2
  exit 1
fi
# Every order() worker is launched as prlimit -> bwrap -> candidate, so grader
# mode cannot apply its per-worker rlimits without prlimit. Fail here, where the
# message is visible, rather than per worker with stderr on /dev/null.
if ! command -v prlimit >/dev/null 2>&1; then
  echo "grader-sandbox: prlimit (util-linux) is required for per-worker resource limits" >&2
  exit 1
fi

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_ROOT="$ROOT/target"
SYSROOT="$(rustc --print sysroot)"
case "$MODE" in
  trusted-build)
    PACKAGE="matrices-fast"
    TARGET="$TARGET_ROOT/trusted"
    ;;
  candidate-build)
    PACKAGE="ssi-candidate-worker"
    TARGET="$TARGET_ROOT/candidate"
    ;;
  self-check)
    PACKAGE=""
    TARGET="$TARGET_ROOT/sandbox-self-check"
    ;;
esac
rm -rf "$TARGET"
mkdir -p "$TARGET"

for tool in "$SYSROOT/bin/cargo" "$SYSROOT/bin/rustc"; do
  if [[ ! -x "$tool" ]]; then
    echo "grader-sandbox: required tool is absent: $tool" >&2
    exit 1
  fi
done

# Start from an empty root. The checkout is read-only. A tmpfs hides the host
# target tree, then exposes only this mode's clean target directory, preventing
# candidate build scripts from replacing or reading trusted build artifacts.
BWRAP=(
  bwrap
  --unshare-pid
  --unshare-net
  --unshare-ipc
  --unshare-uts
  --unshare-cgroup-try
  --new-session
  --die-with-parent
  --cap-drop ALL
  --clearenv
  --tmpfs /
  --proc /proc
  --dev /dev
  --tmpfs /tmp
  --ro-bind /usr /usr
  # /etc is needed read-only for the system linker: `cc` is /usr/bin/cc ->
  # /etc/alternatives/cc, and ld resolves /etc/ld.so.cache + /etc/ld.so.conf.d.
  # Without it rustc fails with `linker cc not found` (ENOENT on the dangling
  # alternatives symlink). No runner secrets live here; HOME/credentials stay
  # hidden and this bind is read-only.
  --ro-bind /etc /etc
)
for path in /bin /sbin /lib /lib64; do
  if [[ -e "$path" ]]; then
    BWRAP+=(--ro-bind "$path" "$path")
  fi
done
BWRAP+=(
  --ro-bind "$SYSROOT" /toolchain
  --ro-bind "$ROOT" /workspace
  --tmpfs /workspace/target
  --dir /workspace/target/build
  --bind "$TARGET" /workspace/target/build
  --dir /tmp/home
  --dir /tmp/cargo
  --setenv PATH /toolchain/bin:/usr/bin:/bin
  --setenv HOME /tmp/home
  --setenv CARGO_HOME /tmp/cargo
  --setenv CARGO_TARGET_DIR /workspace/target/build
  --setenv CARGO_NET_OFFLINE true
  --setenv RUSTC /toolchain/bin/rustc
  --chdir /workspace
)

if [[ "$MODE" == "self-check" ]]; then
  "${BWRAP[@]}" -- /bin/bash -ceu '
    [[ "$(id -u)" -ne 0 ]]
    [[ ! -e "${RUNNER_TEMP:-/__unset_runner_temp}" ]]
    [[ ! -e /root ]]
    if touch /workspace/.sandbox-write-probe 2>/dev/null; then
      echo "grader-sandbox: checkout unexpectedly writable" >&2
      rm -f /workspace/.sandbox-write-probe
      exit 1
    fi
    touch /workspace/target/build/.sandbox-write-probe
    rm /workspace/target/build/.sandbox-write-probe
    touch /tmp/write-probe
    SSI_REQUIRE_NETWORK_BLOCK=1 /workspace/.github/scripts/assert-no-network.sh
  '
  echo "grader-sandbox: build boundary self-check passed"
  exit 0
fi

exec "${BWRAP[@]}" -- /toolchain/bin/cargo build \
  --release --offline --locked --package "$PACKAGE"
