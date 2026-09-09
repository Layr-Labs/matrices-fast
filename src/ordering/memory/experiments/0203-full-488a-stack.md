# 0203 — full 488a-class stack on clean tip (CoS-locked)

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` / jonathan308 475a @ hidden 0.84349 (local tip 0.793834)
- **Rival:** jonathan 488a near-miss hidden **0.843417** (~0.73 bip), local ~0.793790, worst ~1.114s
- **Score:** 0.793834 → **0.793790** (−0.000044 / −0.44 bip)
- **Status:** **MISS / REVERTED**. Not submitted. Identical twin of jonathan 488a local score.

## Hypothesis

Ship the **full** 488a four-widen stack (not a thinned twin) on clean tip to measure local score under the new promote gate (≤0.79375 or tip−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s). Beyond-488a levers deferred until this baseline is known. No extrarelbl / KaHIP / Scotch.

## Exact stack (vs tip) — applied then reverted

`src/ordering/mod.rs` only:

1. **mid_force:** `(7_000..22_000).contains(&n) && nnz >= 48_000` (was 8_000..20_000 && nnz≥50_000)
2. **FINAL_FIVE:** `MAX_N=14_000`, `MAX_NNZ=100_000`, **OPS=192_000_000** (was 12k/80k/128M)
3. **REDUCE deep:** `REDUCE_RECURSE_MAX_NNZ` 150k→**250k**; `REDUCE_RECURSE_DEEP_MAX_CORE_N` 80k→**120k**; `DEEP_MAX_CORE_NNZ` stays 250k
4. **PAIR_DESCENT_EXT_OPS:** 48M→**96_000_000**

Unchanged: SUBTREE_CHAIN_MAX_N 45k, pair_descent_ext nnz≤60k / max_deg*50≤n, no extrarelbl, no KaHIP Eco/seed2, no Scotch.

## Result

Yukon `/tmp/yukon-run-0203.log`: **SCORE 0.793790**, fill **0.925745**.
Buckets: lt_1k 0.887474 / 1k_10k 0.839778 / gt_10k 0.689036.

Flops vs tip: **12 better / 4 worse / 284 same**.

Better: nuclear10a (−104139), crudeoil_lee2_06 (−3227), mpbp_15 (−2328), mpbp_34 (−690), mpbp_35 (−547), mpbp_07 (−432), powerflow0300p (−363), chp_shorttermplan2a (−345), transswitch0300p (−332), chp_shorttermplan1a (−324), glider400 (−263), chp_partload (−88).

Worse: crudeoil_li05 (+326), crudeoil_lee4_06 (+295), rsyn0840m04m (+113), powerflow0118p (+17).

## Gate check

| criterion | result |
|---|---|
| SCORE ≤ 0.79375 OR ≤ tip−0.0001 (0.793734) | **FAIL** (0.793790) |
| ≥10 better / 0 worse | **FAIL** (12/4) |
| uncapped worst ≤ 1.14s | not probed (score/mover gate already failed; no submit) |

## Why miss

Full 488a on this tip reproduces jonathan’s local **0.793790** twin — not past the promote bar, and PAIR_EXT 96M introduces 4 regressions (li05 / lee4_06 / rsyn0840m04m / powerflow0118p). Do not submit identical twin. Entire stack reverted to tip `74b6ccd` constants.

## Follow-ups

- Tip left clean. Beyond-488a structural lever needed to beat 0.843417; do not re-ship this twin alone.
- Stop after 0203 (no 0204).
