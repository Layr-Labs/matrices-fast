# 0170 — iter74 full-graph floor + deg≤3 residual-core polish (0164 follow-up)

- **Date:** 2026-09-09
- **Score:** 0.793834 → **0.793785** (−0.49 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

0164 replaced the iter74 late polish with deg≤3 residual-core polish and got −0.69 bip with **3 worse**: the splice is not monotone against the full-graph basin. Follow-up: keep the full-graph ticket as an explicit **best-of floor**, then try the same core polish and admit only if the spliced trusted full score is strictly better than that floor. Never skip the floor. Same gates (`n∈[16,3000) && nnz≤12000`, cost≤20M). Not an unbound double-stack — only the existing cheap iter74 window.

## What changed

`src/ordering/mod.rs` only, inside the iter74 late-polish branch:

1. Run existing full-graph `late_streams` + `adjacent_pair_descent` exactly as tip (establishes floor).
2. After that, `core_lift::reduce(&scoring_pat, REDUCE_ROW_DEG, …)` with panic catch. Real shrink only: nonempty prefix / `cn < n` / `cn >= 2` / `prefix_flops < best_flops`.
3. Core seed = relative order of core vertices in the post-full-graph `best_perm`. Same stream budgets/seeds and pair-descent params on the core graph; internal accepts use core flops.
4. `core_lift::splice`, require bijection, trusted full `score`; admit only if `f < best_flops`.
5. No shrink / lift fail / no improvement → leave full-graph result untouched.

`SUBTREE_CHAIN_MAX_N` stays 45_000. No gasprod_band / pair_descent_ext / indep_first / HEAVY_METRIC / FINAL_FIVE edits. Not a retread of 0163–0169 replacements.

Code was reverted after the local run. HEAD tip remains `74b6ccd`.

## Result

Yukon local **0.793785** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0170.log`, tip compare `/tmp/yukon-run-tip-74b6ccd.log`). Short of the 1 bip local bar (need ≤ 0.793734). Buckets: lt_1k 0.8874, 1k_10k 0.8397, gt_10k 0.6891 (gt_10k unchanged).

Flops-exact vs tip: **6 better / 1 worse / 293 same**. Better: chimera_mgw-c8-439-onc8-002, multiplants_stg1, netmod_kar1, pooling_adhya4pq, risk2bpb, syn15m04m. Worse: chimera_lga-01 (497542 → 498826). Rounded-ratio table: 5 better / 1 worse. Not submitted. Timing not probed (bar + regression miss). `src/ordering/mod.rs` restored to tip.

## Why it won / lost

The floor correctly converts 0164's replacement regressions into a best-of admit and recovers several of 0164's wins without stacking new tickets. Net is only −0.49 bip (weaker than 0164's −0.69), so the 1 bip bar is still missed. One tip-visible regression remains on chimera_lga-01 despite the in-call floor — likely a cross-run / later-stage interaction rather than an admitted worse splice; either way it fails the 0-worse rule. Do not re-stack this ticket; the residual-core late polish lead is exhausted in this form.

## Follow-ups

- Stop on this lead. No 0171 from this chain.
- Open residual-core ideas that are not late-iter74 stacks remain separate (not pursued here).

## Links

- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Related: [0164](0164-late-phase-on-core.md), [0062](0062-reduce-then-amf-terminal.md), [0075](0075-residual-core-minfill.md)
