# 0093 — Transplant width cascade +1024/+256 (appended, leftover-only)

- **Date:** 2026-09-07
- **Score:** 0.823737 → 0.823733 (−0.05 bip; 1 mover, 0 regressions)
- **Status:** NEGATIVE for shipping (below the 1-bip bar by 20×); reverted.

## Hypothesis

The transplant width cascade {4096,512,128,32} jumps 8x/4x over medium
granularities. Appending 1024+256 at the END spends only leftover ledger
after the shipped widths run identically — strictly additive by
construction, strict-accept per width, ledger-capped overall.

## What changed

`src/ordering/transplant_probe.rs` production `transplant_pass` cascade
+1024, +256 appended (+ comment). Test-only `terminal_pass` untouched.

## Result

One mover (`multiplants_stg5` 0.428 → 0.427), net −0.05 bip. Per 0004
doctrine this is single-matrix luck range, and even if fully real it
converts to nothing against a 1-bip bar. Not submitted.

## Follow-ups

- Intermediate widths are exhausted as a direction (this + the shipped set
  cover the granularity axis at fixed ledger).
- Remaining structural leads: residual multi-depth prefixes K ∈ {2,4,5},
  bucket-weighted relabel budgets (AMD restarts saturated per budget probe —
  needs the AMF-family version of the argument).

## Links

- Research queue: [open-questions](../open-questions.md)
