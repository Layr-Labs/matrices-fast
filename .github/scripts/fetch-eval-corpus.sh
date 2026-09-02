#!/usr/bin/env bash
#
# Fetch today's hidden EVAL corpus from the private bucket to the path given as
# $1 (a temp path OUTSIDE the repo tree, so the eval bytes are never committed).
# Used by .github/workflows/benchmark.yml.
#
# Bucket layout:
#   eval/current.txt                       -> one line: today's prefix, e.g. "2026-06-24"
#   eval/<prefix>/patterns.jsonl           -> the day's corpus
#   eval/<prefix>/patterns.jsonl.sha256    -> (optional) checksum, "<hex>  patterns.jsonl"
#
# The pointer indirection lets the daily rotation job upload a new dated object
# and then flip current.txt atomically; keeping N dated prefixes gives free
# history for audit/repro.
#
# S3-compatible: the same `aws s3` CLI with --endpoint-url drives Cloudflare R2
# (plan's preferred store, no egress fees), AWS S3, and GCS's S3 interop. The
# credential is scoped READ-ONLY to this one bucket (least privilege); a leaked
# Actions secret can neither write, delete, nor reach anything else.
#
# Required environment (set from Actions secrets by the workflow):
#   EVAL_BUCKET_NAME       bucket name
#   AWS_ACCESS_KEY_ID      read-only key id
#   AWS_SECRET_ACCESS_KEY  read-only secret
# Optional:
#   EVAL_BUCKET_ENDPOINT   S3-compatible endpoint URL (required for R2/GCS; omit for AWS S3)
#   AWS_DEFAULT_REGION     region (R2 uses "auto")

set -euo pipefail

dest="${1:?usage: fetch-eval-corpus.sh <dest-path>}"

if [ -z "${EVAL_BUCKET_NAME:-}" ]; then
  echo "fetch-eval-corpus: EVAL_BUCKET_NAME is not set" >&2
  exit 1
fi

# --endpoint-url is what routes the S3 CLI at R2/GCS instead of AWS. Omitting it
# (empty endpoint) leaves the CLI pointed at AWS S3.
endpoint_args=()
if [ -n "${EVAL_BUCKET_ENDPOINT:-}" ]; then
  endpoint_args=(--endpoint-url "$EVAL_BUCKET_ENDPOINT")
fi

s3() { aws "${endpoint_args[@]}" s3 "$@"; }
s3api() { aws "${endpoint_args[@]}" s3api "$@"; }

workdir="$(mktemp -d)"
trap 'rm -rf "$workdir"' EXIT

# 1. Read the pointer to today's prefix.
s3 cp "s3://${EVAL_BUCKET_NAME}/eval/current.txt" "$workdir/current.txt" >/dev/null
prefix="$(tr -d ' \t\r\n' < "$workdir/current.txt")"
if [ -z "$prefix" ]; then
  echo "fetch-eval-corpus: eval/current.txt is empty" >&2
  exit 1
fi
# Do NOT echo the resolved prefix/object key: this runs in benchmark.yml, whose
# Actions log is public. The dated key names a specific eval corpus (invariant:
# never reference any eval file). The bucket name is a masked secret, but the
# date prefix is not, so printing it here would leak the rotation date. Operators
# who need the prefix can read eval/current.txt from the bucket directly.
echo "fetch-eval-corpus: pointer resolved"

# 2. Download the referenced corpus to the destination.
s3 cp "s3://${EVAL_BUCKET_NAME}/eval/${prefix}/patterns.jsonl" "$dest" >/dev/null

# 3. Verify integrity if a checksum object is published, so a half-uploaded or
#    truncated corpus fails loudly instead of scoring against a partial file.
if s3api head-object --bucket "$EVAL_BUCKET_NAME" \
      --key "eval/${prefix}/patterns.jsonl.sha256" >/dev/null 2>&1; then
  s3 cp "s3://${EVAL_BUCKET_NAME}/eval/${prefix}/patterns.jsonl.sha256" \
      "$workdir/expected.sha256" >/dev/null
  expected="$(awk '{print $1}' "$workdir/expected.sha256")"
  if command -v sha256sum >/dev/null 2>&1; then
    actual="$(sha256sum "$dest" | awk '{print $1}')"
  else
    actual="$(shasum -a 256 "$dest" | awk '{print $1}')"
  fi
  if [ "$expected" != "$actual" ]; then
    echo "fetch-eval-corpus: checksum mismatch (expected $expected, got $actual)" >&2
    exit 1
  fi
  echo "fetch-eval-corpus: checksum OK"
else
  echo "fetch-eval-corpus: no checksum object published; skipping integrity check"
fi

echo "fetch-eval-corpus: corpus downloaded and verified"
