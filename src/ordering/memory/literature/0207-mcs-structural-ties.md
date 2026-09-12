# Maximum cardinality search and structural ties

Robert E. Tarjan and Mihalis Yannakakis, "Simple Linear-Time Algorithms to
Test Chordality of Graphs, Test Acyclicity of Hypergraphs, and Selectively
Reduce Acyclic Hypergraphs," SIAM Journal on Computing 13(3), 566–579,
August 1984. [Primary publisher record](https://epubs.siam.org/doi/10.1137/0213035).

The paper introduces maximum cardinality search as a simplified method for
chordality testing. Its abstract connects chordal graphs with sparse Gaussian
elimination. The publisher record and author-university record were inspected
on September 12, 2026 UTC. No fetched implementation is copied.

The existing solver already extracts reverse MCS visitation orders from a
validated incumbent chordal completion. The new experimental hypothesis is
that fixed structural priorities among equally weighted vertices explore
different elimination orders without changing the maximum-cardinality rule.
This priority policy is our inference and experiment, not a performance
claim attributed to the paper.

The original 0207 experiment measured original degree, completion degree,
incumbent column count and a fixed
position hash are measured as alternative priorities. A standard-library
binary heap chooses the maximum visited-neighbor count first, then the fixed
priority. Stale entries are discarded. There is one initial entry per vertex
and one update per completed edge, so heap work is O((n+E) log(n+E)), rather
than the original linear bucket method. Candidate completions are bounded
to n <= 12k, input nnz <= 200k, factor nnz <= 150k.

Independent exhaustive small-graph completion and PEO checks validate that
implementation. Original-pattern symbolic scoring is the only adoption
criterion. The original experiment is described in
[0207](../experiments/0207-optimization-campaign.md); its heap limits and
cost discussion above describe that historical bounded screen.

The subsequent [0215](../experiments/0215-indexed-mcs-and-lex-bfs.md) replaces
stale heap entries with one eager indexed item per live vertex. It preserves
all twelve structural-policy outputs against the frozen reference and 292
public completion pairs while measuring about 9.6x lower kernel time locally.
Queue work is O((n+E) log n) with O(n) live queue storage. The selected terminal
stage additionally observes original AMD <=1B, dimension <=50k, input <=1.3M
and exact factor <=1M. These current bounds do not revise the historical
experiment, nor establish a wall-clock guarantee; its predecessor still failed
the hidden cap, and full runtime and official validation remain necessary.
