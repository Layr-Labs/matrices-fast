# 0152 — Retire stage 13 (alternate-seed PEO chains), and nothing else

- **Date:** 2026-09-10
- **Score:** 0.792439 → **0.792452** (−0.164 relative bip; 4 rows worse, 0 better)
- **Status:** work removal with a small score cost and large cap headroom;
  submitted as the third and last arm of a cap bisect

## Hypothesis

Stage 13 restarts the PEO chain from orderings the portfolio built and
discarded. It is the pipeline's largest block of low-yield time and it lands on
the rows nearest the 2 s cap. Retiring it should cost little score and buy real
headroom — and it deletes the last identity-keyed predicate on that path.

## What changed

`src/ordering/mod.rs`: the alternate-seed chain block is deleted, along with
`PEO_ALT_LEDGER`, `PEO_ALT_MAX_LNNZ`, `PEO_ALT_MAX_N` and the `peo_alt_danger`
skip window `(3_000..8_000).contains(&n) && nnz >= 9_000`, whose own comment
named the dev families (`lee1_07`, `mpbp_15`, `chimera`) it was drawn around.
`PEO_ALT_SEEDS` stays: the terminal transplant still draws donors from the
runner-up pool. The test-only `probe::alt_lineage` module, which instrumented
only this stage, goes with it.

## Result

| | arm W (base) | this |
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

## Why it is submitted alone

Two trees of mine died on the 2 s cap while the tree they descend from passes.
Three changes were shared. The stage-1b window removal was submitted alone and
**promoted**; the `best_flops` repair was submitted alone and **cleared the cap**
(graded 0.842835, rejected in the dead band). This is the third. If it clears
the cap, then no single one of the three causes the timeout and the failure is
an *interaction* — which is a much more uncomfortable result than a culprit.

## Follow-ups

- Whatever this returns, the 7.5 s of corpus-wide work it frees is the
  cheapest funding available for a new mechanism, at a cost of 0.164 dev bip.

## Links

- [0150](0150-stage1b-window-removal-isolated.md), [0151](0151-best-flops-writeback-isolated.md)
