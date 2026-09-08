# Chordal insertion certificates

Amol Deshpande, Minos Garofalakis, Michael I. Jordan, *Efficient Stepwise
Selection in Decomposable Models*. Author-hosted paper:
https://people.eecs.berkeley.edu/~jordan/papers/forwardselection.pdf
Also archived at https://arxiv.org/abs/1301.2267.

Read Section2, Theorems2.1 and2.2. A missing edge is insertable while retaining
chordality when its endpoints have a common-neighbor separator. Every common
neighbor lies on a length-two endpoint path, so a separating subset of common
neighbors must contain them all. This yields the implementation test: remove
the common neighbors and search for an endpoint-to-endpoint path. Absence of
that path certifies the insertion. Checking that the whole remaining graph is
disconnected would be insufficient; the endpoints specifically must separate.

Our use is a bounded adjacency-list certificate, not the paper's quadratic
eligible-edge matrix. It receives only the sparsity-derived completion, uses
no additional dependency, and charges the graph traversal. The paper optimizes
a statistical-model objective, not our factorization score; no performance
claim transfers.

Maps to [0105](../experiments/0105-separator-insertion-repair-screen.md).
