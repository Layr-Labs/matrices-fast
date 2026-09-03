# Experiment 0052: single late round at the 64M boundary

## Status

Verified and selected for submission. Starting from the hidden-accepted 0051 source,
this experiment changes exactly one functional constant: chain round 4 moves
from 32M to 64M per block. Round 5 stays at 16M, rounds 2 and 3 stay at 8M, and
all other ordering behavior remains identical to the official leader.

## Accepted base

Experiment 0051 was submitted as `d6de8499-2be4-42a5-8fac-c6828e511f9a` and
promoted as commit `7177486`:

```text
development score = 0.845707
development fill  = 0.944811
hidden score      = 0.871418
hidden fill       = 0.955486
```

It improved our preceding hidden record by 0.000409, 4.69 basis points, and is
the official #1 at the start of this experiment. The hidden gain was 1.18 times
the measured public gain, confirming for a second consecutive step that
conditional late-round depth generalizes to the private distribution.

## Why test 64M once, and only once

The failed budget family before the cap-safe reshaping used 64M in several
rounds simultaneously. Those submissions exceeded the hidden two-second cap.
That makes 64M an important empirical boundary, not a casually safe setting.

The new evidence is different in three ways:

1. Rounds 2 and 3 remain at their hidden-accepted 8M budgets.
2. Round 5 remains at its hidden-accepted 16M budget.
3. Only round 4, which is reached after several strict chain improvements,
   receives 64M.

Thus this is not a return to the failed multi-round configuration. It is a
single conditional phase at the boundary, tested on top of two successive
hidden promotions. If it fails, 32M remains the accepted limit and no larger
round-4 budget should be attempted.

## Hypothesis

Round 4 has been the strongest late budget axis at both measured steps:

```text
all-8M frontier -> round 4 at 16M: -0.000219 projected
16M/16M base    -> round 4 at 32M: -0.000347 measured
```

The curve has not flattened, while the corresponding round-5 increases were
smaller. Doubling round 4 to 64M should therefore deliver another meaningful
score reduction without widening any gate, adding any phase, or increasing any
other budget.

## Focused result

The exact 8M / 8M / 64M / 16M schedule produced:

| bucket | accepted 8/8/32/16M | candidate 8/8/64/16M | change |
|---|---:|---:|---:|
| lt_1k | 0.893224 | **0.893334** | +0.000110 |
| 1k_10k | 0.869703 | **0.868807** | -0.000896 |
| gt_10k | 0.792071 | **0.791922** | -0.000149 |
| weighted score | 0.845707 | **0.845411** | **-0.000296** |

The smallest bucket regresses slightly, but the medium and large reductions
more than compensate under the challenge's fixed 0.30 / 0.30 / 0.40 weights.
The net public improvement is about 3.50 basis points. Unlike experiments 0050
and 0051, this is not an all-bucket win, so the submission case rests on the
larger exact weighted margin and on the already positive hidden translation of
the same round-4 family.

The medium bucket carries most of the gain. Notable public movers include
`crudeoil_pooling_ct3`, `crudeoil_lee1_07`, and the `rsyn` family. The large
bucket also improves, including continued reductions in the crude-oil,
pooling, and power-flow matrices. The best-of guard remains unchanged, so every
accepted ordering still has a strictly lower predicted flop count than its
incumbent on that matrix.

## Runtime evidence

Focused probes on the Mac Studio reported:

```text
lt_1k worst  = 0.376 s
1k_10k worst = 0.739 s
gt_10k worst = 1.003 s
```

For the hidden-accepted 32M/16M parent, the corresponding worst values were
0.369, 0.644, and 0.937 seconds. The large-bucket worst crosses one second but
remains well below the public two-second watchdog on this box. The probes were
run concurrently, so these values are conservative A/B diagnostics rather than
a substitute for the trusted full run.

The runtime containment is structural:

- no new search stage;
- no new block or stream;
- no gate or size-window widening;
- no change to early rounds 2 and 3;
- no change to round 5;
- no terminal-pass or follow-up change;
- exactly one conditional late phase doubles from 32M to 64M.

This is still a deliberate risk at the known boundary. The current 0.871418
promotion remains safe if the candidate fails private validation.

## Nearby points and stopping rule

Experiment 0051 already measured the local two-dimensional bracket:

```text
r4/r5 = 32/16M -> 0.845707 (accepted)
r4/r5 = 16/32M -> 0.845774
r4/r5 = 32/32M -> 0.845680
```

Doubling round 5 on the 32M round-4 source bought only 0.31 basis points and was
rejected. This experiment therefore keeps round 5 at 16M. Round 4 is not tested
above 64M: a 128M point would exceed the known failed per-block depth and has no
credible hidden safety argument.

If 64M passes and promotes, the next research direction must switch away from
raw late-round depth unless it can reallocate fixed work. If 64M fails, restore
the promoted 32M/16M source immediately and treat 32M as the hard private limit.

## Implementation

The functional diff against promoted commit `7177486` is exactly:

```rust
cfg4.budget = 64_000_000; // accepted base: 32_000_000
```

All inputs and outputs are unchanged: `order()` receives only `&Pattern` and
returns a deterministic `Vec<usize>` permutation.

## Full verification

The exact candidate passed:

```text
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release unit suite passed 25 active tests, with 16 ignored and zero
failures. The trusted 300-matrix run completed successfully:

```text
development score = 0.845411
fill ratio         = 0.944632
```

The full command includes the isolated candidate build, source purity scan,
pinned license check, optimized worker, deterministic repeat, permutation
validation, public hard watchdog, and trusted flop scorer. Local and Studio
source hashes must match before submission. The exact full score and fill are
recorded in this page only after the command completes.

## Rule compliance

- Edits remain inside `src/ordering/`.
- `order(pattern: &Pattern) -> Vec<usize>` remains the sole entrypoint.
- Rust standard library and existing allowed challenge modules only.
- No dependency, manifest, lockfile, build script, network, filesystem,
  subprocess, FFI, clock, thread-count, environment, or randomness change.
- No matrix names, corpus positions, values, stored orders, or hidden metadata.
- The trusted grader independently scores the returned permutation.

## Decision

Submit: the exact full corpus and unit suite passed. Do not test or submit a
budget above 64M, and do not combine this boundary step with a larger round-5
budget.
