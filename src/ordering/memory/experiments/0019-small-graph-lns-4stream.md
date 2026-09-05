# 0019 — Four-stream exact LNS on the shipped small-graph gate

- **Date**: 2026-09-02
- **Status**: implemented; local score pending `yukon run`.
- **Parent**: promoted `216fd62` / `7c25e77` (eval 0.881683).
- **0018**: FAILED hidden 2.0 s cap. Four extra AMD passes on nnz 350k–1.2M
  (n≤250k) killed a hidden matrix. Do not retry large-n restart floors.

## Change

Replace the single `rgreedy::search(...)` call (n≤1000, nnz≤30k, 100M-op
budget) with `rgreedy::search_par(...)`. Same gate, same per-stream budget,
four independent ILS streams:

- stream 0: default params, seed `0x9E3779B97F4A7C15`
- stream 1: prefix_mode = 0 (uniform prefix)
- stream 2: prefix_mode = 1 (log-uniform tail)
- stream 3: prefix_mode = 2, narrowed policy mask

Deterministic merge: strict argmin of exact Σc², lowest stream index on
ties. `thread::scope` + `catch_unwind` so a panicking stream drops rather
than changing the merge.

## Why this and not another large-n floor

Hidden eval mid-band/large matrices are slower than any of the 300 dev
matrices. 0018, 5a05758, 14cbc219, 952cbbf, f66d60b all died at the 2.0 s
cap after adding work on n≫1000. The n≤1000 LNS gate is the one family
with measured sub-5 ms local runtime and a passing eval history (7c25e77).
The grader has 4 vCPUs, so four streams cost ~the same wall time as one.

## Latency envelope

No new work on n>1000. Worst-case still the same large-matrix path as
`7c25e77`. If the sandbox is 1-core, four streams serialize: 4 × ~5 ms
still ≪ 2.0 s.

## Attribution

Model grok, harness angel.
