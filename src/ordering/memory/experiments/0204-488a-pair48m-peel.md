# 0204 — 488a PAIR96M peel (+ SUBTREE150k contingency)

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` / jonathan308 475a @ hidden 0.84349 (local tip 0.793834)
- **Path:** Scoreboard-488a-PAIR48M → SUBTREE150k after 0-worse
- **Status:** **MISS / NO SUBMIT** (board moved to 62654a5 / 647a @ 0.843173; 74b6ccd stale). Package `/tmp/0204-package/`.

## Hypothesis

0203 full 488a (PAIR_EXT 96M) scored 0.793790 with 12/4. Peel PAIR96M→48M to clear regs; if 0-worse but score>0.79375, stack 0163 SUBTREE150k.

## Stage 1 — peel (PAIR@48M)

Stack vs tip: mid_force 7–22k/nnz48k + FINAL_FIVE 14k/100k OPS192M + REDUCE deep 120k/250k + PAIR_EXT **48M**. No SUBTREE yet.

Yukon `/tmp/yukon-run-0204.log`: **SCORE 0.793796**. **11 better / 0 worse**.
Better: nuclear10a (−104139), crudeoil_lee2_06 (−2568), mpbp_15 (−2328), mpbp_34 (−537), chp_shorttermplan2a (−345), mpbp_35 (−290), glider400 (−263), transswitch0300p (−257), powerflow0300p (−221), rsyn0840m04m (−110), chp_partload (−88).
0203 regs cleared (li05/lee4_06/powerflow0118p same; rsyn0840m04m better).

## Stage 2 — + SUBTREE150k same-count

`SUBTREE_CHAIN_MAX_N` 150k; `SUBTREE_CHAIN_ROUNDS_MAX_N` 45k; first-round only on 45k<n≤150k nnz<1.2M; late PEO-large skipped.

Yukon `/tmp/yukon-run-0204b.log`: **SCORE 0.793789** (−0.45 bip vs tip). Fill ~0.9257.
Buckets: lt_1k 0.8875 / 1k_10k 0.8398 / gt_10k 0.6890.

Flops vs tip: **14 better / 0 worse / 286 same**.
Stage1 set retained + n>45k: cont6-qq (−19824), transswitch2736spr (−4277), transswitch2383wpr (−1983).

| criterion (old tip gate) | result |
|---|---|
| SCORE ≤ 0.79375 OR tip−0.0001 | **FAIL** (0.793789) |
| ≥10 better / 0 worse | **PASS** (14/0) |
| uncapped worst ≤ 1.14s | **not probed** (no submit; board moved) |

## Decision

**No submit** on 74b6ccd (stale). Entire 0204 stack reverted; rebase to **62654a5**.
