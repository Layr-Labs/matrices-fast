# 0189 — density-gated else-floor second seed (nnz≤4n)

- **Date:** 2026-09-09
- **Base:** 0188 package on tip `74b6ccd` (local keep 0.793612; tip 0.793834). Official `d6c204c9` validating at run time.
- **Score:** 0.793612 → **0.793589** (−0.000023)
- **Status:** local KEEP bar met, then **reverted** — official `d6c204c` **failed** (n/a). Not submitted. 0188 package left intact.

## Hypothesis

0178 swapped the tip 4-ticket else-floor 50M repeat seed
`0xD1B5_4A32_D192_ED03` → `0x6A09_E667_F3BC_C909` unconditionally and won
`mpbp_46` / `rsyn0840m02m` while regressing `crudeoil_pooling_ct3`
(dens≈4.32). A structural density cut `nnz ≤ 4 * n` puts those two wins
inside and ct3 outside, so the sparse plateaus can take the new seed while
the denser regression stays on `0xD1B5`. Stacked on 0188 mid K2/K4. No
extra stream, no budget change, well_below / n≤3k / small_streams untouched.

## What changed

`src/ordering/mod.rs` only (medium exact tip 4-ticket else). `probe.rs`
untouched. 0188 mid density gate kept.

Else branch only (`danger_timing` and the default):

- when `nnz <= 4 * n`: 50M ticket seed → `0x6A09_E667_F3BC_C909`
- when `nnz > 4 * n`: 50M ticket stays `0xD1B5_4A32_D192_ED03`
- 100M `0xD1B5_4A32_D192_ED03` ticket unchanged in both arms
- well_below, n≤3k, and small_streams tables unchanged

## Result

Yukon local **0.793589** vs 0188 keep **0.793612** (`/tmp/yukon-run-0189.log`).
Fill tiebreak 0.925575 → 0.925571. Buckets: lt_1k 0.8875 / 0.9599 unchanged,
1k_10k 0.8398→0.8397 / 0.9468, gt_10k 0.6886 / 0.8839 unchanged.

Flops-exact vs 0188 package: **2 better / 0 worse / 298 same**.

Movers (all dens≤4, else floor):

- mpbp_46 (n=4518, nnz=14492, dens≈3.21): 448113 → 447665 (−448)
- rsyn0840m02m (n=3486, nnz=9576, dens≈2.75): 45153 → 44761 (−392)

Outside cut / unchanged: crudeoil_pooling_ct3 dens≈4.32 stays 867796
(0178 regression avoided). chimera_selby dens≈5.40 and sfacloc2_4_80 dens≈4.23
also unchanged (0178 wins that sat outside nnz≤4n). 0188 mid wins preserved
(nuclear10a, popdynm200; pooling_dt2).

Comparative `probe_timing_and_score` (SSI_PROBE_REPEAT=3) on movers + ct3 +
tip slow rows (`/tmp/probe-0189-movers.log`):

- focused-set worst order() **1.427 s** (0188 focused worst 1.450 s)
- mpbp_46 0.649 s, rsyn0840m02m 0.705 s, ct3 1.020 s
- lee4_10 1.428 s, nuclear10a 0.897 s, popdynm200 1.018 s

Meets keep bar vs 0.793612 (>1 mover, 0 worse, score better, worst order()
not up). After the local run, `d6c204c` flipped to **failed** (official score
n/a). Per hard rule: **reverted 0189 seed change**, left 0188 package intact,
**did not submit** 0189.

## Why it won / lost

Density-gating the 0178 seed swap keeps the sparse danger-band wins and
excludes the dens≈4.32 ct3 regression. Same ticket count and budgets; only
the sparse arm's second seed changes.

## Follow-ups

- Stopped after `d6c204c` failed. 0189 seed reverted; 0188 mid K2/K4 remains.
- Do not re-stack 0189 until a promoted 0188-class base exists.

## Links

- Prior miss: [0178](0178-medium-exact-second-seed.md)
- Stack base: [0188](0188-density-selected-mid-k4.md)

## Submission

- Not submitted (`d6c204c` failed; hard rule).
- Local claimed (pre-revert): 0.793589
- Official `d6c204c`: failed / n/a
