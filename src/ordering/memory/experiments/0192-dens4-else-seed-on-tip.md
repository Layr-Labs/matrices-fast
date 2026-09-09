# 0192 — density-gated else-floor second seed (nnz≤4n) on clean tip

- **Date:** 2026-09-09
- **Base:** clean tip `74b6ccd` (local 0.793834). Mid family closed for hidden; 0190 mid hole **reverted** (mid_k2 only). No Ammf / HEAVY_METRIC_ORDER edit.
- **Score:** 0.793834 → **0.793811** (−0.000023)
- **Status:** KEEP locally / submitted. Model Grok 4.6, harness Grok Bot.

## Hypothesis

0178 swapped the tip 4-ticket else-floor 50M repeat seed
`0xD1B5_4A32_D192_ED03` → `0x6A09_E667_F3BC_C909` unconditionally and won
`mpbp_46` / `rsyn0840m02m` while regressing `crudeoil_pooling_ct3`
(dens≈4.32). A structural density cut `nnz ≤ 4 * n` puts those two wins
inside and ct3 outside. Same change as 0189/0191 but on **clean tip**
(mid_k2 only; no mid K4 dens/hole stack). No extra stream, no budget
change, well_below / n≤3k / small_streams untouched.

## What changed

`src/ordering/mod.rs` only (medium exact tip 4-ticket else). `probe.rs`
untouched. Mid block restored to tip `mid_k2` (depth==2 only). 0190 mid
hole removed before this edit.

Else branch only (`danger_timing` and the default):

- when `nnz <= 4 * n`: 50M ticket seed → `0x6A09_E667_F3BC_C909`
- when `nnz > 4 * n`: 50M ticket stays `0xD1B5_4A32_D192_ED03`
- 100M `0xD1B5_4A32_D192_ED03` ticket unchanged in both arms
- well_below, n≤3k, and small_streams tables unchanged

## Result

Yukon local **0.793811** vs tip **0.793834** (`/tmp/yukon-run-0192.log`).
Fill tiebreak 0.9258 → 0.9258. Buckets: lt_1k 0.8875 / 0.9599 unchanged,
1k_10k 0.8398→0.8397 / 0.9468, gt_10k 0.6891 / 0.8844 unchanged.

Flops-exact vs tip: **2 better / 0 worse / 298 same**.

Movers (all dens≤4, else floor):

- mpbp_46 (n=4518, nnz=14492, dens≈3.21): 448113 → 447665 (−448)
- rsyn0840m02m (n=3486, nnz=9576, dens≈2.75): 45153 → 44761 (−392)

Outside cut / unchanged: crudeoil_pooling_ct3 dens≈4.32 stays 867796
(0178 regression avoided). Mid tip paths unchanged (nuclear10a /
popdynm200 / lee4_10 / pooling_dt2 bit-identical to tip).

Same-machine uncapped `probe_timing_and_score`:

- tip WORST **1.654 s** (`/tmp/probe-timing-tip-0190ab.log`)
- 0192 WORST **1.516 s** (`/tmp/probe-timing-0192.log`)
- worst **below** tip (−0.138 s)

Meets keep bar (>1 mover, beat 0.793834, 0 worse, uncapped worst ≤ tip).

## Why it won / lost

Density-gating the 0178 seed swap keeps the sparse danger-band wins and
excludes the dens≈4.32 ct3 regression. Same ticket count and budgets; only
the sparse arm's second seed changes. Running on clean tip (no mid K4)
avoids stacking on the hidden tip-tie mid package.

## Follow-ups

- Mid K2/K4 dens/hole family closed for hidden after 0190 tip-tie reject.
- Do not restack dens≤4 seed on dead mid bases.

## Links

- Prior (reverted stacks): [0189](0189-density-gated-else-seed.md), [0191](0191-0190-plus-dens4-seed.md)
- Prior miss: [0178](0178-medium-exact-second-seed.md)

## Submission

- Submitted (see log for id).
- Local claimed: 0.793811
- Public note: dens≤4 else-floor 50M seed on clean tip mid_k2; 0190 mid hole reverted/closed for hidden.
