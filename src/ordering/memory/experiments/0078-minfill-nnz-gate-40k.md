# 0078 — MinFill nnz gate 12k → 40k (n < 3000 held)

- **Date:** 2026-09-06
- **Score:** 0.827195 → 0.827195 (exact tie; all buckets identical)
- **Status:** NEGATIVE locally; reverted.

## Hypothesis

`probe_ties` shows exactly three tied-at-AMD matrices in the nnz band the
full-pattern MinFill family never sees (`watercontamination0303r`
1310/37028, `knp5-43` 2065/31132, `knp5-44` 2157/32604). Raising only
`MINFILL_MAX_NNZ` (n gate unchanged, membership matrix still ≤ 9 MB, still
4× below the slow tier) buys them MinFill candidacy at zero structural risk.

## What changed

`src/ordering/mod.rs`: `MINFILL_MAX_NNZ` 12_000 → 40_000 (+ comment).
Relabel-restart sub-gate untouched.

## Result

Full trusted 300-matrix run: byte-identical score, zero movers. MinFill does
not beat AMD on any of the three — same lesson as 0076/0077 from the other
side: the tied medium graphs are tied because no cheap exact heuristic
breaks them, not because the gates are misplaced.

## Follow-ups

- Do not widen MinFill gates further without a cost-model change; the family
  is exhausted on this corpus at 12k and at 40k.
- Remaining structural leads: residual-core multi-depth prefixes K ∈ {2,4,5},
  bucket-weighted relabel budgets (gt_10k leverage), RELABEL_AMF ceiling
  measurement in the 130k–400k band.

## Links

- Research queue: [open-questions](../open-questions.md)
