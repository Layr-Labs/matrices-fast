# Experiment 0044: density-gated deep search for sparse small patterns

## Status

Submission candidate. This is the timeout repair for experiment 0043. The
exact source passes the official local build and full 300-matrix scorer at
`0.847551` with fill tiebreak `0.946401`. The current promoted frontier's
public development score is `0.847732`; lower is better.

## Trigger

Experiment 0043 changed the `n < 1,000` subtree allocation from 16 blocks by
2M operations to four blocks by 8M operations. It kept the same 32M requested
work ceiling and improved the development score to `0.848766`. It passed the
full public corpus on an idle machine, but hidden submission
`794e1fc0-d883-465e-9e22-dd7cc51a6852` failed after 5 minutes 42 seconds with:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The hidden grader intentionally does not reveal the matrix. The correct
response is therefore not to infer an identity or add a corpus lookup. The
repair follows the grader's general runtime guidance and gates the deeper path
by both graph dimension and nonzero count.

## Implementation

The promoted small-matrix configuration remains the default:

```text
min_s      = 16
max_s      = 256
max_blocks = 16
budget     = 2,000,000
streams    = 1
```

Only sparse small patterns receive the deeper shape:

```text
condition  = n < 1,000 and nnz <= 10,000
min_s      = 16
max_s      = 256
max_blocks = 4
budget     = 8,000,000
streams    = 1
```

Both shapes have the same nominal ceiling:

```text
16 * 2,000,000 * 1 = 32,000,000
 4 * 8,000,000 * 1 = 32,000,000
```

`subtree_cfg_for` now accepts `(n, nnz)` rather than only `n`, and the five
existing call sites pass the already available pattern nonzero count. No
additional graph scan, allocation, dependency, thread, timer, identity test,
or source file outside `src/ordering/` is introduced.

The medium tier keeps `max_s = 128`, 16 blocks, and 2M operations. The large
tier keeps the promoted 16-by-2M allocation with `max_s = 384`. All later
rounds and the terminal-deep chain remain unchanged.

## Why nonzero count is the right safety variable

Dimension alone does not bound the cost of sparse ordering. Two matrices with
the same `n` can have very different adjacency volume, quotient-graph update
cost, fill simulation cost, and local-search move cost. The failed 0043 gate
bounded only dimension. Adding `nnz` bounds the amount of input structure on
the changed path and sends denser small patterns back to the exact
hidden-accepted configuration.

The 10,000 threshold is structural. For `n < 1,000`, it limits the changed
path to an average symmetric pattern degree on the order of twenty or less.
It is not a matrix-name rule and it is not selected from hidden identities.
The orderer still receives only the sparsity pattern.

## Density cutoff sweep

The focused `probe_lt1k` test evaluates all 147 public matrices with
`n < 1,000`. Starting from the four-by-8M result, three density cutoffs were
measured on the same idle arm64 verification host:

| deep-search condition | small flop geomean | worst `order()` time | interpretation |
|---|---:|---:|---|
| all `n < 1,000` | 0.893262 | 0.659 s on the primary run | hidden timeout; reject |
| `nnz <= 20,000` | 0.893287 | 0.322 s | same score basin |
| **`nnz <= 10,000`** | **0.893287** | **0.324 s** | selected |
| `nnz <= 5,000` | 0.893572 | 0.322 s | gives up too much score |

The 10k and 20k results are bit-for-bit identical in flop score across the
public bucket. Choosing 10k therefore removes an entire band of denser inputs
at zero observed score cost. Tightening further to 5k changes several useful
public results and reduces the overall improvement to approximately one basis
point, leaving inadequate promotion margin.

The per-block budget was also reduced from 8M to 6M behind the 10k gate. That
point scored `0.893777` in the small bucket, which is below the promotion-sized
gain. It was rejected and is not present in this candidate.

## Development score

The selected cutoff retains nearly all of the 0043 public improvement:

| configuration | `lt_1k` geomean | overall public score |
|---|---:|---:|
| promoted frontier, 16 x 2M | 0.893893 | 0.847732 |
| **10k density gate, four x 8M** | **0.893287** | **0.847551** |

