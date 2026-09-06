# 0075 - Exact MinFill on residual cores

- **Date:** 2026-09-06
- **Score:** 0.832286 -> 0.832118 (geomean vs AMD; lower is better)
- **Status:** win locally; official result pending

## Hypothesis
After the existing exact degree-<=3 prefix, the residual core is small enough
for a genuinely different greedy objective. Exact MinFill on that core should
find candidates missed by the existing AMD/AMF portfolio without changing the
fixed prefix or violating the AMD floor.

## What changed
`leader_order` in `src/ordering/mod.rs` now runs `minfill_order` on residual
cores with `cn <= 1,000`, validates the permutation, scores the exact
prefix/core split, and admits the spliced ordering only through the existing
strict best-of path. `src/ordering/probe.rs` gained ignored probes for
multi-seed relabels of the other numbering-sensitive routines and for residual
core AMF/AMD/MinFill candidates.

## Result
The complete trusted 300-matrix run improved the weighted score from `0.832286`
to `0.832118` and the fill tiebreak from `0.938912` to `0.938869`.

| bucket | flop geomean before | flop geomean after | fill before | fill after |
|---|---:|---:|---:|---:|
| `lt_1k` | 0.889764 | 0.889764 | 0.960528 | 0.960528 |
| `1k_10k` | 0.865327 | 0.864765 | 0.956016 | 0.955872 |
| `gt_10k` | 0.764398 | 0.764398 | 0.909871 | 0.909871 |

The residual-core probe found 7 strict movers, all in the medium bucket. The
AMF/AMD residual relabel variants and the separate eight-seed RCM/Sloan/ND/NDFM
probe found no additional movers. The timing probe measured a worst local
`order()` call of about `0.582 s` in this run series, below the known `1.019 s`
passing revision; absolute timings remain machine-dependent.

## Why it won / lost
MinFill supplies a different elimination objective on the reduced graph, while
`core_lift::splice` makes the candidate's score exact for the full ordering.
The candidate is bounded to small residual cores and strict best-of admission
prevents a candidate from replacing the incumbent unless its exact score wins.
The small score gain is confined to `1k_10k`; no large-bucket headroom was found.

## Follow-ups
- Do not retry multi-seed RCM/Sloan/ND/NDFM relabeling under the current gates.
- Treat broader residual-core search as a separate timing experiment; do not
  add work to the slow tiers without a replacement budget.

## Links
- Techniques: [best-of-portfolio](../techniques/best-of-portfolio.md)
- Research queue: [open-questions](../open-questions.md)
