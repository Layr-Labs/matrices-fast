# Experiment 0043: four deep blocks for the small-matrix subtree search

## Status

Rejected after submission. The exact source passed the official build, purity,
license, permutation, determinism, timeout, and full 300-matrix development
corpus path at `0.848766` with fill tiebreak `0.947446`, but submission
`794e1fc0-d883-465e-9e22-dd7cc51a6852` exceeded the two-second cap on one
hidden matrix. Experiment 0044 adds the density gate derived from that result.

## Context

The promoted frontier at commit `223023b` introduced a size-specific
`max_s = 128` cap for the first subtree-refinement round on matrices with
`1,000 <= n < 10,000`. Its public development score is `0.848955`, and its
official hidden score is `0.874307`. That configuration is retained exactly
for the medium bucket. The large-matrix configuration is also retained
exactly. This experiment changes only the work allocation used when
`n < 1,000`.

The subtree refinement has a requested-work ceiling expressed as
`max_blocks * budget * streams`. The frontier's small-matrix setting was:

```text
min_s      = 16
max_s      = 256
max_blocks = 16
budget     = 2,000,000
streams    = 1
```

The resulting ceiling is 32 million word operations. The candidate keeps
that ceiling unchanged but reshapes it as four searches with eight million
operations each:

```text
min_s      = 16
max_s      = 256
max_blocks = 4
budget     = 8,000,000
streams    = 1
```

This is a reallocation, not an increase in requested work. It is selected by
the structural `n < 1,000` size gate. No matrix names, corpus identities,
precomputed permutations, values, right-hand sides, or answers are used.

## Hypothesis

The small bucket has short elimination trees and relatively few subtree
blocks worth searching. With sixteen blocks, a two-million-operation search
often stops before the most valuable blocks converge. Starting additional
low-ranked blocks has little value when the tree does not contain many useful
independent regions. Spending the same total work on four deeper searches
should allow the best-ranked blocks to finish more of their local search and
produce a stronger incumbent.

This hypothesis is deliberately restricted to small graphs. Applying deeper
per-block budgets to the medium and large tiers can produce expensive
downstream refinement basins and is not safe under the two-second hidden
per-matrix cap. Those tiers therefore remain on the already promoted
configuration.

## Sweep

The focused `probe_lt1k` test scores all 147 development matrices with
`n < 1,000`. Each point below keeps `max_s = 256`, `min_s = 16`, one stream,
and at most 32 million requested operations:

| shape | requested ceiling | small-bucket flop geomean | measured worst |
|---|---:|---:|---:|
| 32 blocks x 1M | 32M | 0.894256 | 0.542 s |
| 16 blocks x 2M, promoted base | 32M | 0.893893 | 0.521 s in the base probe |
| 8 blocks x 4M | 32M | 0.893854 | 0.528 s |
| **4 blocks x 8M** | **32M** | **0.893262** | **0.659 s** |

The curve is not merely a lottery at the final point. Moving from 32 shallow
blocks toward fewer deeper blocks first recovers the promoted base, then a
small additional improvement at eight blocks, and finally a substantially
larger improvement at four blocks. The selected point improves the small
bucket by `0.000631` against the promoted 16-by-2M configuration.

A separate `min_s = 8` test at 16 blocks by 2M scored `0.893877`. That result
is slightly better than the promoted base but clearly worse than four blocks
by 8M, so the candidate retains `min_s = 16`. Lowering the global
`SUBTREE_MIN_N` from 64 to 32 was also score-neutral (`0.893893`) with a small
timing regression and was reverted.

## Full-corpus result

The exact candidate was built with the repository's sandboxed worker build
and evaluated by the official local harness over all 300 matrices. The final
row written by the harness is:

```text
OK    score=0.848766    fill=0.947446
```

The rounded bucket report was:

| bucket | count | flop geomean | fill geomean |
|---|---:|---:|---:|
| `lt_1k` | 147 | 0.8933 | 0.9621 |
| `1k_10k` | 108 | 0.8734 | 0.9589 |
| `gt_10k` | 45 | 0.7969 | 0.9278 |

