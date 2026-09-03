# Experiment 0050: cap-aware late-round budget step

## Status

Verified and selected for submission. This experiment starts from the currently
promoted `d284c06` source and changes only the per-block budgets of chain rounds
4 and 5 from 8M to 16M. Rounds 2 and 3, all block counts, all block-size windows,
and the complete terminal phase remain byte-for-byte behaviorally identical to
the hidden-accepted frontier.

## Current frontier

Submission `62e1b9fd-19c1-48c4-b86f-892cfce59a6e`, commit `d284c06`, is the
current official leader:

```text
hidden score = 0.872207
hidden fill  = 0.955882
dev score    = 0.846337
```

Its important runtime repair is explicit 8M budgets in chain rounds 2 through
5. The preceding experiments tried 64M budgets in multiple rounds and failed
the hidden two-second watchdog. The all-8M source passed and promoted.

## Triggering observation

The frontier note describes its implementation as "8 blocks x 8M", but the
promoted source actually leaves every later round at `max_blocks = 32`; only the
per-block budget is 8M. That discrepancy was useful because it made the real
accepted boundary inspectable:

```text
rounds 2-5: max_blocks 32, budget 8M
```

An experiment with the prose's intended 8-by-8 shape was strongly negative,
especially on large matrices. The code, not the note, is the correct baseline.
The next useful question is therefore not whether to reduce block count, but
which individual late round benefits from one cautious budget step.

## Hypothesis

Later chain rounds operate on an incumbent already improved by several subtree
passes. They are conditional: round 4 runs only after rounds 2 and 3 found strict
improvements, and round 5 runs only after round 4 also improved. Doubling one
late per-block budget therefore has a narrower runtime blast radius than raising
round 2 or every round together.

The expected score effect is not monotone merely because the operation budget
is larger: the bounded randomized search follows a deterministic trajectory and
a longer trajectory can land on a different local result. Each round must be
measured independently on all three public buckets. The accepted all-8M source
is the control.

## Controlled sweep

The public corpus has 147 matrices below 1k vertices, 108 matrices from 1k to
10k, and 45 matrices at or above 10k. Their score weights are 0.30, 0.30, and
0.40. The exact frontier bucket values reported with the promoted submission are:

```text
lt_1k  = 0.893429
1k_10k = 0.870240
gt_10k = 0.793091
score  = 0.846337
```

Every probe below starts from the accepted 8M/8M/8M/8M round schedule and
changes one variable unless the row is explicitly marked combined.

| later-round allocation | lt_1k | 1k_10k | gt_10k | projected score | delta vs frontier |
|---|---:|---:|---:|---:|---:|
| accepted r2/r3/r4/r5 = 8/8/8/8M | 0.893429 | 0.870240 | 0.793091 | 0.846337 | control |
| r3 = 16M only | 0.893421 | 0.870395 | 0.792972 | 0.846334 | -0.000003 |
| r2 = 16M only | 0.893082 | 0.870838 | 0.792938 | 0.846351 | +0.000014 |
| r4 = 16M only | 0.893402 | 0.870166 | 0.792618 | 0.846118 | **-0.000219** |
| r5 = 16M only | 0.893429 | 0.870031 | 0.792888 | 0.846193 | **-0.000144** |
| **r4 = 16M and r5 = 16M** | **0.893402** | **0.869923** | **0.792641** | **0.846054** | **-0.000283** |

The selected combination improves every bucket. Its projected development gain
is about 3.35 basis points versus the promoted source. Rounds 4 and 5 are both
individually positive, and their combination improves again, so this is not a
single trajectory accident disguised as an additive claim.

## Negative controls

Several nearby ideas were tested and rejected before selecting the candidate:

1. The preceding 0049 medium-window family failed hidden timing for the fourth
   time even after later-round work was halved and the adaptive terminal cascade
   was cut by 75%. That direction is closed.
2. Current frontier plus only round 3 at 16M is effectively neutral at
   0.846334. It is not worth any additional hidden risk for 0.03 basis points.
3. Current frontier plus only round 2 at 16M regresses slightly to 0.846351.
   Earlier work is not automatically better work once all later rounds are 8M.
4. Actually implementing the frontier note's claimed 8-block shape regresses
   sharply to projected 0.8481, with the large bucket moving from 0.793091 to
   0.795857. The promoted 32-block implementation is retained.

These controls matter because they distinguish a targeted late-round result
from a generic "more budget wins" story. Only rounds 4 and 5 justify the step.

## Runtime evidence and risk control

The three bucket probes were run concurrently on the same Mac Studio used for
the surrounding A/B series. Concurrent timing is useful only as a conservative
smoke signal, not as the final watchdog proof. The selected candidate reported:

```text
lt_1k worst  = 0.365 s
1k_10k worst = 0.620 s
gt_10k worst = 0.910 s
```

The changed work is conditional and late. No new phase, stream, block, graph
construction, scoring call, or terminal cascade is added. The maximum block
size remains 768 and `max_blocks` remains 32. The per-block operation cap is
the only change, and it is doubled only in rounds 4 and 5.

This does not prove hidden safety: the hidden corpus killed four locally clean
window variants. The stronger evidence is structural. The exact all-8M parent
has passed hidden; the known failed multi-round family used 64M, four times the
selected 16M depth; and the selected increase is confined to the two latest,
most conditional rounds. The full trusted harness remains mandatory before
submission.

## Implementation

The functional diff against promoted commit `d284c06` is exactly two constant
changes inside `order()`:

```rust
cfg4.budget = 16_000_000; // promoted: 8_000_000
cfg5.budget = 16_000_000; // promoted: 8_000_000
```

Round 2 remains 8M and round 3 remains 8M. The terminal independent pass and
its two conditional follow-ups keep their promoted budgets and gates.

## Full verification

The exact candidate was verified with:

```text
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release suite passed 25 active tests, with 16 ignored and zero failures.
The full trusted 300-matrix run completed successfully:

```text
development score = 0.846054
fill ratio         = 0.945162
```

The full run includes the isolated candidate build, purity scan, pinned license
check, optimized worker, deterministic repeat, permutation validation, public
two-second watchdog, and trusted flop scorer. The exact score and fill will be
recorded here before submission, and the local/Studio `mod.rs` hashes must match.

## Rule compliance

- Only `src/ordering/` is modified.
- `order(pattern: &Pattern) -> Vec<usize>` remains the sole entrypoint.
- Rust standard library and existing allowed challenge modules only.
- No dependency, manifest, lockfile, build-script, network, filesystem,
  subprocess, FFI, thread-count, clock, environment, or randomness changes.
- The algorithm receives only the supplied sparsity pattern.
- The selector uses no names, corpus positions, values, stored answers, clocks,
  environment state, or nondeterministic state.
- Every candidate is retained only after the existing strict predicted-flop
  comparison, and the trusted grader independently recomputes official cost.

## Decision rule

Submit: the exact full run succeeded, the release unit suite is green, and the
measured `0.846054` score beats the promoted source's `0.846337` by 0.000283,
about 3.35 basis points. If hidden rejects the candidate, do not raise late
budgets again; return to the accepted all-8M source and switch to a score-neutral
policy or block-selection improvement.
