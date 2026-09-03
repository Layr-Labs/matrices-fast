# Experiment 0048: tiered medium later-round windows with bounded work

## Status

Selected and locally verified for submission. This experiment reduces the
subtree-size windows and block caps of three conditional medium rounds. It
improves the development score while strictly reducing the maximum search
allocation of every changed round relative to the current hidden-accepted
frontier.

## Frontier

The current promoted frontier is submission `cdae07b8`, commit `fed9a10`:

```text
development score = 0.847732
hidden score      = 0.873711
```

Its first-round subtree configuration is already tiered by matrix dimension,
but later rounds use global windows:

```text
round 1: tiered max_s (medium 128), 16 medium blocks by 2M
round 2: tiered max_s (medium 128), 32 blocks by 2M
round 3: global max_s 512, 32 blocks by 2M
round 4: global max_s 768, 32 blocks by 2M
round 5: global max_s 768, 32 blocks by 2M
```

The frontier note swept the later windows globally and found 512/768 best for
the weighted score, but did not publish per-tier results. Earlier work on the
first-round window established that medium and large matrices want opposite
caps, so a global optimum can hide a medium-only improvement.

## Timeout evidence constraining the design

Experiments 0046 and 0047 both lowered the medium first-round `min_s` from 32
to 24. They scored `0.847635` and `0.847610` on the full development harness,
respectively, but both failed hidden evaluation before scoring:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

The second failure persisted after reducing the first-round allocation from
32M to 16M. Inspection of `subtree_refine` explains why minimum-size changes are
risky: the implementation enumerates all eligible disjoint subtrees before
ranking them, and newly admitted small subtrees may carry boundaries up to
`max_sub = 1,200`. Reducing only `max_blocks` cannot prevent that eligibility
change.

This experiment therefore restores the hidden-passing `min_s = 32` and the
entire first two rounds exactly. It changes only maximum eligible subtree sizes
in later rounds, then caps their searched blocks at 16 to control any increase
in the number of smaller disjoint blocks exposed by the lower windows.

## Hypothesis

The medium first round already prefers `max_s = 128`, while large matrices
prefer 384. The global later schedule of 512 then 768 was chosen before this
size-tier structure was understood. Medium elimination trees may benefit from
the same widening schedule at a smaller scale: a moderate third-round window,
followed by wider but not 768-vertex fourth and fifth rounds.

Smaller windows can improve quality because one oversized block otherwise
swallows several useful local basins. They can also expose more disjoint blocks,
so the safe implementation combines the window change with a 16-block cap. No
stream, round, per-block budget, or input feature is added.

## Controlled medium sweep

All probes cover all 108 development matrices with
`1,000 <= n < 10,000`. Small and large behavior is excluded from the branch and
therefore unchanged. Measurements were taken in release mode on an idle
16-logical-core, 64-GiB arm64 verification host.

The initial factor sweep used the frontier's 32-block later-round caps:

| round-3 `max_s` | round-4/5 `max_s` | medium geomean | finding |
|---:|---:|---:|---|
| 512 | 768 | 0.872433 | promoted control |
| 384 | 768 | 0.872169 | smaller third round helps alone |
| 512 | 512 | 0.872444 | later reduction alone does not help |
| 384 | 512 | 0.872061 | interaction is positive |
| 256 | 384 | 0.872299 | too narrow as a pair |
| 320 | 512 | 0.871979 | strong basin |
| 256 | 512 | 0.871940 | strong basin |
| 192 | 512 | 0.872064 | turning point below 256 |
| 256 | 448 | 0.871938 | later cap can shrink further |
| 224 | 448 | 0.872213 | third round now too small |
| **288** | **448** | **0.871905** | best 32-block point |
| 320 | 448 | 0.871941 | neighboring point confirms plateau |

The important result is a broad basin rather than a single lucky integer.
Third-round caps 256, 288, and 320 with later caps 448 or 512 all land within
`0.000074`. The curve turns back at 192 and 224. Selecting 288/448 uses the
center of the good third-round region and the smaller of the equivalent later
windows.

## Runtime-control sweep and selected point

The best 32-block point is not shipped directly. Prior failures show that a
nominally smaller window may expose more disjoint blocks and alter conditional
downstream work. The changed rounds are therefore capped at 16 blocks:

| configuration | medium geomean | changed-round ceiling | decision |
|---|---:|---:|---|
| frontier 512/768, 32 blocks | 0.872433 | 64M per round | hidden-passing control |
| 288/448, 32 blocks | **0.871905** | 64M per round | best dev, less safety |
| **288/448, 16 blocks** | **0.872053** | **32M per round** | **selected** |

The selected point retains a medium-bucket improvement of `0.000380` while
halving the requested search ceiling for every changed round. It uses the same
per-block 2M budget and one stream. It does not touch the first two rounds,
which are already proven on hidden evaluation.

Focused selected result:

```text
1K10K_GEOMEAN = 0.872053 (count 108)
1K10K_WORST   = 0.512 s on arki0016
1K10K_TOTAL   = 29.0 s
```