The overall public score improves from `0.848955` to `0.848766`, an absolute
reduction of `0.000189`, or about 2.2 basis points relative to the frontier.
The medium and large bucket values match the promoted algorithm; the score
movement comes from the intended small tier.

The focused small-bucket probe completed all 147 matrices in 21.1 seconds and
reported a worst `order()` duration of 0.659 seconds on
`multiplants_stg1b`. The full official corpus run also completed without a
timeout on an otherwise idle arm64 host with 16 logical CPUs and 64 GiB RAM.

## Runtime investigation and rejected alternatives

Runtime was treated as a hard gate rather than inferred from the nominal work
ceiling. During this experiment, a broader 16-block-by-2M allocation for
`n >= 1,000` looked strong in bucket probes but failed the official run on
`batchs121208m` at at least 2.1 seconds. That configuration was rejected.

A second attempt restricted the change to large matrices and restored later
chain rounds to one-million-operation budgets. It entered a poor downstream
basin: the large-bucket score barely moved (`0.796916` to `0.796834`) while
`unitcommit_200_100_1_mod_8` took 8.981 seconds in the probe. It was also
rejected. Neither rejected configuration is present in this candidate.

One full run of the final small-only source on the primary workstation was
also killed on an unchanged medium-tier matrix while the workstation was
under substantial unrelated CPU contention. A source comparison against the
promoted commit confirmed that the matrix did not enter the changed branch.
The same exact tree then completed the official run on the idle verification
host. This is recorded rather than hidden because local wall time under host
contention is not an algorithmic pass. The hidden grader remains the final
runtime authority.

The small-only candidate has a materially stronger safety case than the
rejected broad changes:

1. It runs only for `n < 1,000`.
2. Its measured changed-path worst case is 0.659 seconds.
3. Its requested-work ceiling remains exactly 32M.
4. Medium and large code paths are the promoted, hidden-accepted paths.
5. The official full run completed on the idle verification host.

## Correctness and rule compliance

- Only `src/ordering/` is changed.
- The implementation uses the Rust standard library and the challenge's
  existing allowed modules; no dependency or manifest changes are made.
- `order()` still receives only `&Pattern` and returns a permutation.
- The best-of portfolio accepts a candidate only after a strict flop
  improvement, so the new search cannot knowingly replace the incumbent with
  a worse ordering on a matrix.
- The unit suite checks bijection, determinism, empty and singleton cases,
  the AMD floor, fixture improvement, and the configured work limit.
- The full scorer independently validates output permutations and recomputes
  factorization cost.

Verification commands and results:

```text
cargo test -p ssi-candidate-worker --release probe_lt1k -- --ignored --nocapture
  PASS: 1 passed; small geomean 0.893262; worst 0.659 s

cargo test -p ssi-candidate-worker --release
  PASS: 25 passed; 0 failed; 16 ignored

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: full 300-matrix corpus; score 0.848766; fill 0.947446
```

The full build path includes the required purity and `cargo-deny 0.20.2`
license checks. The compiler reports four pre-existing dead-code warnings;
there are no build errors or test failures.

## Interpretation

The result reinforces a size-dependent view of bounded subtree search. The
important parameter is not only the total work budget or the maximum subtree
size. The shape of the allocation determines whether the algorithm spends
most of its time starting searches or lets a few promising searches reach a
useful depth. For short elimination trees, four deep searches dominate a
larger collection of shallow searches at the same nominal cost.

The change is intentionally narrow because the same intuition does not carry
safely into larger tiers. Medium and large graphs can form a different
incumbent after the first refinement; that incumbent changes the next
elimination tree and can activate expensive later rounds. The small tier has
both a structural dimension bound and direct timing evidence, making it the
appropriate place to capture this gain without repeating the hidden-timeout
failure mode seen in broader experiments.

## Next work if promoted

The next safe axis is not to deepen the medium or large chain further. Those
experiments have already demonstrated nonlinear timeout risk. Follow-up work
should either improve candidate selection at constant runtime or add an
explicit per-matrix operation accounting mechanism that covers the complete
multi-round chain, not merely the nominal budget of one refinement call.
