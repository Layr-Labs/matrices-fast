# Exact sparse updates and retaining proven pair improvements

This candidate is prepared for one transparent organizer assessment. It makes
no prediction about the hidden score and does not claim a public-development
improvement. It is based on the current source commit
`649c230a5d60e2122263bd3b653e508938e9fca2`, preserving that source's exact
four-pivot terminal cleanup, triple helper, atomic randomized-search budget
checks, no-progress exits, leaf-map reuse and removal of corpus-specific seed
selectors. It does not overwrite those improvements with an older candidate.

Both the untouched source worker and this candidate passed the complete trusted
300-matrix public benchmark, including the original two-second watchdog. The
score files report **0.844195 before and 0.844196 after**, with fill **0.944012**
for both. Reconstructing the same weighted score from the exact per-matrix flop
rows gives **0.8441954058177199 before and 0.8441955206763234 after**: a regression
of approximately **0.0000001148586**, or **0.00136 relative basis points**.
The displayed six-decimal scores straddle a rounding boundary. Exactly one row
changed: `rsyn0840m04m` increased from 209807 to 209817 flops; the other 299 flop
counts were unchanged. This is a small measured regression, not an improvement.

The reason to assess the candidate is a general correctness-supported mechanism:
a complete ordering containing a proven improving adjacent pair can be retained
when a local simulation budget expires. Previously, that valid result was
sometimes discarded. The accompanying changes avoid redundant graph work and
keep cached incumbent costs consistent. The local mechanism is supported by
proof and exhaustive tests; its value on the organizer's unseen problems is
unknown. The official evaluation should decide competitiveness.

## Exact identity and permitted scope

The production-source tree SHA-256 is
`751f30b42ff7fe91683678cdfffb2a948112c2b90cef2bc95fc5ef5f98f350f2`.
The digest hashes canonical sorted JSON mapping each relative production path
under `src/ordering/` to its file SHA-256, excluding research notes. The tested
candidate worker SHA-256 is
`584dd20eda21da236bdf350e244b202fc1e30ffb3e073929cc960ab852d84e6f`.
The saved untouched 649c230 worker SHA-256 is
`17b70a6ca77411ab7c341fff04619f7a39d1c235fcb90c695ae4cdd996f5d3d2`.
The complete source, worker copies, per-file manifest, logs and scores were
frozen and preserved; source did not change between the paired runs.

Only `src/ordering/` is changed. No dependency, harness, scoring wrapper, corpus,
sandbox configuration or watchdog was changed. The new code adds no matrix
identity selector, fixed permutation, external data, environment read, clock
read, filesystem access or network access. No hidden evaluation corpus was
accessed, fingerprinted or used to choose parameters. The existing seed policy
on 649c230 is preserved exactly, with no added salts, rounds or trajectories.

The production delta consists of three independent changes: exact sparse-word
fill updates; paired incumbent cost assignments with safe cached terminal
scoring; and immediate retention of proven adjacent-pair swaps in the existing
earlier pair-descent passes. The terminal four-pivot algorithm and its work
ledger are unchanged. The old source's earlier research variants and their
scores are historical and are not the source or measurements offered here.

## Exact sparse-word elimination

The fill game maintains `deg[u] == popcount(adj[u])` for every live vertex. A
pivot row's zero words cannot add fill to any neighboring row. The candidate
records the nonzero word indexes while materializing that pivot's neighbors.
When fewer than half the words are active, each neighboring row starts its
accumulator from the exact existing degree and adds only newly inserted bits:
`popcount(pivot_word & !old_row_word)`, at those active word indexes. Dense pivot
rows keep a complete union and recount, expressed through zipped slices.

The common code still removes the neighbor's self bit and the eliminated pivot
bit, then subtracts two from the accumulated degree. Both are present before
removal: the pivot row contains the neighbor, and the neighbor row contains the
pivot. This gives the same final adjacency and degree as the full-row algorithm.
Boundary vertices in partial games are updated exactly like other live vertices,
but remain ineligible as pivots. Bucket updates and their ordering are unchanged.

The existing deterministic operation charges are preserved, including the
`elimination_ops` calculation consulted by the new frontier's atomic budget
checks. The candidate does not spend saved machine time on extra search. It
reduces the number of neighbor-row word visits in the sparse branch; no universal
wall-time speedup is claimed. The additional index vector is bounded by the
adjacency word width, and the baseline game size limits are unchanged.

## Consistent incumbent costs and cached scoring

The inherited second and third subtree acceptance sites assigned an improved
permutation to `best_perm` without assigning the corresponding measured value
to `best_flops`. Later comparisons could therefore use a stale cost. This
candidate assigns `best_flops = f2` and `best_flops = f3` at those two sites.
Every preceding incumbent assignment was reviewed for the paired-state invariant.

With the cost cache consistent, the independent terminal subtree pass can use
`best_flops` instead of recomputing the exact cost of the same `best_perm`.
This avoids one redundant canonical scoring call. The subsequent four-pivot
terminal cleanup, its exact objective checks, its gates and its shared work
allowance are preserved from 649c230. The complete-program comparison does not
attribute the one changed public row to a particular component; no additional
component-isolation result is claimed.

## A proven pair choice remains a complete valid ordering

Before a consecutive adjacent pair a,b is eliminated, let its current degrees
be da,db. Eliminating both vertices in either order leaves the same residual
fill graph. The second pivot has the same column count in the two orientations:
its neighborhood comes from the same union of neighbor sets, with the pair
removed. The first pivot's column count is its current degree plus one.
Therefore db < da makes the orientation b,a strictly better for the pair's sum
of squared column counts. The suffix then sees the same residual graph, so the
complete ordering with its existing suffix is strictly improved.

