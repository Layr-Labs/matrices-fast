# 0099: component-wise donor mixing did not improve the available portfolio

Date: 2026-09-07. Base: local `58ccecf`, submitted as `d21ea6f6`.
Status: rejected from production; reference code is test-only.

The exact symbolic objective is additive over disconnected input components.
An ordering may interleave their vertices, but fill cannot cross components.
Therefore the exact column counts of each already-scored donor can be summed
by original component, and the best induced subsequence can be chosen for
each component independently. A final exact score verifies the assembled
permutation and its predicted sum before accepting it.

The implementation labels components once, reuses one scoring workspace,
keeps at most eight donor indices per component, and builds the final order
into a flat pre-sized output buffer. It admits n <= 50000 and nnz <= 200000,
with a two-million-unit input/score ledger reserving the incumbent and final
verification. There must be at least two components of size three or more:
one- and two-vertex components have order-invariant costs and cannot supply
a compensating improvement to a globally worse donor.

Two tests passed under the candidate build sandbox. Complementary orders of
two three-vertex paths each cost 23; mixing their best component subsequences
costs 18. A second test checks all 720 six-vertex permutations with reversed
donors, deterministic results, bijections, and exact score acceptance. Budget
exhaustion and invalid donors fail closed.

A read-only census found 34 disconnected dev patterns, 20 with multiple
components of size at least three. The focused production probe covered all
20, including the density-refused gams05 case: **zero changed exact flop
counts**, compared with the completed 300-row official `58ccecf` baseline.
This was a 20-row screen, not a fresh full-corpus performance run.

No production call remains. `component_mix.rs` is retained only under
`#[cfg(test)]` for its reproducible negative result and independent witness.
The existing score-unique runner-up ledger may not preserve component-diverse
donors, but changing that ledger would be a separate experiment and could
change upstream trajectories. There is no evidence yet to justify it.

Related: [0098](0098-bounded-structural-terminal-portfolio.md).
