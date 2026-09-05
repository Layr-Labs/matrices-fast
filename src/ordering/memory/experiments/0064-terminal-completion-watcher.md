# Terminal completion watcher on the current matrices-fast frontier

## Context and baseline

This candidate starts from the current promoted source `2a28517ddc00dac41a52c270f9601702df982200`, corresponding to submission `6ef357df-0852-483d-81af-9cb0acb0dedd`. The official benchmark listing reported a score of 0.867211 at the start of this work. AMD is the reference at 1.00, and lower scores are better. These are official existing results, not new measurements of this candidate.

The operator requested direct submission to the official runner with no local benchmark execution. Consequently this session did not execute the candidate locally, run a development-corpus benchmark, or claim a measured score. The installed tools were inspected, the Yukon CLI and its agent skill were updated, and the current challenge was freshly cloned with Git LFS available. GCC, Cargo and cargo-deny were already present. The active challenge has a different repository and benchmark identifier from the older ssi-ordering challenge represented by existing workspaces. The new clone is linked to the active matrices-fast benchmark.

The current source already includes the approved Rust dependency set. With the operator's authorization that existing set is retained. The added graph algorithm uses standard-library containers and receives its symbolic input from the parent's existing approved scoring helpers. No new dependency is declared. The model used for this work is GPT 6 Astra, with Codex as the harness. Attribution is separate from that of the inherited frontier.

## Change and hypothesis

The existing implementation generates many minimum-degree, minimum-fill and separator orderings, performs local search, and finishes with exact low-degree reduction followed by core ordering. The new component operates after that entire pipeline. It reconstructs the chordal completion induced by the finished permutation, attempts to remove redundant fill edges, and obtains another elimination order with maximum-cardinality search.

This changes the search space in a specific way: the new pass searches subgraphs of the incumbent's chordal completion instead of searching additional random labelings or local permutations. A redundant edge need not be removed by any of the bounded permutation moves already tried. Conversely, a completion with no removable edge is inexpensive to abandon under the deterministic work cap. The final exact scorer decides whether the resulting ordering is an improvement.

The terminal placement is intentional. Earlier public experiments describe cases where a better intermediate ordering seeded a weaker downstream local optimum. Here all existing candidate generators and descent seeds stay the same. A candidate is accepted only after the inherited pipeline has finished and only when the full-pattern flop count is strictly smaller. A tie retains the inherited permutation. This establishes objective non-regression for completed runs; it does not establish a runtime guarantee.

## Graph construction

The caller obtains the incumbent's permuted sparsity pattern, elimination tree, and exact symbolic column counts using its existing functions. The new completion module propagates filled-column reach sets from children to parents in increasing elimination order. Original entries below the current diagonal and propagated child entries are deduplicated using a timestamp array. Each reach edge is mapped through the incumbent permutation into the original vertex numbering and inserted into both adjacency rows.

Each reconstructed column is checked against the exact symbolic column count before it is used further. A mismatch declines the entire new candidate, leaving the established ordering available. The total symbolic factor size is checked before allocating the explicit completion. Child reach vectors are released once their parent consumes them, and graph adjacency is sorted before the deletion algorithm begins.

Only edges absent from the original pattern enter the removable edge list. That list is generated in lexicographic endpoint order, which allows allocation-free binary searches for support edge identifiers. Original edges are never deleted. The initial deletion queue is sorted by endpoint degree sum with the edge identifier as a deterministic secondary key.

## Deletion certificates and event handling

For a chordal graph, an existing edge can be removed while retaining chordality when its common-neighbor set is a clique. The deletion test computes that set using sorted adjacency, then verifies all required adjacencies. A failed clique test produces two nonadjacent common neighbors. Together with the tested edge's endpoints they form a four-cycle whose unique chord is the candidate edge.

That four-cycle remains a certificate against deletion until one of its four supporting edges disappears. The watcher records reverse links only for supporting fill edges; original supporting edges are permanent. Each candidate owns four intrusive watcher nodes. Deleting a support wakes affected candidates through those reverse lists, and a queue-membership flag avoids duplicate work. When a candidate is retested, its stale links are detached before a new certificate is installed.

The watcher implementation comes from the earlier local ssi-ordering work in this workspace. Its source was inspected before reuse; the old benchmark's absolute scores are not transferred to this challenge or treated as current measurements. The new integration and single-pass completion reconstruction are specific to this candidate. Existing watcher self-tests are retained as source, but were not executed locally in this session.

## Explicit limits

The added caller gate is `16 <= n <= 30000` and `nnz <= 180000`. The module additionally caps the exact symbolic factor size at 400000 entries and the removable fill-edge list at 145000 undirected edges. The watcher receives 20000000 deterministic work credits and a common-neighbor width cap of 1024. It stops when its credits are exhausted. Every successful deletion before that stop has already passed its certificate test, so an interrupted search still supplies a well-defined candidate graph.

The limits are structural and apply equally to every input satisfying them. There are no matrix names, fingerprints, stored permutations, per-instance exceptions, clocks, environment reads, filesystem access or external data dependencies. All ordering decisions are functions of the given sparsity pattern and fixed constants. The new pass is sequential and does not introduce additional concurrency beyond the inherited implementation.

The principal cost outside the deletion ledger is symbolic setup, sorting, and linear graph storage. Those costs are bounded by the dimension, original nonzero and symbolic-factor gates. The operation budget is a logical work bound rather than a calibrated number of seconds. The official two-second cap still decides feasibility, and this unmeasured candidate may fail that cap.

## Acceptance and verification

Maximum-cardinality search uses deterministic integer weight buckets and returns reverse visitation order. The caller checks the resulting permutation as a bijection and scores it with the same full-pattern symbolic routine it uses for all other candidates. Only a strict flop improvement replaces the existing permutation. No score supplied by the watcher is trusted.

Static verification consists of source inspection, changed-path inspection and `git -c core.whitespace=cr-at-eol diff --check`. The whitespace setting acknowledges the inherited CRLF line endings without rewriting the entire main module. No local build, benchmark, timing measurement or test execution was used to choose or claim this candidate's score. The submission omits a claimed score because the benchmark reports that claimed scores are recorded only.

All changes are under `src/ordering/`: the caller insertion in `mod.rs`, the new `completion.rs` adapter, the reused `minl_watch.rs`, this experiment record, and memory links. The harness, scoring crate, purity gate, dependency declarations and corpus are unchanged.

## References and next decision

The algorithmic context is Blair, Heggernes and Telle, “A practical algorithm for making filled graphs minimal,” Theoretical Computer Science (2000), https://www.sciencedirect.com/science/article/pii/S0304397599001267 . The paper studies removing excess fill from a given chordal completion. The implementation here uses local clique certificates and event-driven retesting rather than copying code from the paper or a downloaded implementation.

After submission, the official aggregate result determines whether this candidate beats the promoted frontier. A timeout requires reducing the structural envelope or deletion work; a successful tie would show that the extra completion search did not find sufficient aggregate value at this budget. No hidden matrix identities or hidden-corpus reconstruction are required for either decision. Until that result arrives, improvement and runtime remain unmeasured hypotheses.
