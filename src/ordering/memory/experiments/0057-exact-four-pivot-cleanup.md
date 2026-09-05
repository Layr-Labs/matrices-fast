# Exact four-pivot terminal cleanup with enforced atomic search budgets

Effort: ultra

## Result and provenance

This candidate builds on the promoted public source `fbd1e629725034d39b1cf47d65b143cb2cb0330b` from Layr-Labs/matrices-fast, including the work attributed in that repository's existing experiment notes. The original untouched development run passed all 300 matrices at score **0.8445939430699744**, displayed as **0.844594**, and fill **0.944255**. The official promoted score observed before submission was **0.870755** on a different, hidden corpus. Development and hidden scores are not interchangeable.

The final combined candidate passes the full trusted local Yukon run on all 300 matrices at **0.84419540581772**, fill **0.944012**. Its isolated direct ordering probe has a worst call of **0.987 seconds**. The trusted harness and direct probe agree on every matrix's exact flop counts. These are local results; this note makes no claim about the current submission's hidden acceptance.

Only `src/ordering/` is edited. Existing allowed dependencies are preserved; all additions use the Rust standard library. The final source replaces terminal three-pivot descent with fixed four-pivot subset DP and prevents randomized search primitives from exceeding their existing operation cap. It preserves the earlier removal of narrow corpus-conditioned seed selectors and the allocation-only reuse of nested-dissection scratch maps. No new input cells, external data, clock-dependent stopping, search-chain rounds, or larger work allowances are introduced.

## Why these two changes

The preceding exact-three-pivot candidate passed local checks at **0.8444197800114857**, but submission `d17b89c8-7ca1-4524-8410-82f2de991681` failed the official two-second cap on an unspecified hidden matrix. Its remote dependency/purity, build and sandbox steps passed. No hidden score or instance was exposed. A previous upload failed before evaluation because the service could not fetch the public benchmark manifest; retrying resolved that separate setup error.

The follow-up separates move quality from budget enforcement. Larger exact terminal moves can improve an ordering at which strict adjacent-triple descent stops. Meanwhile, checking a counter only between randomized pivots permits a reset, fixed prefix, or deficiency batch to overshoot it. Correcting those checks does not require guessing the hidden matrix or increasing search effort.

## Exact four-pivot move

Let H be the current filled graph before a contiguous window W of four pivots. For an eliminated subset S of W and a remaining pivot v, let C be the connected component containing v in H restricted to S union {v}. The live neighborhood of v after eliminating S is exactly the union of the original H-neighborhoods of C, minus S and v. An edge after eliminating S exists precisely when a path in H connects its endpoints using only vertices of S internally. This gives the formula and shows that the eliminated set, rather than its order, is sufficient state. A still-live external vertex cannot connect two such components internally.

The kernel precomputes four-bit internal-neighbor unions and external-neighbor cardinalities for all 15 nonempty subsets. For each graph word, a 16-word scratch array computes subset ORs and accumulates cardinalities after masking out W. It does not retain 16 copies of the entire graph or clone the elimination game per permutation.

The DP has 16 states and 32 transitions. A transition eliminating v adds the square of one plus its exact live degree. Reconstruction is deterministic: preserve the original window on an optimal tie; otherwise use lexicographic window positions to break ties among strict gains. Eliminating the same complete window leaves the same residual graph, so the local reduction equals the full-order reduction with prefix and suffix fixed.

One complete cycle visits offsets 0, 1, 2 and 3 with stride four. All offsets share the existing 128M ordinary or 48M extended terminal allowance and the existing gate. Evaluation is conservatively precharged at `256*ceil(n/64)+4096` per window, covering subset-word work, masks, DP, component closures, ties and reconstruction. Setup, resets and elimination replay are also precharged. If work expires, completed strict improvements are retained with the remaining permutation unchanged. This is a replacement of the final cleanup phase, not an additional phase.

## Atomic operation-cap enforcement

The randomized search keeps its existing nominal budgets and its existing `hard_cap = budget + budget/4`. Before resetting the graph, replaying each fixed pivot, computing each deficiency, or eliminating a selected pivot, it now checks whether that primitive's existing charge fits the remaining cap. It retains the existing exact charges rather than charging the conservative reservation itself.

Degree lookup and candidate collection are guarded by `4*n+8`. The scan path charges at most `4*n`; the bucket path charges at most `n+2` for bucket advancement plus `2*n+4` for one collection/reservoir traversal. Reset costs `2*n*w+8*n`, deficiency at degree d costs `(d+1)*(2*w+4)`, and elimination costs `(d+1)*(3*w+6)+24`, with `w=ceil(n/64)`.

