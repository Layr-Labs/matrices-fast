# 0087 — Non-aggressive α-5 through the widened giant envelope

- **Date:** 2026-09-06
- **Score:** 0.826558 → 0.826558 (exact tie, all 300 ratios identical)
- **Status:** NEGATIVE locally; reverted.

## Hypothesis

0086 opened the robust envelope (350k/1.7M) but sent only the α-10 variant
through it; its probe moved faclay75, gabriel10, and both poolings. acopf,
kissing2, supplychain and friends remain tied. α-5 (moderate dense
handling) is a different order at the same AMD speed — send it through the
same open envelope while α-2/disabled stay gated (one variable).

## What changed

`src/ordering/mod.rs`: new `ROBUST_ALPHA5_MAX_NNZ = 1_700_000`; the α-5
inner gate `nnz <= 150_000` → `nnz <= ROBUST_ALPHA5_MAX_NNZ` (+ comments).

## Result

All 300 per-matrix ratios identical. α-5 beats neither AMD nor α-10 on any
giant — same verdict as the winner's own dead probe #2 (AMF α-1 on acopf:
still 1.000). The surviving tied giants resist every AMD/AMF variant tried;
consistent with 0039 (partitioners 2–4× worse there).

## Follow-ups

- Do not send α-2/α-disabled at the giants without new evidence; same null
  expected by the same mechanism.
- Giant headroom, if any, is not in AMD-variant space. Next: bucket-weighted
  relabel budgets (pending `probe_relabel_budget` data) and residual
  multi-depth prefixes K ∈ {2,4,5}.

## Links

- Research queue: [open-questions](../open-questions.md)
