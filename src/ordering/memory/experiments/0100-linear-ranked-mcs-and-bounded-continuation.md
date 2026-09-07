# 0100: linear neighbor-rank preparation and bounded structural MCS continuation

Date: 2026-09-07. Effort: max. Status: local follow-on, not submitted.

This experiment starts from `58ccecf`, the 0098 candidate whose full local
score was 0.806370063938585. Its remote submission `d21ea6f6` subsequently
failed without a score or disclosed error category. That failure remains
unexplained; this follow-on is not evidence that the hidden failure is fixed.

## Structural candidate change

The terminal rank family retains the first four ID/completed-degree MCS
orders and adds original degree, completion fill surplus, and current
symbolic column count, each in both rank directions. Rank ties use original
vertex IDs. Equal total rank orders are detected by vector equality, so
redundant extra rank pairs are skipped without probabilistic hashing.

All candidates are generated from one completion snapshot and exact-scored.
A strict winner earns a new round; the chain stops at a fixed point or four
rounds. Admitted rounds share a 1500000-unit `n + nnz + factor_nnz` allowance.
The existing dimension/nonzero/factor limits are unchanged. In particular,
the first round always fits that allowance under the existing limits, so the
four inherited candidates remain available. Later-round symbolic prechecks
are bounded by the four-round limit but are not represented as primitive
operation counts by this structural work proxy.

## Output-preserving preparation optimization

The earlier method comparison-sorted each completed-graph neighbor list for
every static rank. The graph is undirected, so there is a linear alternative:
visit vertices in total rank order and append each vertex into every adjacent
row. The resulting rows are exactly those produced by comparison sorting on
that total rank. One scratch adjacency allocation is reused across ranks.
The outer vertex-rank sort remains O(n log n); neighbor preparation becomes
O(n + completed edges) rather than repeated per-row comparison sorting.

This trades an additional adjacency scratch buffer for fewer comparisons.
It is not allocation-free. Reconstruction, MCS itself, exact scoring, and
the extra rounds are separate costs. Scratch is local to the current call;
there is no cross-matrix cache or corpus lookup.

The rank-transpose test compares every emitted row and both MCS directions
with the comparison-sorted reference on all labelled graphs through five
vertices and three total-rank families. The existing exhaustive completion
tests also cover every graph and input permutation through five vertices,
checking PEO validity and exact flop non-regression. All three focused PEO
tests passed under the candidate build sandbox.

## Matched preparation benchmark

The ignored `peo_extract::tests::probe_rank_sort_vs_transpose` benchmark uses
actual completed graphs from the current ordering of three public dev inputs.
It preallocates both methods' buffers, cycles through the same five ranks,
alternates arm order over six trials, and verifies byte-equal final rows.
Each timing includes four cycles of five rank preparations; the table gives
the median of six measurements. These are preparation-kernel times, not
end-to-end order() speedups. Allocation, reconstruction and outer rank sorting
are outside this microbenchmark.

| Public input | Comparison sorting (ms) | Rank transpose (ms) | Speedup |
|---|---:|---:|---:|
| mpbp_48 | 43.285 | 7.459 | 5.803x |
| crudeoil_lee4_10 | 225.522 | 24.624 | 9.158x |
| nuclear10a | 95.686 | 12.264 | 7.802x |

## Full local score gate

The supplied sandboxed `yukon run` passed all 300 development matrices,
including determinism, bijection and per-worker time checks. Exact integer
flop counts give **0.806316086866332**, versus **0.806370063938585** before:
0.66938 relative score basis points, 10 improved rows and zero regressions.
The bucket ratios are 0.889699744059344 / 0.845191841037713 /
0.714621528343038. The six-decimal emitted score is 0.806316.
The complete sandboxed candidate test suite passed 80 active tests with
zero failures and 31 ignored research/benchmark tests.

| Changed input | Before | After |
|---|---:|---:|
| mpbp_48 | 16194131 | 16188775 |
| chimera_mgw-c16-2031-01 | 2635459 | 2635088 |
| crudeoil_pooling_dt2 | 10397304 | 10396820 |
| crudeoil_lee4_10 | 198559373 | 198557639 |
| procurement1large | 7392848 | 7392774 |
| crudeoil_lee4_06 | 43155507 | 42814429 |
| crudeoil_lee2_06 | 21022199 | 21021960 |
| nuclear10a | 58258512 | 58253472 |
| crudeoil_lee4_09 | 142280230 | 142279294 |
| mpbp_35 | 980034 | 980018 |

The earlier unlimited four-round experiment on a different donor-containing
base gained more on crudeoil_lee4_10. The explicit allowance intentionally
forgoes that additional work; the numbers from that older base cannot replace
this measured comparison. There are no new random seeds or identity gates.

## Next validation need

The predecessor's opaque hidden failure makes extra runtime a live risk.
Before another upload, isolate STRIP/TELOS and ranked-continuation costs, and
evaluate a smaller added-work policy. A faster preparation kernel does not
prove that a wider multi-round family is cheaper end to end. Keep the
output-preserving optimization separable from the score-seeking expansion.

Related: [0098](0098-bounded-structural-terminal-portfolio.md),
[0099 component mixing](0099-component-donor-mixing-negative.md).