Both outer search loops break if an attempted run makes no counter progress. This matters when even a reset cannot fit: returning without changing the counter must not cause the same attempt to repeat forever. The LNS incumbent is restored before that break. Abandoned partial trajectories never replace the saved best complete order.

For valid graph/live-prefix invariants and a nonnegative cap, these checks keep the inspected Game counter at or below the configured cap. This is deliberately a scoped claim: Game construction allocations, other ordering stages, and total wall time are not bounded by that counter. Official runtime validation remains necessary.

## Experiments and exclusions

All full-corpus probes below used the same 300 public matrices and were serialized to avoid campaign compiler/benchmark contention. Lower score is better.

| Variant | Exact dev score | Worst local call |
|---|---:|---:|
| Original promoted source | 0.8445939430699744 | 0.962 s |
| Three-pivot submitted candidate | 0.8444197800114857 | 0.952 s |
| 25% fewer later subtree blocks | 0.8446483165442094 | 0.938 s |
| Atomic guards alone, retaining triples | 0.8444141597768596 | 0.940 s |
| Four-pivot replacement alone | 0.844201026052346 | 0.950 s |
| Final four-pivot plus atomic guards | 0.84419540581772 | 0.987 s |

Reducing later block counts preserved their per-block budgets, seeds and ranking but worsened 37 matrices and improved only nine. Total measured ordering time barely changed, from 77.3808 to 77.2736 seconds. That variant is excluded.

Atomic guards alone improved one matrix and left 299 flop counts unchanged. The four-pivot replacement improved 42, worsened 14 and left 244 unchanged against triples. Its large-matrix bucket was unchanged. The final combined run, rather than an arithmetic combination of separate scores, is the authoritative local measurement. Against the original public baseline it improves 62 matrices, worsens 13 and matches 225. Both alternating name-sorted corpus halves improve (deltas −0.00049163 and −0.00030698); removing the five largest relative wins still improves the score by 0.00019579.

## Verification and reproduction

The combined candidate passed **36 active candidate tests**. The original triple mathematics already had independent exhaustive and filled-prefix checks. For the new four-pivot formula, a separate standard-library Python set-elimination reference checked 5,396 windows and 172,672 transition widths, including all 1,024 five-vertex graphs and fixed-seed states with prefix fill. Every four-window order was compared with brute-force elimination. A seven-vertex synthetic witness has cost 66 with no strictly improving adjacent triple, yet one four-window reduces it to 59. This witness establishes a new reachable move, not a corpus-score claim.

Production Rust tests use a separate explicit-clique Boolean-graph oracle, covering 1,084 windows, 34,688 transitions and 26,016 permutations. They include filled prefixes, cross-word vertex indices, tied optima, deterministic strict-gain ties, the witness, invalid inputs and completed improvements surviving budget exhaustion. Budget tests cover resets, nonzero fixed-prefix caps, deficiency work, tiny budgets, no-progress termination and a 1603-vertex case using degree buckets. The new counter regression fails on the former implementation, where a zero cap still charges 80 operations on an eight-vertex graph.

Reproduce from this submission's source using the repository's unchanged setup and sandboxed candidate-build scripts: run `yukon setup`, then `yukon run`. Candidate unit tests and the ignored `probe_timing_and_score` should also be built and run inside the repository's candidate sandbox, never as an unrestricted candidate build. The direct probe emits exact per-matrix COUNTS plus every ordering-call duration. Aggregate bucket geometric means with weights 0.3/0.3/0.4; fill remains the tie-break metric.

The environment used Yukon v2026.09.04-1, Rust/cargo 1.96.0, cargo-deny 0.20.2 and Git LFS 3.7.1. The development corpus SHA-256 was `faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`. Timing was measured on the same local machine as the baseline, with no competing campaign benchmark. The hidden grader is different hardware with a strict two-second worker limit.

## Remaining uncertainty and further research

Larger exact moves cost more per visited window, and reducing a search budget can alter which later rounds run. Neither nominal work reductions nor a local sub-second maximum prove hidden-corpus timing safety. A true matrix-wide ledger and deterministic allocation based on observed marginal gains remain useful research directions. Larger DP windows should be tested as replacements under fixed work, with paired corpus results and isolated timing. External Pro research has been requested for future directions; no returned Pro result contributed to this implementation.
