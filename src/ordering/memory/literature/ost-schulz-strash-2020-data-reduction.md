# Data reduction before nested dissection

Lara Ost, Christian Schulz and Darren Strash. *Engineering Data Reduction for Nested Dissection*. 2020. [arXiv:2004.11315](https://arxiv.org/abs/2004.11315).

## Relevant ideas

The paper combines simplicial-node, indistinguishable-node and twin reductions with path compression, degree-two elimination and triangle contraction. Reducing the graph before ordering can improve both partitioning time and the resulting fill. The guarantees and equivalence notions differ between rules; the paper's main optimization target is minimum fill, not this challenge's squared-column FLOP score.

## Mapping to the contract and caps

All these reductions use graph structure and can preserve the pattern-only deterministic contract. A lifted ordering still needs exact FLOP scoring before admission. Unbounded neighbor-pair scans or dense contraction states can violate the 2-second/4-GiB limits on KKT hubs, so a bounded-work implementation and density gates remain necessary.

The inherited `core_lift.rs` already provides an exact elimination-prefix/core split. [Experiment 0098](../experiments/0098-clique-floor-and-minl-epochs.md) examined complementary core orderings but retained none of those experiments. No new contraction or nested-dissection family was implemented from this paper.

See [nested dissection](../techniques/nested-dissection.md) and the [minimum-FLOPs distinction](luce-ng-2013-minimum-flops.md). The source was used for mathematical ideas, not copied code.
