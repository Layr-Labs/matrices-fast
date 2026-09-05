# Experiment 0047: lower-work medium subtree floor

## Status

Rejected after submission. The 16M candidate hit the same hidden 2.0-second
per-matrix timeout as the 32M version, so no official score was produced.
Submission `a4153739-d5b0-43f7-9b04-c5c626541083`, commit `ff67064`, failed in
the Benchmark step. This experiment preserved the quality-producing medium
first-round floor from experiment 0046 but cut that path's block allocation in
half; the failure shows that `max_blocks` was not the dominant hidden cost.

## Frontier

The current promoted frontier is submission `cdae07b8`, commit `fed9a10`:

```text
development score = 0.847732
hidden score      = 0.873711
```

Its size-tiered first-round subtree configuration is:

| tier | `min_s` | `max_s` | blocks x budget | requested ceiling |
|---|---:|---:|---:|---:|
| `n < 1,000` | 16 | 256 | 16 x 2M | 32M |
| `1,000 <= n < 10,000` | 32 | 128 | 16 x 2M | 32M |
| `n >= 10,000` | 32 | 384 | 16 x 2M | 32M |

Only the medium tier changes here. Small and large matrices retain the exact
promoted configuration.

## Negative result that determines this experiment

Experiment 0046 lowered the medium first-round `min_s` from 32 to 24 while
retaining 16 blocks by 2M. It produced a strong local result:

```text
medium geomean = 0.872108
full dev score = 0.847635
fill tiebreak  = 0.946272
```

Submission `b925cc0a-77c2-458e-a89a-899469b5e941` nevertheless failed before
scoring. The public GitHub Actions receipt is unambiguous:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The workflow ran for 9m22s and failed in its Benchmark step. The result teaches
an important distinction: retaining the same *nominal* 32M ceiling is not enough.
Lowering `min_s` admits additional 24-to-31-vertex subtrees on at least one
hidden medium matrix, so it can increase *actual* work even though the constants
still multiply to the promoted ceiling.

The continuation must therefore reduce the actual first-round allocation, not
just rely on a copied cap or public wall-clock timing.

## Hypothesis

The 0046 score improvement comes from exposing useful smaller first-round
subtrees, but the public corpus may not require all 16 eligible blocks. Reducing
`max_blocks` can simultaneously:

1. cap additional work on a hidden tree with many 24-to-31-vertex subtrees;
2. preserve the earliest, highest-priority useful blocks on development trees;
3. prevent lower-ranked blocks from perturbing later chain rounds adversely;
4. leave the 2M per-block search depth that the promoted allocation established
   as more effective than shallow 1M-to-1.5M searches.

This is an allocation change, not a new algorithm or a corpus-specific gate.

## Controlled lower-work sweep

All rows use the promoted source plus medium `min_s = 24` and `max_s = 128`.
Only first-round `max_blocks` and `budget` vary. The focused probe covers all 108
public matrices with `1,000 <= n < 10,000` on an idle 16-logical-core, 64-GiB
arm64 Mac Studio.

| blocks x budget | requested ceiling | vs failed 32M | medium geomean | projected full score | decision |
|---:|---:|---:|---:|---:|---|
| 16 x 2M | 32M | 100% | 0.872108 | 0.847635 | hidden timeout; reject |
| 16 x 1.5M | 24M | 75% | 0.872785 | 0.847838 | worse than frontier |
| 12 x 2M | 24M | 75% | 0.872180 | 0.847656 | viable, less safety |
| 10 x 2M | 20M | 62.5% | **0.871931** | **0.847581** | best development score |
| **8 x 2M** | **16M** | **50%** | **0.872024** | **0.847610** | **selected risk-adjusted point** |
| 6 x 2M | 12M | 37.5% | 0.872307 | 0.847694 | margin too small |

The sweep has two strong signals. First, per-block depth matters: spreading 24M
over 16 shallow blocks loses badly, while 12 deeper blocks remain competitive.
Second, more blocks are not monotonically better. Ten blocks outperform the
failed 16-block candidate, suggesting later selected blocks can interfere with
subsequent chain rounds rather than adding independent improvements.

The numerically best public point is 10-by-2M, but it is not selected. The
8-by-2M point gives up only about `0.000025` in measured full score while
removing another 20% of the first-round work. It halves the allocation that
failed hidden evaluation and retains approximately 1.45 basis points of public
frontier margin. Six blocks provide still more safety, but their expected hidden
gain is likely too small after the observed medium dev-to-hidden attenuation.

## Selected implementation

Only the medium branch differs from the promoted frontier:

