# 0164 — late iter74 polish on the degree<=3 residual core

- **Date:** 2026-09-09
- **Score:** 0.793834 → **0.793765** (−0.69 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

The open residual-core lead is still unused as a replacement: one existing late cleanup should run on the residual core of the shipped degree<=3 lift instead of on the full graph. Same task count, sequential, never additive. Not the failed nested K=4 prefix, not FINAL_FIVE, not pair-EXT, not the subtree chain.

## What changed

Replaced the iter74 late exact-polish ticket (`n >= 16 && n < 3_000 && nnz <= 12_000`, already inside `n <= 10_000` and `nnz <= 80_000`). That ticket is the late full-graph `rgreedy::search` streams plus the trailing adjacent-pair descent, sequential, same budgets and seeds.

When `core_lift::reduce` at degree <= 3 returns a real shrink (`prefix` nonempty, `cn < n`, `cn >= 2`) and `prefix_flops < best_flops`, those same streams run on the residual core. The seed is the incumbent's relative order of the core vertices. A candidate is spliced and admitted only on a trusted strict full-score decrease. If there is no real shrink, no headroom, or the lift fails, the original full-graph ticket runs. Never both. `SUBTREE_CHAIN_MAX_N` stays 45_000.

Code was reverted after the local run. HEAD remains `74b6ccd`.

## Result

Yukon local **0.793765** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0164.log`). Short of the 1 bip local bar (need `< 0.793734`). Buckets: lt_1k 0.8873, 1k_10k 0.8397, gt_10k 0.6891 (gt_10k unchanged).

8 better / **3 worse** / 289 same. Better: chimera_lga-01, korcns, multiplants_stg1, netmod_kar1, pooling_adhya4pq, risk2bpb, sporttournament18, syn15m04m. Worse: chimera_mgw-c8-439-onc8-002, rsyn0810m02hfsg, rsyn0820m02m. Not submitted. `src/ordering/mod.rs` restored to the tip.

## Why it won / lost

The replacement can move cheap rows (8 strict wins) but is not monotone on the shipped incumbent: splicing a polished core after the exact degree<=3 prefix drops the full-graph late-polish basin on three rows. The net is only −0.69 bip, so it misses both the 1 bip floor and the no-regression rule. Do not retry this ticket as an additive stack, and do not replace iter74 this way again without a guard that refuses a splice worse than the unreplaced full-graph polish.

## Follow-ups

- Do not add a second late pass on the same rows. The open lead wants a replacement, not a stack.
- A later replacement needs an explicit floor against the full-graph ticket's own result (run one, keep the better) — that is still one admit, but it is not what 0164 shipped, and it was not tested here.

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Related: [0062](0062-reduce-then-amf-terminal.md), [0075](0075-residual-core-minfill.md)
