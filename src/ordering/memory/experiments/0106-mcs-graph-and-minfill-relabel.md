# 0106 — MCS-on-graph + budgeted MinFill relabel (iter60)

## Dropped
iter56–59 residual-core ND path (timing 1.855s or score regress to ~0.80607). Per Hobs: no more core-ND densify variants.

## Leap
1. **MCS graph ordering**: maximum-cardinality search on the *input* adjacency (not fill completion). Different selection rule than AMD/AMF/MinFill. O(n+nnz); gate n<5k nnz<40k; relabel lottery budgeted.
2. **Budgeted MinFill relabel** across full MinFill gate (n<3k nnz<12k) instead of tip's nested n<2k/nnz<10k only — more deficiency-lottery tickets on cheap 2–3k ties.

Kept: EXTRA densify n<3k; tip exact-search floor (no medium+2).

## Result
(pending)