```rust
const MID_MAX_S: usize = 128;
const MID_BLOCKS: usize = 8;
const MID_BUDGET: i64 = 2_000_000;

// Inside subtree_cfg_for(n):
} else {
    cfg.min_s = 24;
    cfg.max_s = MID_MAX_S;
    cfg.max_blocks = MID_BLOCKS;
    cfg.budget = MID_BUDGET;
}
```

The effective first-round configuration is:

```text
if 1,000 <= n < 10,000:
    min_s      = 24
    max_s      = 128
    max_blocks = 8
    budget     = 2,000,000
```

The selection remains distribution-level. It uses only `n`, a basic property of
the supplied sparsity pattern already used by the promoted implementation. It
does not use matrix names, corpus membership, values, right-hand sides,
reference permutations, stored scores, environment variables, clocks, or
randomness.

## Runtime case

Public timing cannot prove hidden safety; experiment 0046 passed the entire
public corpus and still timed out. The relevant evidence here is deterministic
work reduction:

- selected first-round medium work falls from the failed 32M to 16M;
- maximum eligible subtree size remains 128 vertices;
- per-block depth remains the promoted 2M rather than introducing a deeper stream;
- stream count and number of chain rounds are unchanged;
- later chain-round configurations are unchanged;
- small and large tier behavior is unchanged;
- no additional graph construction, scorer call, allocation, or traversal is added.

The focused 8-by-2M probe reported:

```text
1K10K_GEOMEAN = 0.872024 (count 108)
1K10K_WORST   = 0.498 s on arki0016
1K10K_TOTAL   = 28.9 s
```

For comparison, 10-by-2M measured a 0.503-second public maximum and 12-by-2M a
0.497-second maximum. Those similar development timings show why local seconds
alone are not a useful selector: the public worst matrix is dominated by work
outside this first-round cap. The halved deterministic allocation is the safety
argument for hidden inputs.

## Expected score and hidden translation

The promoted medium bucket is 0.872433 and the selected bucket is 0.872024. With
small and large byte-identical, the exact score model gives:

```text
0.847732 + 0.30 * (0.872024 - 0.872433) ~= 0.847609
```

The exact full-corpus score is `0.847610` with fill tiebreak `0.946244`, the
minor final-digit difference coming from the rounded bucket values above. This
improves the development frontier by `0.000122`, about 1.44 basis points.

The current leader's note estimates that prior medium changes translated from
development to hidden at roughly 0.55x. That estimate is sparse and not a
guarantee, but it suggests the selected development delta has enough hidden
margin to improve while remaining substantially safer than the failed 32M path.
The official hidden grader remains authoritative.

## Full-corpus verification

The exact full-corpus command was:

```text
bash scripts/local-candidate-build.sh && cargo run --release
```

It completed all 300 matrices successfully:

```text
OK    score=0.847610    fill=0.946244

bucket    count     flop_geomean     fill_geomean
lt_1k       147           0.8939           0.9624
1k_10k      108           0.8720           0.9579
gt_10k       45           0.7946           0.9254
```

It runs the repository's isolated candidate build, source purity scan, pinned
`cargo-deny 0.20.2` license check, optimized worker, repeat determinism check,
permutation validation, 2.0-second per-matrix cap, and trusted flop scorer. The
exact result above is from the selected source.

The final release unit suite is also run with:

```text
cargo test -p ssi-candidate-worker --release
```

The release suite passed 25 tests with zero failures and 16 intentionally ignored
research probes. Four pre-existing dead-code
warnings may be emitted for unused relabel constants/function and unused score
variants; these are not introduced by this experiment.

## Rule compliance

- Only `src/ordering/` is modified.
- `order(pattern: &Pattern) -> Vec<usize>` remains the candidate entrypoint.
- Rust standard library and existing allowed local challenge modules only.
- No dependency, manifest, lockfile, build-script, network, filesystem,
  subprocess, thread, FFI, clock, environment-variable, or randomness changes.
- The algorithm observes only the sparsity pattern.
- The result remains deterministic and is validated as a permutation.
- Candidate orders replace the incumbent only after a strict predicted-flop
  reduction.
- The grader independently recomputes all official costs.

Verification receipts:

```text
cargo test -p ssi-candidate-worker --release
  PASS: 25 passed; 0 failed; 16 ignored

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: 300 matrices; score 0.847610; fill 0.946244
```

## Decision

Close the entire `min_s < 32` direction for the medium first round. Even 8
blocks by 2M, half the failed allocation, exceeded the hidden cap. The likely
cost is admitting or scanning additional 24-to-31-vertex candidate subtrees,
not merely the number of selected blocks. Revert to the promoted `min_s = 32`,
16-by-2M configuration before exploring another axis. Do not submit 10-by-2M,
12-by-2M, 16-by-1.5M, or 6-by-2M from this sweep.

Official failure receipt:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```
