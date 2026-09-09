# 0202 — PAIR_DESCENT_EXT_OPS_BUDGET 48M→64M

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834).
- **Score:** 0.793834 → **0.793831** (−0.000003)
- **Status:** **CLEARED / REVERTED** under CoS priority reset (ops-budget family banned). Not submitted. Tree restored: `PAIR_DESCENT_EXT_OPS_BUDGET=48M`, `FINAL_FIVE_OPS=128M`.

## Hypothesis

Modest +16M EXT pair-descent budget might improve EXT-class rows without gate/SWEEPS widen.

## What changed (then reverted)

`PAIR_DESCENT_EXT_OPS_BUDGET` 48_000_000→64_000_000 only. Gates/SWEEPS/nnz/FINAL_FIVE untouched.

## Result

Yukon ran (`/tmp/yukon-run-0202.log`): **0.793831**, fill 0.9258.
Flops vs tip: **6 better / 2 worse** (all in ~pair-ext class):
- better: crudeoil_lee2_06 −557, chp_shorttermplan1a −214, mpbp_35 −112, powerflow0300p −112, crudeoil_li05 −29, transswitch0300p −22
- worse: crudeoil_lee4_06 +295, rsyn0840m04m +131

Priority reset: revert regardless of score; do not submit ops-budget bumps.

## Why cleared

CoS priority reset: ops-budget family banned; parent will dispatch next (488a-class structural). Reverted to tip constants.

## Follow-ups

None from Order — tip left clean; await parent dispatch.
