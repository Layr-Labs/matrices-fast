# Residual-core refinement after a fixed elimination prefix

## Exact contract and implementation

An exactly replayed prefix `P` leaves a residual graph `H` and a fixed squared-width cost `C(P)`. Every residual permutation `Q` satisfies `F(G,P++lift(Q)) = C(P)+F(H,Q)`. Thus core orders can be compared on `H` while keeping the prefix fixed. Different prefixes require their own accumulated costs and residual graphs.

The terminal degree-three reducer in source `2a28517...` selects among four AMF configurations and default AMD, lifts its single best core order, and compares it with the completed whole-graph incumbent. Because this phase follows whole-graph cleanup, its winner can still benefit from an existing local optimizer applied directly to the smaller core.

The selected September 6 extension adds one k5→k4 cycle for residual dimensions 5…4096 and directed nnz≤65536, at 16M logical units per kernel. Retain the original candidate first; k4 receives an accepted k5 result. Validate each permutation and exact core score. Before final strict acceptance, verify that accumulated prefix cost plus refined core cost equals the exact whole-graph score. Refusal leaves the completed incumbent available.

## Boundaries for alternative core methods

Reordering perfect elimination orders of one fixed chordal completion cannot improve its squared-width objective; a successful method must change the completion. The tested eraser constructs a completion of `H`, protects **all** edges of `H`, and only deletes admissible completion-only edges. Prefix-created fill edges are required residual edges, not removable extras.

For a current chordal graph, deleting an edge whose common neighborhood is a clique preserves chordality. The cited exposed-edge paper's definition requires that neighborhood to be **nonempty**. The empty-common case is a separately justified bridge extension: in a chordal graph such an edge is a bridge, and deleting a bridge preserves chordality. For either case the completion's squared-width reduction is `3+2|N(u)∩N(v)|`, derived through the edge/triangle identity. This local certificate does not replace validation of the lifted permutation or acceptance against the completed solver.

## Costs and measured limits

Bitset construction, replay, resets and candidate scans depend on residual dimension and density. The two per-kernel allowances do not include the entire solver's previous work or prove its wall time. Caller rescoring and simultaneously retained core/portfolio data also matter. Use residual gates and the supplied sandbox/watchdog; original-graph size alone does not describe core cost.

On the 300-case public comparison, the selected cleanup improves six final cases with no losses, score 0.8390208740493201 versus 0.8390631716578836. Fifty active tests and all 300 trusted cases pass. The gain is 0.504105 relative basis points locally; no new official promotion is established here.

The tested eraser improves 46 core orders but only one final whole-graph count, and that count is improved more by cleanup. The causes of all remaining non-gains were not fully classified. One alternative-prefix experiment makes no final gain. These controls demonstrate why core improvement, whole-graph improvement and complementary value must be reported separately.

## Links

- [Experiment 0065](../experiments/0065-returned-pro-and-core-cleanup.md).
- [Heggernes–Peyton minimal-fill refinement](../literature/heggernes-peyton-minimal-fill.md).
- [Culbertson–Guralnik–Stiller edge erasures](../literature/culbertson-guralnik-stiller-edge-erasures.md).

## Revalidation on the advanced frontier

[0066](../experiments/0066-core-cleanup-latest-frontier.md) repeats the selected cleanup on promoted 08a9935, preserving its new k4 heuristic screen. Public score 0.8389332841047089→0.8388909882140252, six gains and no losses, 50 tests and 300 trusted cases pass. Restoring the terminal cached-score assignment is required when adding later acceptance checks. Earlier six-return controls remain scoped to 2a28517.
