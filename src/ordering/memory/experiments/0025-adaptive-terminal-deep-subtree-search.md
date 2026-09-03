# 0025 — Bound a failed terminal deep subtree search

- **Date:** 2026-09-03
- **Failed attempt:** 0.851168 → 0.849622, but hidden validation exceeded
  the 2.0 s per-matrix limit (`7bacfdd7-14b7-4f1b-8bb6-b0b3242bfe12`)
- **Corrected attempt:** frontier source 0.851055 → **0.850518**
  (−0.000537; fill **0.948442**)
- **Parent:** `dd06965` (promoted hidden score 0.876877)
- **Status:** locally verified bounded retry

## Hypothesis

Late subtree refinement benefits more from a few deep ranked searches than from
many shallow searches. The work must also have a narrow structural gate and a
small matrix-wide limit because local corpus timing does not cover hidden graph
shapes.

## Failure evidence

The first attempt added one 32M requested-work phase across
`n <= 350,000 && nnz <= 1,500,000`. Its public corpus score was 0.849622 and its
local worst call was 0.886 s, but the benchmark action reported:

> hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed

The wide gate allowed the full extra search on hidden matrices with expensive
elimination-tree structure. The local timing result was therefore not a safe
bound.

## Corrected design

The retry starts from `dd06965`, which already adds a 24M independent subtree
pass inside `1,000 <= n <= 80,000 && nnz <= 250,000`. One final pass runs after
that incumbent:

- `1,000 <= n < 10,000`: 4 ranked blocks × 4M requested operations,
  `min_s=16`, `max_s=768`.
- `10,000 <= n <= 80,000`: 8 ranked blocks × 2M requested operations,
  `min_s=16`, `max_s=1,200`.
- Both branches use one stream, `max_sub=1,200`, and round 5 seed
  diversification.

Each branch requests at most 16M operations per matrix, half the failed extra
phase. The narrow parent gate excludes the failed attempt's large and dense
region. A result is retained only when it is a bijection and strictly reduces
the trusted `sum(c_j^2)` score.

## Result

The full 300-matrix run passed at **0.850518** with fill **0.948442**. Bucket
flop geomeans are **0.896482 / 0.875923 / 0.796991** for
`lt_1k / 1k_10k / gt_10k`. The parent scores 0.851055 on the same corpus.

The focused allocation sweep gave these useful points:

| extra pass | score |
|---|---:|
| 4 × 2M, max_s 768 | 0.850794 |
| **4 × 4M, max_s 768** | **0.850530** |
| 4 × 8M, max_s 768 | 0.850242 |
| **8 × 2M, max_s 1200** | **0.850829** |

The final adaptive combination uses the stronger 4 × 4M medium result and the
safer 8 × 2M large result. On `rsyn0815m04m`, exact flops fell from 170,169 to
165,245. All 25 active tests passed. `probe_timing_and_score` measured a worst
`order()` time of **0.852 s** on `crudeoil_lee4_10`.

## Lesson

A local worst time is evidence, not a hidden-runtime bound. Keep each new search
phase bounded independently, and gate it by both dimension and nonzero count.
After many shallow subtree rounds, use the remaining work for depth, but do not
extend that work to graph classes that the local corpus does not test safely.

## Links

- Earlier bounded subtree pass: [0022](0022-bounded-subtree-work.md)
- Earlier widening chain: [0024](0024-subtree-round-4-chain.md)
- Portfolio and timing policy: [best-of-portfolio](../techniques/best-of-portfolio.md)
