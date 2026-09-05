# 0021 — Exact elimination-tree subtree refinement

- **Date:** 2026-09-02
- **Score:** 0.859116 → **0.851513** (fill 0.955319 → **0.949215**)
- **Status:** public win; hidden timeout

## Hypothesis

An elimination-tree postorder puts every subtree in one contiguous permutation
range. Reordering one such range is an exact local problem: column counts inside
the subtree depend on its descendants, and the fill state above the subtree
depends on the eliminated vertex set, not its internal order. The dormant
`rgreedy::subtree_refine` code can therefore use exact elimination games on small
subtrees of a large matrix without searching the full matrix.

## What changed

After the existing terminal search and pair descent, `order()` postorders the
incumbent elimination tree for matrices with `1,000 <= n <= 350,000` and
`nnz <= 1,500,000`. These measured limits bound whole-pattern setup and scoring
on hidden inputs while keeping all public matrices. The phase recomputes the
postordered pattern, tree, column counts, and parent array. It then searches at
most 32 disjoint subtrees, selected by exact incumbent flop contribution per
subtree size. Each subtree has 32 to 384 internal vertices, at most 1,200
vertices including its boundary, two fixed search streams, and a nominal 2M
operation budget per stream.

The phase uses `round = 0`; this does not enable the narrow later-round seed
rules in `rgreedy`. It does not use matrix names, clocks, or new dependencies.
The full trusted scorer checks the combined candidate, and `order()` accepts it
only when it is a valid bijection with fewer flops.

A regression test first failed at ratio 0.308420 on `pooling_sppc1pq`. It passed
after the production call was connected, at ratio 0.212146.

## Result

The full 300-matrix Yukon run passed:

| Bucket | Before | After |
|---|---:|---:|
| `lt_1k` | 0.896482 | 0.896482 |
| `1k_10k` | 0.884759 | **0.877695** |
| `gt_10k` | 0.811860 | **0.798150** |
| weighted score | 0.859116 | **0.851513** |

Representative ratio reductions were:

- `pooling_sppc1pq`: 0.308420 → 0.212146
- `pooling_sppc3pq`: 0.764226 → 0.686528
- `mpbp_35`: 0.468796 → 0.429743
- `mpbp_21`: 0.919824 → 0.873580
- `nuclear104`: 1.000000 → 0.997940
- `acopf_case9241pegase_qcqp`: 1.000000 → 0.999469

The final timing probe measured a 0.801 s worst `order()` call. The prior
accepted version measured 0.755 s on the same computer. All 300 public matrices
stayed below the 2 s limit, but this local result did not predict the hidden
case.

## Hidden result and root cause

Submission `ce2b5d90-2f10-456a-9824-b0854759990e` failed because one hidden
matrix exceeded the 2 s cap. The phase had no matrix-wide search limit. Its
2M-operation budget applied to each stream of each block. At 32 blocks and two
streams, it requested up to 64 searches and 128M operations. Each search could
also use its documented 1.25x hard cap, and local graph setup was outside that
counter. The hidden matrix exposed this aggregate-work error.

## Configuration sweep

The probe measured only the 153 matrices with `n >= 1,000`. Its current partial
score was 0.843102.

| Configuration | Nominal search work | Partial score | Worst added time |
|---|---:|---:|---:|
| 32 small blocks, one stream | 64M | 0.839213 | 0.017 s |
| 32 ranked blocks, two streams | 128M | **0.832241** | 0.016 s |
| 64 ranked blocks, split streams | 256M | 0.831619 | 0.034 s |
| 64 ranked blocks, 5M budget | 640M | 0.830483 | 0.067 s |

The selected 128M setting looked like the best public score-to-time point. The
hidden timeout proved that this conclusion was unsafe because the sweep measured
only the public corpus.

## Why it won

The full-matrix portfolio makes global heuristic choices. Many of its
permutations still contain expensive local etree subtrees. Exact search can
replace only those blocks while keeping the rest of the incumbent fixed. The
method gave gains in both scored buckets and across pooling, power, process,
synthetic, and graph families. It is not one large-matrix outlier.

## Follow-ups

- Test a different fixed stream for the existing second medium whole-graph
  search without adding work.
- Apply a strict matrix-wide search ceiling before another hidden submission.
- Keep the subtree phase off `n < 1,000`; prior hidden results show that this
  tier has little time margin for added exact search.

## Links

- Technique: [best-of portfolio](../techniques/best-of-portfolio.md)
- Predecessor: [0020 medium exact search](0020-medium-exact-search.md)
- Fix: [0022 bounded subtree work](0022-bounded-subtree-work.md)
