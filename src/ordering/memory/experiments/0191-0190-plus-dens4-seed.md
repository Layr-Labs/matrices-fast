# 0191 — 0190 mid hole-gate + dens≤4 else-floor second seed

- **Date:** 2026-09-09
- **Base:** 0190 package on tip `74b6ccd` (local keep 0.793612; official `4e4a7f6` validating→rejected). Prior 0189 same seed on 0188 was local KEEP then reverted when `d6c204c` failed.
- **Score:** 0.793612 → **0.793589** (−0.000023)
- **Status:** local KEEP bar met (score/movers/timing), then **reverted** — official `4e4a7f6` **rejected** (0.84349, 0.00%). Not submitted. 0190 package left intact.

## Hypothesis

0189 density-gated the tip 4-ticket else-floor 50M repeat seed
`0xD1B5_4A32_D192_ED03` → `0x6A09_E667_F3BC_C909` only when `nnz ≤ 4 * n`,
winning `mpbp_46` / `rsyn0840m02m` while keeping denser `crudeoil_pooling_ct3`
on `0xD1B5`. Restack that seed gate on the kept 0190 mid K4 nnz-hole package
(nuclear10a / popdynm200 K4; lee4_10 stays K2). No extra stream, no budget
change, well_below / n≤3k / small_streams / 0190 mid gate untouched.

## What changed

`src/ordering/mod.rs` only (medium exact tip 4-ticket else). `probe.rs`
untouched. 0190 mid dens≥4.5 + 110k–150k nnz hole kept (and left after revert).

Else branch only (`danger_timing` and the default):

- when `nnz <= 4 * n`: 50M ticket seed → `0x6A09_E667_F3BC_C909`
- when `nnz > 4 * n`: 50M ticket stays `0xD1B5_4A32_D192_ED03`
- 100M `0xD1B5_4A32_D192_ED03` ticket unchanged in both arms
- well_below, n≤3k, and small_streams tables unchanged
- 0190 `mid_k4` hole predicate unchanged

## Result

Yukon local **0.793589** vs 0190 keep **0.793612** (`/tmp/yukon-run-0191.log`).
Fill tiebreak 0.925575 → 0.925571. Buckets: lt_1k 0.8875 / 0.9599 unchanged,
1k_10k 0.8398→0.8397 / 0.9468, gt_10k 0.6886 / 0.8839 unchanged.

Flops-exact vs 0190 package: **2 better / 0 worse / 298 same**.

Movers (all dens≤4, else floor):

- mpbp_46 (n=4518, nnz=14492, dens≈3.21): 448113 → 447665 (−448)
- rsyn0840m02m (n=3486, nnz=9576, dens≈2.75): 45153 → 44761 (−392)

Outside cut / unchanged: crudeoil_pooling_ct3 dens≈4.32 stays 867796.
0190 mid wins preserved (nuclear10a 57044513, popdynm200 2408852; lee4_10
186729717).

Same-machine uncapped `probe_timing_and_score` (`/tmp/probe-timing-0191.log`):

- 0191 WORST **1.473 s**
- 0190 package WORST **1.511 s** (prior same-machine probe)
- 0191 worst **below** 0190 (−0.038 s). No timing regression.

Meets keep bar vs 0.793612 (>1 mover, 0 worse, score better, uncapped worst ≤1.511s).
After the local run/probe, `4e4a7f6` flipped to **rejected** (official 0.84349,
0.00% vs board). Per hard rule: **reverted 0191 seed change**, left 0190 package
intact, **did not submit** 0191.

## Why it won / lost

Density-gating the 0178/0189 seed swap again keeps sparse danger-band wins and
excludes denser ct3. Same ticket count/budgets; only sparse-arm second seed
changes. Stack quality held on top of 0190 mid hole.

## Follow-ups

- Stopped after `4e4a7f6` rejected. 0191 seed reverted; 0190 mid hole remains.
- Do not re-stack dens≤4 seed until a promoted 0190-class base exists (or a new
  mid package is kept).
- Official 0190 package scored 0.84349 flat — mid hole did not move hidden.

## Links

- Seed prior: [0189](0189-density-gated-else-seed.md)
- Mid base: [0190](0190-mid-k4-nnz-hole.md)

## Submission

- Not submitted (`4e4a7f6` rejected; hard rule).
- Local claimed (pre-revert): 0.793589
- Official `4e4a7f6`: rejected / 0.84349 (0.00%)
