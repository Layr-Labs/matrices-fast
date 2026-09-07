# 0092 — Transplant admits ties (below-anchor gate → strict ties)

- **Date:** 2026-09-07
- **Score:** 0.823737 → 0.823737 (exact tie; results.tsv row kept)
- **Status:** NEGATIVE locally; reverted.

## Hypothesis

The transplant pass runs only on below-anchor rows (`inc_f >= amd_flops`
refuses). The 54+17+7 tied rows are exactly the population nothing moves;
strict-accept means a tie can only improve, and the pass is ledger-bounded
(250k units ≈ ms per row), so admitting ties is structurally safe.

## What changed

`src/ordering/transplant_probe.rs`: `inc_f >= amd_flops` → `inc_f >
amd_flops` (+ comment). One operator.

## Result

Full trusted 300-matrix run: byte-identical score. Transplant cannot beat
AMD on ties either — same verdict as every family tried on ties (0076/0077/
0078/0087): tied rows are tied because no cheap exact heuristic breaks them.

## Follow-ups

- Do not re-admit ties to any best-of pass without a new mechanism; the
  population is now proven barren across MinFill, extra streams, extra
  lotteries, wider gates, and transplant.
- Open: width cascade {4096,512,128,32} — intermediate widths untested
  (needs reservation-policy study first); residual multi-depth K ∈ {2,4,5}.

## Links

- Research queue: [open-questions](../open-questions.md)
