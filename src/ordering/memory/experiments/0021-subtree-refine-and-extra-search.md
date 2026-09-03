# 0021 — Subtree refinement + extra exact-search streams

- **Date:** 2026-09-03
- **Score:** 0.859116 → **0.855650** (fill 0.955319 → **0.953783**)
- **Status:** win
- **Matrices:** 62 better / **0 worse** / 238 identical (300-matrix dev corpus)

## Hypothesis

The exact elimination-game search already in `rgreedy` is the strongest local
optimizer in the portfolio, but it is gated off the high-weight `gt_10k`
bucket because a bitset fill graph on the whole matrix is too expensive.
An elimination-tree *subtree* is an independently optimizable subproblem:
column counts inside a subtree depend only on that subtree plus its
boundary, and reordering inside it cannot change flops above the subtree
root. `subtree_refine` was already implemented and unused. Wiring it on
sparse medium/large graphs should move `gt_10k` without touching the slow
dense-KKT tier.

Separately, `n <= 1000` still has unused wall-clock. Four independent
default-policy trajectories at 80M ops, started from the *same* portfolio
incumbent as the existing 100M stream, should add small-bucket wins
without replacing the accepted 100M path.

## What changed

Edits only in `src/ordering/mod.rs` (plus this note). No new dependencies.

1. **Subtree refinement** for
   `1,500 <= n <= 80,000 && nnz <= 350,000 && nnz <= 5n && max_deg*50 <= n`.
   The incumbent is postordered (flops-invariant) so each etree subtree is a
   contiguous block. `rgreedy::subtree_refine` then searches up to 24 blocks
   (`min_s=24`, `max_s=900`, `max_sub=2800`, 16M ops, 1 stream, ranked).
   Density and hub gates keep this off `crudeoil_lee4_*` / `gams05`.
2. **Ammf and AmindNorm** custom quotient metrics, already implemented in
   `custom_metrics.rs` but unused, under the existing dense gate
   `nnz >= 10n && nnz <= 300k` with an extra `n < 8,000` cap so they do not
   run on `gams05`.
3. **Third 50M exact-search stage** on `1,000 < n <= 3,500 && nnz <= 20,000`,
   using a different fixed seed from the two existing medium stages.
4. **Four parallel default-policy exact-search streams** (80M each) on
   `n <= 800 && nnz <= 12,000`, started from the same portfolio snapshot as
   the original 100M stream. Replacing the 100M stream lost some of its
   basins; running both from the same seed keeps every previous small-graph
   win and adds the extra-stream wins.

Every new candidate is bijection-checked and accepted only on a strict
trusted-scorer improvement. AMD remains the floor.

## Result

Full `yukon run` on the 300-matrix development corpus:

| Bucket | Parent | Candidate |
|---|---:|---:|
| `lt_1k` | 0.896482 | **0.894949** |
| `1k_10k` | 0.884759 | **0.880063** |
| `gt_10k` | 0.811860 | **0.807866** |
| weighted score | 0.859116 | **0.855650** |
| fill tiebreak | 0.955319 | **0.953783** |

62 matrices improved, 0 worse. Representative exact-flop changes:

| Matrix | n | Parent flops | Candidate flops | Change |
|---|---:|---:|---:|---:|
| `wastewater05m1` | 98 | 8,935 | 8,189 | −8.35% |
| `gasprod_sarawak16` | 4,596 | 213,026 | 195,801 | −8.09% |
| `mpbp_35` | 11,120 | 1,423,831 | 1,322,586 | −7.11% |
| `pooling_adhya4pq` | 170 | 21,640 | 20,324 | −6.08% |
| `mpbp_21` | 11,716 | 1,555,528 | 1,471,455 | −5.40% |
| `mpbp_34` | 11,556 | 1,397,396 | 1,345,503 | −3.71% |
| `transswitch0300p` | 11,659 | 393,759 | 388,482 | −1.34% |
| `powerflow0300p` | 11,251 | 302,379 | ~295k | −1.56% |

`gt_10k` moved on 15 matrices, including a former 1.000 tie
(`gasprod_sarawak81` 1.000 → 0.998). The wall-clock of the full harness
run was ~244 s vs ~226 s for the parent on the same box; per-matrix
`order()` still prints `(capped)` well inside the 2 s SIGKILL.

## Negative / course correction

Replacing the `n<=1000` 100M stream with the 4×80M fan lost basins
(`syn40m` 4,879 → 4,886, `wastewater05m1` the other way). LNS is
seed-dependent: extra streams must start from the *same* portfolio
incumbent, not from a 100M-improved incumbent and not as a replacement.
After that fix: 62 better / 0 worse.

## Links

- Technique: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Predecessor: [0020-medium-exact-search](0020-medium-exact-search.md)
