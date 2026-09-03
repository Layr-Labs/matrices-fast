# Experiment 0049: bounded medium terminal cascade

## Status

Verified and selected for submission. This experiment combines the
score-positive medium later-round windows from 0048 with a 75% reduction in
each adaptive terminal follow-up pass. It is designed directly from three
hidden timeout receipts and restores all small and large behavior.

## Frontier

The current promoted frontier is submission `cdae07b8`, commit `fed9a10`:

```text
development score = 0.847732
hidden score      = 0.873711
```

The promoted source passes the hidden two-second per-matrix gate. Every proposed
change is evaluated not only for public score but for the maximum downstream
work it can activate relative to that accepted source.

## Three failures and the new diagnosis

Three development-winning medium changes were submitted before this experiment:

| experiment | development score | intended runtime control | hidden result |
|---|---:|---|---|
| 0046 | 0.847635 | unchanged nominal 32M first-round cap | timeout |
| 0047 | 0.847610 | first-round cap reduced to 16M | timeout |
| 0048 | 0.847618 | later-round windows smaller; later blocks 32 to 16 | timeout |

All three completed the 300-matrix public harness and passed unit, purity,
license, determinism, and permutation checks. All three failed at the same
hidden grader line before an official score was produced:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

Experiment 0047 showed that first-round `max_blocks` was not enough to control
the lower-`min_s` direction. Experiment 0048 restored the accepted `min_s` and
reduced the changed later rounds themselves, yet still timed out. That isolates
the important common dependency: every changed chain can produce a different
best permutation before the terminal phase, and that new incumbent can activate
two conditional terminal follow-up passes.

Those passes were described by their comments as a small 4M operation cap, but
the source actually configured each as four blocks by 4M operations: a 16M
requested ceiling per pass. A candidate that activates both may therefore add
32M of downstream work even when the earlier rounds were made cheaper.

## Hypothesis

The score-positive 288/448 medium window schedule should remain useful if the
terminal cascade is retained at shallower depth. Two blocks by 2M operations per
follow-up pass is the allocation described by the existing comment and reduces
each pass from 16M to 4M. The two-pass cascade ceiling falls from 32M to 8M.

This configuration keeps the cascade's adaptive structure and distinct round-6
and round-7 seeds. It therefore preserves an opportunity to improve the newly
exposed elimination tree rather than deleting the phase entirely.

## Ablation: the terminal cascade carries the gain

Starting from experiment 0048's medium-only later windows and 16-block caps:

| terminal follow-up policy | medium geomean | interpretation |
|---|---:|---|
| promoted 4 blocks x 4M per pass | **0.872053** | strong score, hidden timeout |
| follow-up passes disabled for medium | 0.872414 | most of gain disappears |
| **2 blocks x 2M per pass** | **0.872138** | **retains useful gain at 25% work** |

The disabled result proves that the follow-up passes are not incidental: they
convert the different 288/448 incumbent into most of the measured score gain.
The selected shallow cascade recovers `0.000276` of that `0.000361` difference,
about 76%, while requesting only one quarter of the original follow-up work.

## Selected medium configuration

The first two rounds stay exactly on the hidden-accepted frontier:

```text
round 1: min_s 32, max_s 128, 16 blocks x 2M
round 2: min_s 16, max_s 128, 32 blocks x 2M
```

The later chain uses the score-positive smaller windows with explicit block
caps:

```text
round 3: min_s 16, max_s 288, 16 blocks x 2M
round 4: min_s 16, max_s 448, 16 blocks x 2M
round 5: min_s 16, max_s 448, 16 blocks x 2M
```

The independent terminal pass remains unchanged. Only its two conditional
follow-ups are reduced for medium matrices:

```text
terminal follow-up 2: min_s 8, 2 blocks x 2M, round seed 6
terminal follow-up 3: min_s 8, 2 blocks x 2M, round seed 7
```

Small (`n < 1,000`) and large (`n >= 10,000`) matrices retain four blocks by 4M
in both terminal follow-ups. The production change is gated exactly to
`1,000 <= n < 10,000`.

## Focused result

The selected medium probe covers all 108 public matrices in the middle score
bucket:

```text
1K10K_GEOMEAN = 0.872138 (count 108)
1K10K_WORST   = 0.508 s on arki0016
1K10K_TOTAL   = 28.7 s
```

The promoted medium bucket is 0.872433. Small and large are behaviorally
unchanged, so the projected full score is:

```text
0.847732 + 0.30 * (0.872138 - 0.872433) = 0.8476435
```

The expected development improvement is approximately `0.0000885`, about 1.04
basis points. That is smaller than the timed-out candidates by design: the
candidate exchanges some public margin for a hard reduction in the exact
downstream phase implicated by the failure sequence.

The full trusted 300-matrix run confirmed the projection:

```text
development score = 0.847644
fill ratio         = 0.946432
```

## Runtime accounting

Relative to the accepted frontier, the selected medium path has:

| phase | frontier ceiling | selected ceiling | change |
|---|---:|---:|---:|
| round 3 | 32 x 2M = 64M | 16 x 2M = 32M | -50% |
| round 4 | 32 x 2M = 64M | 16 x 2M = 32M | -50% |
| round 5 | 32 x 2M = 64M | 16 x 2M = 32M | -50% |
| terminal follow-up 2 | 4 x 4M = 16M | 2 x 2M = 4M | -75% |
| terminal follow-up 3 | 4 x 4M = 16M | 2 x 2M = 4M | -75% |

The window caps also fall from 512/768/768 to 288/448/448. The first two rounds
and the independent terminal pass are unchanged. No phase, stream, thread,
graph construction, scoring call, or allocation family is added.

Smaller windows can expose more disjoint blocks, which is why the explicit
16-block caps are retained even though the windows themselves are smaller. The
terminal passes use both fewer blocks and half the per-block budget, limiting
the cost of a single newly exposed pathological block as well as aggregate work.

This is a deterministic work argument. Public seconds are not treated as proof
of hidden safety after three public-passing timeouts.

## Implementation

The later-round changes are gated by the challenge's medium dimension range:

```rust
cfg3.max_s = if (1_000..10_000).contains(&n) { 288 } else { 512 };
if (1_000..10_000).contains(&n) { cfg3.max_blocks = 16; }

cfg4.max_s = if (1_000..10_000).contains(&n) { 448 } else { 768 };
if (1_000..10_000).contains(&n) { cfg4.max_blocks = 16; }

cfg5.max_s = if (1_000..10_000).contains(&n) { 448 } else { 768 };
if (1_000..10_000).contains(&n) { cfg5.max_blocks = 16; }
```

The two terminal configs select 2-by-2M only for the same medium range and keep
4-by-4M everywhere else.

The selector uses only `n`, a basic property of the supplied sparsity pattern
already used throughout the promoted tier configuration. It does not inspect
matrix names, values, corpus position, reference orders, stored answers,
environment variables, files, clocks, or nondeterministic state.

## Full verification

The exact candidate was verified with:

```text
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release unit suite passed 25 active tests (16 ignored, 0 failed). The full
run completed successfully at `0.847644` / `0.946432`. The latter includes the isolated source build, purity scan, pinned
`cargo-deny 0.20.2` license check, optimized candidate worker, deterministic
repeat, permutation validation, hard two-second public watchdog, and trusted
flop scorer. The exact score and fill are recorded before submission.

## Rule compliance

- Only `src/ordering/` is modified.
- `order(pattern: &Pattern) -> Vec<usize>` remains the sole entrypoint.
- Rust standard library and existing allowed challenge modules only.
- No dependency, manifest, lockfile, build-script, network, filesystem,
  subprocess, FFI, thread-count, clock, environment, or randomness changes.
- The algorithm receives only the sparsity pattern.
- It returns a deterministic permutation and keeps candidates only after strict
  predicted-flop improvement.
- The trusted grader independently recomputes all official costs.

## Decision

Submit: the full 300-matrix run and release unit suite both pass. This is the
first candidate whose runtime repair covers both the directly changed chain and
the adaptive downstream cascade implicated by the three hidden timeouts. Do not
restore the 4-by-4M terminal follow-ups merely for a larger development delta;
the competition assigns no score at all to a candidate killed by the watchdog.
