# Edge erasures and chordal graphs

Culbertson, Guralnik and Stiller, [*Edge Erasures and Chordal Graphs*, arXiv:1706.04537v2](https://arxiv.org/pdf/1706.04537v2). Primary-source check: 2026-09-06.

The paper's exposed-edge condition requires a **nonempty clique** common neighborhood: for edge `uv`, `N(u) ∩ N(v)` must be nonempty and complete. Definition 5, Lemma 7 and Theorem 8 concern erasures that preserve connected chordality. Omitting “nonempty” changes this contract.

For our chordality-only certificate, we independently proved that deleting an edge of a chordal graph preserves chordality exactly when its current common neighborhood is a clique, allowing the empty set. Nonadjacent common neighbors would form a new chordless four-cycle after deletion. Conversely, a newly induced cycle would have the removed edge as its only former chord; a longer cycle would already contradict chordality on one side of that chord. The empty-common-neighborhood case is a bridge deletion and can disconnect the graph. It is our extension, not the paper's exposed-edge definition or connectedness guarantee.

Our separate FLOP derivation makes the certificate useful for elimination ordering. Every perfect elimination ordering of a chordal graph `T` costs

`F(T) = n + 3m + 2·triangles(T)`.

Deleting a certified edge with common neighborhood `C` therefore saves exactly `3 + 2|C|` in that completion. The squared-width identity and decrement are not attributed to the paper. Eligibility must be recomputed after each deletion: disjoint edge endpoints do not make cached certificates independent.

For a residual graph `H`, protect **all** its edges, including fill created by the fixed prefix, while editing a completion `T ⊇ H`. Reconstruct and verify a perfect elimination ordering of edited `T`; replaying it on `H` produces a completion contained in `T`. A successful certified deletion thus strictly improves the original completed core order. Removable fill edges have endpoint paths in protected `H`, so they cannot be bridges in this application. Finally add the exact prefix cost and compare against the completed whole-graph incumbent; a local certificate does not guarantee a new portfolio winner.

The native research trial capped the core at 512 vertices, 8,192 directed entries and maximum degree 64. It used an 8M logical-work allowance, at most two scans and 64 deletions, and rejected completions exceeding 32,768 undirected edges after metered replay. At most three 512-by-8-word bit matrices need coexist: 98,304 payload bytes plus bounded scratch. Initial completion can still be dense; construction and refused work remain charged. These limits do not themselves prove compliance with the full solver's two-second or 4 GiB limits, and successful lifting incurs a whole-graph score outside the core ledger.

The experiment found real core improvements but no additional pointwise benefit over the cheaper cleanup controls on the 300 public patterns. Keep the certificate and implementation as research evidence; increasing its caps is not supported by that result. A bounded diagnostic of prefix-plus-core lower bounds is a better next step than assuming every improved core offers final score headroom.

See the [refinement technique](../techniques/residual-core-refinement.md), [experiment 0065](../experiments/0065-returned-pro-and-core-cleanup.md), and [minimal-fill literature note](heggernes-peyton-minimal-fill.md).
