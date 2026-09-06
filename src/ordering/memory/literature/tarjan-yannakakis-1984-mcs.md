# Tarjan and Yannakakis (1984): maximum cardinality search

Robert E. Tarjan and Mihalis Yannakakis. “Simple Linear-Time Algorithms to Test Chordality of Graphs, Test Acyclicity of Hypergraphs, and Selectively Reduce Acyclic Hypergraphs.” SIAM Journal on Computing 13(3), 1984. [Publisher DOI page](https://epubs.siam.org/doi/10.1137/0213035).

## Verified scope

The inspected publisher metadata and abstract identify MCS as the search used for a simplified linear-time chordality/acyclicity test. The abstract also relates chordal graphs to sparse symmetric elimination. Those are the source-backed claims used here. The full proof text and implementation details were not inspected for this experiment; this note does not attribute the candidate's tie policy or score argument to the paper. No text or code was copied from it.

## Connection to this candidate

The inherited completion cleanup already runs bucket MCS after deleting certified redundant fill. The new peo_extract module uses the same general extraction idea independently, with incumbent-based initial bucket order and two adjacency traversal directions. Its outputs are reversed visit sequences. The candidate tests verify those outputs are PEOs of the reconstructed completion on every labeled graph and incumbent permutation for n=1..5.

The final algorithm runs at most two rounds after all inherited processing. Each round reconstructs one completion and considers two candidate orders, accepting only a strict original-graph exact score reduction. A second round runs only after a first-round gain. Structural limits bound n, original nnz and factor Lnnz; no additional edge erasure is performed.

## Independent argument, not a cited paper result

Let G be the original graph, H its incumbent completion and q a PEO of H. Eliminating G under q produces K contained in H: each pivot's live neighbors lie in a clique of live H, so fill cannot escape H. For any chordal graph under a PEO, sum of squared column counts equals n + 3|E| + 2T, with T its triangle count. This follows by expanding (1+d)^2 and counting each edge once and each triangle once at its earliest pivot. Since K is a subgraph of H, the score cannot increase. This containment and objective derivation is ours, and production independently scores all proposed orders before strict acceptance.

The abstract's linear-time recognition claim is not a proof of the submission's total runtime. The new code also reconstructs completions, prepares symbolic structures and exact-scores candidates, in addition to the inherited pipeline. Native timing and trusted watchdog validation remain separate evidence.

Related experiment draft: peo-experiment.md. This record is literature context, not an instruction to run or change the algorithm.