The repair pays only `0.000008` overall relative to the ungated point while
improving on the promoted frontier by `0.000181`, approximately 2.1 basis
points relative to the frontier score.

The exact full-corpus result written by the trusted local harness is:

```text
OK    score=0.847551    fill=0.946401
```

Rounded per-bucket output:

| bucket | count | flop geomean | fill geomean |
|---|---:|---:|---:|
| `lt_1k` | 147 | 0.8933 | 0.9621 |
| `1k_10k` | 108 | 0.8724 | 0.9587 |
| `gt_10k` | 45 | 0.7946 | 0.9254 |

The unchanged medium and large buckets match the promoted frontier. The
improvement comes from the intended sparse-small branch.

## Timing evidence

Timing was measured on the idle verification host because the primary
workstation had unrelated sustained CPU contention. On that idle host, the
10k-gated `probe_lt1k` result was:

```text
LT1K_GEOMEAN = 0.893287 (count 147)
LT1K_WORST   = 0.324 s on multiplants_stg1b
LT1K_TOTAL   = 11.0 s
```

The changed public path therefore has more than a six-times margin against
the two-second cap on that host. More importantly, all patterns outside both
structural bounds execute the exact promoted allocation that already passed
hidden evaluation. The full official local run completed all 300 matrices
without timeout.

This does not claim that a public timing proves hidden safety. Experiment 0043
demonstrated that it does not. The evidence for 0044 is narrower:

1. the changed path is bounded by both `n` and `nnz`;
2. its public worst time is 0.324 seconds on an idle host;
3. dense small patterns revert to the promoted path;
4. medium and large patterns are completely unchanged;
5. the full official public harness passes on the exact source.

## Rejected broader work

This candidate does not include any of the broader budget-shape experiments.
A 16-by-2M shape for matrices with `n >= 1,000` improved public buckets but
failed the official public cap on `batchs121208m`. A large-only variant with
later rounds restored to 1M entered a pathological refinement basin and took
8.981 seconds on `unitcommit_200_100_1_mod_8`. Those results show why a fixed
nominal operation product is not sufficient: a different first-round
incumbent changes the elimination tree and may activate a costly downstream
chain. All such changes were reverted.

## Correctness and compliance

- Only files inside `src/ordering/` are modified.
- The implementation uses Rust's standard library and existing permitted
  challenge modules only.
- There are no dependency, manifest, lockfile, generated-code, FFI, network,
  environment, subprocess, or filesystem changes in `order()`.
- Selection uses only `Pattern.n`, `Pattern.col_ptr`, and `Pattern.row_idx`.
- The returned vector is still validated as a bijection.
- Candidate orderings replace the incumbent only on a strict predicted-flop
  improvement.
- The density gate is a general structural rule, not a corpus lookup.

Verification receipts:

```text
cargo test -p ssi-candidate-worker --release probe_lt1k -- --ignored --nocapture
  PASS: 1 passed; geomean 0.893287; worst 0.324 s

cargo test -p ssi-candidate-worker --release
  PASS: 25 passed; 0 failed; 16 ignored

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: 300 matrices; score 0.847551; fill 0.946401
```

The official path includes the repository purity scan and the pinned
`cargo-deny 0.20.2` license gate. Four pre-existing dead-code warnings remain;
there are no build errors, permutation errors, or test failures.

## Learning

The useful result is not simply that four blocks beat sixteen on public small
graphs. It is that budget shape and input volume must be considered together.
Deep local search is productive on short, sparse elimination trees, but the
same per-block depth is not robust for every graph that happens to have fewer
than 1,000 vertices. A two-dimensional gate captures the productive regime
while preserving the already validated fallback everywhere else.

If this candidate passes hidden evaluation, future work should keep the 10k
gate fixed and pursue improvements that do not deepen the subtree chain. A
promising direction is constant-cost candidate ranking or early termination
based on completed move counts. Broad increases in chain depth should remain
closed unless the implementation gains a true end-to-end operation bound.
