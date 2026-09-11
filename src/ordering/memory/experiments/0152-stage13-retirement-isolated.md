# 0152 — Retire stage 13 (alternate-seed PEO chains)

- **Date:** 2026-09-10
- **Score:** 0.792439 → **0.792452** (−0.164 relative bip; 4 rows worse, 0 better)
- **Status:** loss, reverted

## Hypothesis

Stage 13 restarts the PEO chain from orderings the portfolio built and
discarded. It is the pipeline's largest block of low-yield time and it lands on
the rows nearest the 2 s cap. Retiring it should cost little score and buy real
headroom — and it would delete `peo_alt_danger`, a skip window whose own
comment names the dev families it was drawn around.

## What changed

`src/ordering/mod.rs`: the alternate-seed chain block is deleted, along with
`PEO_ALT_LEDGER`, `PEO_ALT_MAX_LNNZ`, `PEO_ALT_MAX_N` and the `peo_alt_danger`
skip window `(3_000..8_000).contains(&n) && nnz >= 9_000`. `PEO_ALT_SEEDS`
stays: the terminal transplant still draws donors from the runner-up pool. The
test-only `probe::alt_lineage` module, which instrumented only this stage, goes
with it.

## Result

| | base | this |
|---|---:|---:|
| dev score | 0.792439 | **0.792452** |
| corpus `order()` time, 16 vCPU | 145.95 s | **138.46 s** (−5.1 %) |
| worst row | 1.315 s | **1.161 s** |
| rows over 1.0 s | 14 | **10** |

Four rows move, all worse, all small: `mpbp_15` +0.27 %,
`maxcsp-ehi-85-297-71` +0.16 %, `syn40hfsg` +0.09 %,
`kall_circlesrectangles_c6r39` +0.03 %.

**Do not read the inherited figure of "0.21 bips" as absolute-vs-relative
equivalent** — it is 0.21 *absolute* bip, which is 0.26 relative, and the
measurement here on a clean base is 0.164 relative. The two agree; the units do
not.

## Why it won / lost

Lost. The seconds stage 13 spends are not idle: the `best_perm` it leaves is
the input the three terminal local searches (`14.transplant`, `15.minl`,
`15.peel`) start from, so deleting the stage does not simply subtract its own
work — it changes what every stage after it is working on. The four
regressions are that effect, and they are the reason a stage whose own yield
looks low is not free to remove.

**Retiring the stage is therefore not the way to remove `peo_alt_danger`.** The
window has to be replaced in place by a structural predicate, with the stage
kept.

## Follow-ups

- Write stage 13's score back into `best_flops` instead of deleting the stage;
  that closes the invariant leak [0151](0151-best-flops-writeback-isolated.md)
  found without touching the basin.
- Re-derive `peo_alt_danger` as a monotone predicate on `n`/`nnz`/degree rather
  than a fitted window.

## Links

- [0151](0151-best-flops-writeback-isolated.md)
