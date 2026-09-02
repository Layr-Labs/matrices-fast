# Development corpus

`patterns.jsonl` here is the **full development corpus** (300 patterns, n up to
~340,000), one CSC sparsity pattern per line. It spans the families NLP / QCP /
QP / QCQP and populates all three scoring size buckets (lt_1k / 1k_10k / gt_10k).

It is shipped in-repo via **Git LFS** because the file is ~99 MB. Install Git LFS
before cloning, or fetch it afterward:

```sh
git lfs install
git lfs pull   # if you cloned before installing git-lfs
```

Without `git-lfs`, this path holds a small text *pointer* instead of JSONL, and
the harness stops with a message telling you to run `git lfs pull`.

## Line format

Each line is one symmetric sparsity pattern as compressed-sparse-column (CSC):

```json
{"n": 4, "nnz": 12, "indptr": [...], "indices": [...], "hash": "...", "source": "st_e09"}
```

- `n` — matrix dimension. `indptr` (len n+1) / `indices` (len nnz) — CSC columns.
- The stored pattern is the **full symmetrized** pattern and **includes the
  diagonal**; the harness reader (`ssi_scoring::pattern_from_jsonl_line`) drops
  the diagonal to produce the off-diagonal contract `Pattern`.
- `hash` (SHA-256 of the canonical pattern) and `source` (origin problem) are
  metadata; the harness uses `source` as the display name.

## Grading a different corpus

Point the harness at another corpus for one run with `SSI_CORPUS_FILE`:

```sh
SSI_CORPUS_FILE=/path/to/other.jsonl cargo run --release
```

Unset, the harness grades this in-repo corpus. The competition's hidden
evaluation corpus is never published.
