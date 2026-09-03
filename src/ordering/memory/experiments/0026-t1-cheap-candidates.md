# 0026 — T1 cheap candidates: attribute, then add ONE density-gated variant

- **Date:** 2026-09-03
- **Parent:** `971649b` (promoted hidden score 0.876273; dev 0.850594)
- **Status:** revised and resubmitted after a hidden failure (see below)

## First attempt (FAILED hidden validation)

Wired `DegP125` + `Ammf` into the custom-metrics gate (+4 AMF-class passes),
added an ND-leaf AMD hybrid, and fixed three missing `best_flops` updates.
Submitted as `5827d321` without a local benchmark run. It **failed** hidden
validation with no score.

## Failure analysis

A local `yukon run` of the exact submitted tree PASSED at 0.850546 — only
−0.56 bips under the parent, below the 1-bip bar anyway — so the hidden
failure is a timeout, not scoring: doubling quotient passes (4→8) on the wide
`nnz ≤ 300k && nnz ≥ 10n` gate repeats the 0021/0025 additive-work failure
mode (a hidden dense medium can cost far more per pass than any dev matrix).
Reverting to `[SqDiv, SqPure]` and re-running scored a byte-identical
0.850546, proving the +4 passes bought literally ZERO on dev: pure risk, no
reward. Revert confirmed correct.

## Attribution (`probe_metric_variants`, new ignored probe in `probe.rs`)

Scored the 7 remaining unwired variants × α{1.0,10.0} against the LIVE
portfolio incumbent on all 35 gated matrices (a variant only matters if it
beats best-of, not AMD alone):

- 8 of 9 win nothing anywhere (DegP125/Ammf independently confirmed at 0 by
  the two byte-identical full runs).
- Exactly one win: **DegPlusDegme** on `gams05`, 0.792→0.6905, max 23 ms per
  pass. Upper bound with everything added: 0.849576 (−11 bips).

## Shipped design

- Custom block keeps the 4 proven passes under their gate; DegPlusDegme ×
  {1.0, 10.0} rides under an extra **density gate `nnz <= 20n`** (`gams05`
  sits at ~14.6x). Per-pass quotient cost explodes with density, so only the
  NEW work is density-capped while proven work is untouched. Worst-case added
  time ≈ 45 ms.
- ND-leaf AMD hybrid (`amd_leaf_order` + both `deg_fill` closures, degree-sort
  fallback, `catch_unwind`-guarded) and the round 2/3/5 `best_flops` hygiene
  stay: microsecond-scale, zero gate changes.

## Result

Full 300-matrix `yukon run`: **0.849564** (fill 0.9469; buckets
0.8963 / 0.8761 / **0.7946**), i.e. −12 bips vs the 0.850594 parent —
12x the 1-bip bar. `probe_timing_and_score` worst `order()`: **0.774 s**
(`gams05`), inside the 0.77–0.83 s envelope of revisions known to have passed
hidden validation on this box.

## Lesson

Attribute before wiring: a full `yukon run` with and without the +4 passes
scored byte-identical, which a 59-second probe then explained (8 of 9 variants
win nothing). Never double a quotient-pass family again without per-variant
attribution, and density-gate any new quotient work — the wide `nnz ≥ 10n`
gate admits arbitrarily dense hidden matrices. Single-matrix wins (≈ all of
this −12 bips is `gams05`) are fragile per [0004](0004-structured-relabelings.md);
hidden translation is the verdict.