Public wall-clock timing is descriptive rather than proof of hidden safety. The
deterministic bound is the stronger evidence: each changed round searches at
most half as many blocks, and every accepted subtree is no larger than before.

## Exact implementation

Only the conditional chain's rounds 3 through 5 receive medium-specific caps:

```rust
cfg3.max_s = if (1_000..10_000).contains(&n) { 288 } else { 512 };
if (1_000..10_000).contains(&n) {
    cfg3.max_blocks = 16;
}

cfg4.max_s = if (1_000..10_000).contains(&n) { 448 } else { 768 };
if (1_000..10_000).contains(&n) {
    cfg4.max_blocks = 16;
}

cfg5.max_s = if (1_000..10_000).contains(&n) { 448 } else { 768 };
if (1_000..10_000).contains(&n) {
    cfg5.max_blocks = 16;
}
```

The challenge's own score buckets use the same dimension boundaries. This is a
uniform distribution-level policy, not per-instance tuning. It does not inspect
matrix names, values, corpus position, reference permutations, stored scores,
filesystem state, environment variables, clocks, or randomness.

## Expected score

The outer buckets are behaviorally unchanged. Applying the exact 0.30 medium
weight to the measured bucket delta gives:

```text
0.847732 + 0.30 * (0.872053 - 0.872433) = 0.847618
```

That is an expected development improvement of `0.000114`, about 1.34 basis
points relative to the frontier. The current leader's note estimated that prior
medium changes transferred to hidden evaluation at roughly 0.55x. That estimate
is based on few points and is not treated as a guarantee, but the selected
development margin is large enough to justify one hidden-safe submission.

## Runtime case

Relative to the accepted frontier, this candidate:

- keeps first-round medium `min_s = 32` and `max_s = 128` unchanged;
- keeps round-2 medium `min_s = 16` and `max_s = 128` unchanged;
- reduces round-3 `max_s` from 512 to 288;
- reduces round-4/5 `max_s` from 768 to 448;
- reduces round-3/4/5 `max_blocks` from 32 to 16;
- keeps the 2M per-block budget and stream count unchanged;
- adds no round, graph construction, scorer call, thread, or allocation family;
- leaves every small and large matrix on the exact accepted path.

The maximum requested work in each changed round falls from 64M to 32M word
operations. Conditional entry remains unchanged: a later round runs only after
the previous candidate strictly improves the trusted predicted-flop objective.
The candidate can choose a different permutation, but it cannot request more
search work in any changed round.

## Negative side sweeps

Two other equal-or-lower-work axes were closed before this candidate:

1. Reducing `max_sub` from 1,200 to 1,000 was medium score-neutral at displayed
   precision and slightly regressed the large bucket; 800 regressed medium.
2. Sweeping the block-ranking size exponent from 0.75 to 0.5, 2/3, 0.8, 5/6,
   6/7, and 1.0 showed a shallow large-only improvement near 5/6 but medium
   regressions. The large delta was too small to stack before establishing a
   hidden-passing medium change, so the production comparator remains exactly
   the promoted 0.75 implementation.

These negative results are intentionally absent from the submitted source.

## Verification

The exact selected source was run through:

```text
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

Both checks passed:

```text
cargo test -p ssi-candidate-worker --release
  PASS: 25 passed; 0 failed; 16 ignored

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: 300 matrices; score 0.847618; fill 0.946416
```

Rounded bucket report:

| bucket | count | flop geomean | fill geomean |
|---|---:|---:|---:|
| `lt_1k` | 147 | 0.8939 | 0.9624 |
| `1k_10k` | 108 | 0.8721 | 0.9585 |
| `gt_10k` | 45 | 0.7946 | 0.9254 |

The full command includes the isolated build, source purity scan, pinned
`cargo-deny 0.20.2` license check, optimized worker, determinism repeat,
permutation validation, hard 2.0-second per-matrix watchdog, and trusted scorer.
The exact full-corpus score improves the current development frontier by
`0.000114`, about 1.34 basis points.

## Rule compliance

- Only `src/ordering/` is modified.
- `order(pattern: &Pattern) -> Vec<usize>` remains the only candidate entrypoint.
- Rust standard library and existing allowed challenge modules only.
- No dependency, manifest, lockfile, build-script, network, filesystem,
  subprocess, FFI, thread-count, clock, environment, or randomness changes.
- The algorithm receives only the sparsity pattern and returns a deterministic
  permutation.
- Candidate permutations replace the incumbent only after strict predicted-flop
  improvement.
- The trusted grader independently recomputes every official cost.

## Decision

Submit 288/448 with 16-block caps. The full 300-matrix harness and release unit
suite pass. Do not ship the slightly better 32-block point until a 16-block
version survives hidden evaluation. The selected configuration is designed from
the two timeout receipts: score improvement is necessary, but no development
gain is useful unless realized work is bounded below the accepted frontier.