The old pair routine assembled a separate `next` buffer and returned no candidate
at every budget exit. This could discard useful swaps in an interrupted first
sweep, or complete improvements from preceding sweeps. The candidate immediately
swaps the two positions in `cur` when the strict inequality is proven. `cur`
remains a complete bijection, with its untouched suffix already present. If the
budget expires, it returns that ordering only when some proven improvement was
made. There is no additional elimination or suffix-copy pass at the exit.

Even an exit after simulation of only the first pivot of an already-selected
pair is safe. The cost argument establishes the complete pair choice before
simulation; finishing the simulation is not necessary for the returned
permutation to be valid. Invalid input and exits before any improving choice
still return no candidate. With sufficient budget, pair choices and full-sweep
results remain the same. Existing operation charges and stopping comparisons
are unchanged. This does not broaden the frontier's atomic randomized-search
budget policy; that separate code is left intact.

A small helper-level witness is a four-vertex star with center 0 and seed
[0,1,2,3]. Its column counts [4,3,2,1] cost 30 flops. The pair choice [1,0,2,3]
has column counts [2,3,2,1] and cost 18. With the helper's budget set to 40,
initialization fits; the first leaf elimination raises the same inherited work
counter to 82 in either implementation. The old version discards its result;
the new version returns the complete cost-18 ordering. This is a proof witness
for the budgeted helper, not a claim that `order()` uses a 40-operation budget
for small production matrices. It is included in the exhaustive test family,
not special-cased in the code.

The guarantee is local to descent relative to its input ordering. Accepting an
improved incumbent can change subsequent heuristic paths, so it is not a proof
that the whole program improves every matrix or an aggregate score. Callers
continue to compute canonical flops and accept only strict improvements. The
measured public regression above is reported without hiding that limitation.

## Verification against the new frontier

The untouched 649c230 source was built through the official candidate sandbox
and saved before any edit. The candidate was then built and tested under the
same sandbox. All **39 active worker tests passed**, with **17 intentional
ignored probes**, zero failures and zero filtered tests. This includes the
upstream triple/four-window mathematical oracles, atomic-budget/no-progress
tests, ND reference tests and the additional tests below. The suite completed
in 2.84 seconds; that unit-test duration is not an ordering-performance claim.

The pair boundary test covers all 64 simple undirected four-vertex graphs, two
seed orders and every integer budget from 0 through 700, for **89,728 calls**.
Three complete sweeps need at most 678 charged operations on those graphs, so
this spans every stopping boundary. An independent boolean fill graph checks
permutation validity, strict cost improvement when a result is returned,
non-worsening quality as budget increases and equality with unrestricted
full-sweep reference permutations. Invalid input remains rejected.

Exact elimination tests cover dimensions 1,2,63,64,65,129,257 with chain,
irregular sparse and dense patterns. They compare every adjacency entry, live
degree, returned column count and work increment after each elimination.
A separate n=1537 test exercises bucket selection and links, cross-word fill,
whole and partial games, permanent boundary vertices, minimum degree and reset.
Only 65 pivots are eligible in its partial case; sparse references bound test
work without a dense cubic large-graph oracle.

Both full public runs passed all 300 fixtures under the trusted repeated
bijection/determinism checks, purity/license gate and unchanged two-second cap.
They ran sequentially using their frozen workers, with no concurrent campaign
compiler or benchmark. Unrelated user workloads were left untouched. The score
files report these rounded bucket values:

| Metric | Untouched 649c230 | Candidate |
|---|---:|---:|
| Weighted flop ratio | 0.844195 | 0.844196 |
| Weighted fill ratio | 0.944012 | 0.944012 |
| Small flop geomean, 147 matrices | 0.890493 | 0.890493 |
| Medium flop geomean, 108 matrices | 0.867553 | 0.867554 |
| Large flop geomean, 45 matrices | 0.791954 | 0.791954 |

The single 10-flop row regression and the exact weighted reconstruction are
reported at the start. Equal rounded metrics do not imply byte-identical
permutations across the two versions; only each version's repeated determinism
is certified by the full harness. Older runtime-only diagnostic comparisons
are not substituted for this new-base paired evaluation.

## Interpretation and limits

This is a general implementation candidate with a concrete budget-boundary
benefit and a nearly unchanged, slightly worse public score. It makes no hidden
score prediction, record claim, reward claim or assertion of organizer acceptance.
The official promotion threshold remains the organizer's decision. The public
number is not supplied as a hidden score, and no locally inferred hidden matrix
or score was used to construct the candidate.

The local harness does not enforce the organizer's four-GiB address-space limit,
so the successful local run does not independently certify that limit. The game
size bounds are preserved, the sparse index vector is small, and the pair change
removes one permutation buffer. Timing can vary between hosts; fewer word visits
and one cached scorer do not prove a universal runtime improvement or future
watchdog success. The candidate adds no new search allowance.

OpenAI Codex developed this delta with bounded agent collaboration and independent
source review. The exact sparse-degree and pair-order invariants, compatibility
with the promoted frontier, and paired incumbent assignments were reviewed before
qualification. Contributed research Markdown remains subject to the organizer's
normal human review before merge or use. This note documents the tested current
649c230-based candidate only, prepared for one transparent official assessment.
