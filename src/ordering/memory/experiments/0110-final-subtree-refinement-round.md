# 0110 — refine the ordering the pipeline actually ships, not the one it had at stage 3

- **Date:** 2026-09-08
- **Score (on `0fb513b`):** 0.805518 → **0.805234** (−2.84 bips), ~55 movers.
  Priced on three consecutive frontiers: `c6b0311` −3.03 / 64 movers,
  `2fef69b` −2.89 / 55, `0fb513b` −2.84.
  First measured on `c6b0311`: 0.806243 → **0.805940** (−3.03 bips, 64 movers,
  fill tiebreak 0.930420 → 0.930243); `2fef69b`'s late polish / `n<=1000`
  refine / wider `SmallScore` took 0.14 bip of it, all at the small end.
- **Status:** win

## Hypothesis

`leader_order` calls `rgreedy::subtree_refine` at exactly two places and both
are at stage 3 — the chain (`n <= SUBTREE_CHAIN_MAX_N = 35_000`) and the
terminal deep pass (`n <= 80_000 && nnz <= 250_000`). Everything after them can
*replace* the incumbent: reduce-then-order on the residual core, the completion
watcher, the small-graph polish, the terminal PEO chain, the alternate-seed
chains, the transplant probe, MINL and the count-ranked peel. So the ordering
that actually ships has, on many rows, never been refined at all.

The tree already contains this argument in miniature: the MINL call site refines
its own winner because "a strict MINL win is a NEW completion the subtree chain
has never refined". The hypothesis is that it does not matter *which* stage won,
only that something after stage 3 did — and only the end of the function knows.

## What changed

`src/ordering/mod.rs` only, **+109 lines / −0**. One constant
(`FINAL_REFINE_MAX_WORK = 400_000`) and one block immediately before
`best_perm` is returned: rebuild the postorder of the finished incumbent, then
run `subtree_cfg_for(n, nnz)` and `terminal_deep_subtree_cfg(...)` on it, each
admitted only on a strict decrease of the exact score.

Gate: `n >= SUBTREE_MIN_N && n + nnz <= FINAL_REFINE_MAX_WORK`.

Two implementation points that matter:

- **Both rounds start from the same postordered candidate.** `counts` and
  `parent` describe that candidate's elimination tree; chaining round two onto
  round one's output would hand `subtree_refine` a tree that no longer matches
  its input and would need a second full setup.
- **The incumbent's score falls out of the setup.** Postorder is
  objective-neutral (verified: 0 of 300 rows change their flop count under it),
  so `Σ counts²` of the postordered pattern *is* `score(best_perm)`. No extra
  scoring pass.

## Result

| bucket | rows | before | after | movers |
|---|---:|---:|---:|---:|
| `lt_1k` | 147 | 0.888210 | 0.888130 | 5 |
| `1k_10k` | 108 | 0.843539 | 0.843100 | 30 |
| `gt_10k` | 45 | 0.715028 | 0.714696 | 20 |
| **score** | 300 | **0.805536** | **0.805247** | **55** |

**55 of 300 rows change their permutation, 0 get worse** (admission is a strict
decrease, so a regression is structurally impossible). Largest single-row gains:
`crudeoil_lee1_07` −1.71 %, `rsyn0820m04m` −0.89 %, `pooling_rt2tp` −0.77 %,
`crudeoil_pooling_dt2` −0.70 %, `multiplants_stg1` −0.55 %, `pinene200` −0.37 %.

The movers are **broad, not concentrated**: 5 / 30 / 20 across the three
buckets. The log's own rule from the `5d2aeab9` and `2e3c00fb`
rejections is that breadth of dev movers is what translates.

Cost: mean +6.1 ms per row, worst +24 ms (`transswitch2383wpr`), +12 ms on the
corpus-maximum row. Gate curve measured on all 300 rows before choosing:

| `n + nnz` | dev bips (`2fef69b`) | rows > +25 ms | worst |
|---|---:|---:|---:|
| ungated | 3.098 | 7 | 110 ms |
| 1 000 000 | 3.090 | 4 | 36 ms |
| 500 000 | 2.946 | 1 | 27 ms |
| **400 000** | **2.886** | **0** | **23 ms** |
| 300 000 | 2.850 | 0 | 18 ms |

`acopf_case9241pegase_qcqp`, `faclay75` and `gabriel10` are the only rows the
ungated form spends 75–119 ms on, and they gain 0.012 % / 0.000 % / 0.001 %
between them — the last 0.21 bip costs the whole time exposure, so it is not
bought.

## Why it won

**Position, not gate width.** The biggest winners are rows that were already
inside *both* stage-3 gates (`crudeoil_lee1_07` n = 3 670, `rsyn0820m04m`
n = 6 028, `pooling_rt2tp` n = 118). The same search, run on the permutation the
function is about to return, finds what the earlier call could not because the
earlier call was looking at a different permutation.

The added cost is bounded by one inequality because every term is linear in
`n + nnz` with no output-driven factor: two `permute_pattern`s, two
`EliminationTree::from_pattern`s, one postorder, one `column_counts_gnp`
(Gilbert-Ng-Peyton, `O(nnz α(n))`, never `O(Lnnz)`), two `O(n)` clones, and two
`subtree_refine` calls already bounded by `SUBTREE_SEARCH_WORK_LIMIT` /
`TERMINAL_SUBTREE_SEARCH_WORK_LIMIT`.

## Three negative results on `gt_10k`, measured first

`gt_10k` carries weight 0.40 over 45 rows and the recent promotions are all
gated `n < 10_000`, so I priced three ways to move its sixteen rows at AMD
ratio ≥ 0.94, always against the **full** shipped `order()` result:

| attempt | tickets | wins | value | cost |
|---|---:|---:|---:|---|
| relabelled residual-core lottery extended above `n = 10_000` (the shipped block is gated `(1_000..10_000).contains(&n) && nnz <= 50_000`): 4 i.i.d. relabels × {AMF α0.5/2.5/5/10, AMD}, exact core ranking, spliced through `prefix_flops + flops(core, p)` | 20/row | **0 of 45** | 0.000 bips | 11.9 s corpus-wide |
| 8 more AMD and 8 more AMF lottery tickets per row, continuing production's own seed stream and ignoring `RELABEL_AMF_MAX_NNZ` | 16/row | **1 of 45** | 0.119 bips (`acopf` 0.9737 → 0.9719 on the 6th extra ticket) | **0.38 s per ticket** on that row |
| METIS default / tuned (16 iparts, 20 FM) / hi-trial (32/30) and Scotch on every `gt_10k` row regardless of gate | 4/row | **0 of 45** | 0.000 bips | ratios 1.0–40x (METIS), up to 9 519x (Scotch on `faclay75`) |

So the `n < 10_000` self-gating is a correct reading of the residue: the
min-degree/min-fill lottery is saturated there and nested dissection is
catastrophic. 27 of the 45 `gt_10k` rows do improve — under refinement, not
under a new generator.

## Follow-ups

- The same question one level up: **which other stage-3 mechanism only ever sees
  a permutation that later stages discard?** `refine_core`, simplicial
  promotion, the five-/four-pivot descents and the adjacent-pair descent all sit
  in the same position.
- The gate is deliberately conservative at 400 000. If a future frontier buys
  back per-row headroom, 1 000 000 is worth a further 0.20 bip.
