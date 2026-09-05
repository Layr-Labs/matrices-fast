# Minimal fill inside a given elimination ordering

Heggernes and Peyton, [*Fast Computation of Minimal Fill Inside A Given Elimination Ordering*](https://epubs.siam.org/doi/10.1137/070680680), DOI `10.1137/070680680`.

Primary-source check: 2026-09-06. The accessible publisher abstract supports the direction below; full-paper algorithm and complexity details were not verified. This page makes no claim to reproduce that algorithm.

The useful idea is to start from the fill produced by an existing heuristic and seek a minimal fill subset inside it. This creates a refinement problem over an already available completion, rather than requiring a new ordering method from scratch. “Minimal” is an inclusion condition; it does not mean globally minimum fill or minimum arithmetic work.

For this benchmark, the objective is `F = sum (live_degree + 1)^2`. Our independently proved connection is narrower than a general equivalence between fill and FLOPs: every perfect elimination ordering of a chordal graph `T` has

`F(T) = |V(T)| + 3|E(T)| + 2·triangles(T)`.

Consequently a strict nested chordal completion `H ⊆ T1 ⊂ T0` has lower completion FLOPs than `T0`. A verified perfect elimination ordering of `T1`, applied to `H`, gives an actual completion contained in `T1`, so its score is no larger. This supplies a certificate for a particular refinement family; unrelated fill-reducing changes can still increase FLOPs. The identity and its application here are our derivation, not a claim from the publisher abstract.

On a residual core, `H` must include every edge created by the fixed reduction prefix. Protecting only edges induced from the original graph invalidates the certificate. A valid improvement lifts with its unchanged prefix cost, but must still beat the completed whole-graph incumbent. In the measured experiment, protected erasure improved 46 selected core orders yet only one final public result; the existing window-cleanup controls dominated it pointwise. That negative result limits the present implementation, not the paper's general direction.

The audited trial used cores of 5–512 vertices, at most 8,192 directed entries and maximum degree 64, with 8M logical units, two scans and 64 deletions. It rejected initial completions above 32,768 undirected edges after metered construction. Low input degree does not prevent dense fill, and that postconstruction cap does not bound work already spent. The full solver still has a two-second limit and 4 GiB cap; exact scoring, reconstruction and whole-graph acceptance require separate accounting and native validation.

See the [refinement technique](../techniques/residual-core-refinement.md), [experiment 0065](../experiments/0065-returned-pro-and-core-cleanup.md), and [edge-erasure literature note](culbertson-guralnik-stiller-edge-erasures.md).
