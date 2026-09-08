# Minimum fill and minimum FLOPs are different

Robert Luce and Esmond Ng. *On the minimum FLOPs problem in the sparse Cholesky factorization*. 2013. [arXiv:1303.1754](https://arxiv.org/abs/1303.1754).

## Relevant result

The paper constructs graph families for which no ordering simultaneously minimizes fill and arithmetic work, and establishes NP-hardness of minimum FLOPs. Fill is proportional to the sum of elimination column counts; the FLOP objective is the sum of their squares. Minimizing the former is not a certificate for minimizing the latter.

## Mapping to this challenge

The pattern-only elimination game and squared-column objective match the symbolic objective ranked here. This does not address numerical pivoting or cancellation in actual indefinite factorization. A minimum-fill candidate is useful only after the exact symbolic scorer checks it. A globally exact algorithm is not suggested for the 2-second cap.

[Experiment 0098](../experiments/0098-clique-floor-and-minl-epochs.md) tried more FLOP-aware core/completion priorities without an end-to-end gain. Its retained clique-floor bound instead prunes impossible completions of the existing bounded elimination game, including live boundary vertices. The bound is derived and tested in that experiment; it is not quoted as a theorem of this paper or claimed Lean-verified.

See [best-of portfolio](../techniques/best-of-portfolio.md) for the distinction between an AMD floor and regression freedom relative to a previous fixed-budget search.
