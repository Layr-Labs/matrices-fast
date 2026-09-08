# Minimum fill is not minimum FLOPs

Robert Luce and Esmond Ng, *On the minimum FLOPs problem in the sparse
Cholesky factorization*, SIAM Journal on Matrix Analysis and Applications35(1),
1–21 (2014). Read the authors' preprint introduction and objective definitions:
https://arxiv.org/pdf/1303.1754.

The paper distinguishes minimum-fill and minimum-FLOP optimization and gives
an NP-hardness result for the latter. Its squared elimination-width objective
is invariant over perfect elimination orders of one fixed completion. Thus
changing only the PEO of that same filled graph cannot directly reduce that
graph's factorization score. Applying a new PEO to the original graph may,
however, realize a smaller completion.

For this challenge, exact flop scoring remains the admission criterion, not
edge-count reduction alone. The independent exchange argument in experiment0105
uses the existing identity F=n+3m+2t to distinguish a one-edge swap from a
one-insertion/multiple-deletion search. No paper algorithm or fetched code was
copied into the candidate. All search must remain structurally gated and
measured against the two-second cap.

See [0105](../experiments/0105-separator-insertion-repair-screen.md).
