# 0025 — Replace a failed additive terminal subtree search

- **Date:** 2026-09-03
- **Failed attempts:** 0.849622 with a 32M additive phase (`7bacfdd7`), then
  0.850518 with a 16M additive phase (`7cdbc0ea`); both exceeded the hidden
  2.0 s per-matrix limit
- **Replacement attempt:** frontier source 0.851055 → **0.850594**
  (−0.000461; fill **0.948499**)
- **Parent:** `dd06965` (promoted hidden score 0.876877)
- **Status:** locally verified lower-work replacement

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

The second attempt narrowed the gate to `n <= 80,000 && nnz <= 250,000` and cut
the added phase to 16M. It scored 0.850518 locally with a 0.852 s worst call,
but submission `7cdbc0ea-d6e8-40ba-a401-47b0dcabffbb` failed with the same
hidden timeout. The promoted frontier therefore has too little hidden margin
for another general terminal phase. The additive architecture was the cause;
the wide gate was only an amplifier.

## Corrected design

The retry starts from `dd06965`, removes its 24M independent subtree pass, and
replaces it inside the same `1,000 <= n <= 80,000 && nnz <= 250,000` gate:

- `1,000 <= n < 10,000`: 4 ranked blocks × 4M requested operations,
  `min_s=16`, `max_s=768`.
- `10,000 <= n <= 80,000`: 8 ranked blocks × 2M requested operations,
  `min_s=16`, `max_s=1,200`.
- Both branches use one stream, `max_sub=1,200`, and round 5 seed
  diversification.

Each branch requests at most 16M operations per matrix. This is 8M less work
than the promoted frontier's removed phase and 16M less than the first failed
addition. A result is retained only when it is a bijection and strictly reduces
the trusted `sum(c_j^2)` score.

## Result

The full 300-matrix run passed at **0.850594** with fill **0.948499**. Bucket
flop geomeans are **0.896482 / 0.876098 / 0.797049** for
`lt_1k / 1k_10k / gt_10k`. The parent scores 0.851055 on the same corpus.

The focused allocation sweep gave these useful points:

| extra pass | score |
|---|---:|
| 4 × 2M, max_s 768 | 0.850794 |
| **4 × 4M, max_s 768** | **0.850530** |
| 4 × 8M, max_s 768 | 0.850242 |
| **8 × 2M, max_s 1200** | **0.850829** |

Those rows measured additive passes and selected the allocation. Replacing the
frontier pass with the same adaptive allocation scores 0.850594. It gives up
only 0.000076 against the failed additive result while lowering terminal work
below the accepted frontier. On `rsyn0815m04m`, exact flops fell from 170,169 to
165,245. All 25 active tests passed. `probe_timing_and_score` measured a worst
`order()` time of **0.829 s** on `crudeoil_lee4_10`.

## Lesson

A local worst time is evidence, not a hidden-runtime bound. When an accepted
solver is close to the hidden cap, a smaller gate does not make additive work
safe. Replace existing work with a stronger equal-cost or lower-cost allocation.
After many shallow subtree rounds, spend fewer operations on deeper ranked
trajectories instead of adding another phase.

## Links

- Earlier bounded subtree pass: [0022](0022-bounded-subtree-work.md)
- Earlier widening chain: [0024](0024-subtree-round-4-chain.md)
- Portfolio and timing policy: [best-of-portfolio](../techniques/best-of-portfolio.md)
