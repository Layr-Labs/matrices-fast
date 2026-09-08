# MCS-ETree: minimal completion inside a supplied ordering

Pinar Heggernes and Barry W. Peyton, *Fast Computation of Minimal Fill Inside
a Given Elimination Ordering*, SIAM Journal on Matrix Analysis and
Applications30(4),1424–1444 (2008), doi:10.1137/070680680.
Author conference abstract: https://www.tau.ac.il/~stoledo/CSC07/HeggernesPeyton.pdf
Full paper text inspected through the author-paper record:
https://www.researchgate.net/publication/220656529_Fast_Computation_of_Minimal_Fill_Inside_A_Given_Elimination_Ordering
Direct CiteSeer PDF fetch failed; the accessible paper text supplied Sections3–5.

The method combines elimination-tree root changes with a constrained maximum-
cardinality choice to obtain an inclusion-minimal completion inside the
starting one. Its implementation can avoid explicit filled graphs, but the
worst-case bound remains roughly n*m with an inverse-Ackermann factor.

Not implemented here. Our present terminal deletion search already targets
inclusion minimality; another subset-only method cannot improve an already
minimal completion. A blocked, budgeted implementation could be a future
replacement for incomplete descents, but no two-second feasibility or benefit
is established. The current alternative screen instead tests leaving the
completion by one certified insertion before further deletions.

See [0105](../experiments/0105-separator-insertion-repair-screen.md).
