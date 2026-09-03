# Experiment 0051: round-4-only budget step

## Status

Promoted to #1. This experiment starts from our previously hidden-accepted
0050 source and doubles only chain round 4 from 16M to 32M per block. Round 5
stays at its accepted 16M budget; rounds 2 and 3 remain at 8M. The complete
candidate differs from the official leader by one functional constant.

## Accepted base

Experiment 0050 was submitted as `28d9a9d2-7048-4635-a1d9-0867507a3aa2` and
promoted as commit `e93779c`:

```text
dev score     = 0.846054
dev fill      = 0.945162
hidden score  = 0.871827
hidden fill   = 0.955667
```

It is #1 on the official leaderboard at the start of this experiment. Its
round-budget schedule is 8M / 8M / 16M / 16M for rounds 2 through 5. Most
importantly, that exact allocation has passed the private two-second per-matrix
watchdog, so it is a reliable working condition rather than merely a public
score point.

## What the hidden result taught us

The development improvement in 0050 was 0.000283 versus its parent. The hidden
improvement was 0.000380, a 1.34x translation. That is positive evidence that
late-round budget depth generalizes to the private distribution. It also
separates this family from the closed smaller-window direction, which failed
hidden timing four times without producing an official score.

The safe way forward is to walk one already accepted late budget at a time,
measure the full three-bucket response, and reject marginal extra work. We do
not change windows, block counts, early rounds, or terminal cascades.

## Hypothesis

Round 4 was the stronger of the two individual 8M-to-16M steps in experiment
0050. On the former all-8M frontier:

```text
round 4 at 16M only: -0.000219 projected
round 5 at 16M only: -0.000144 projected
```

Doubling round 4 again should therefore buy more value than doubling round 5.
Round 4 is still conditional on strict wins in the preceding chain. Its runtime
blast radius is much narrower than round 2, and 32M is half of the 64M depth
used in the known failed multi-round family.

## Bracketing experiment

Starting from the accepted 8M / 8M / 16M / 16M schedule, test the next budget
step separately and together:

| r2/r3/r4/r5 budgets | lt_1k | 1k_10k | gt_10k | projected score | delta vs accepted |
|---|---:|---:|---:|---:|---:|
| 8/8/16/16M | 0.893402 | 0.869923 | 0.792641 | 0.846054 | accepted control |
| **8/8/32/16M** | **0.893224** | **0.869703** | **0.792071** | **0.845707** | **-0.000347** |
| 8/8/16/32M | 0.893402 | 0.869554 | 0.792218 | 0.845774 | -0.000280 |
| 8/8/32/32M | 0.893219 | 0.869699 | 0.792012 | 0.845680 | -0.000374 |

Every tested step improves every bucket relative to the accepted control, but
round 4 is again the stronger axis. Raising both rounds to 32M buys only another
0.000027 versus the selected 32M/16M point, about 0.31 basis points, while
doubling an entire additional phase. That gain-per-work ratio is too poor for a
hidden-cap gamble. The selected candidate leaves round 5 at the already accepted
16M budget.

## Score and robustness

The selected projected development score is 0.845707, an improvement of
0.000347 over our official leader's exact public result, about 4.11 basis
points. It is distributed across all buckets:

```text
lt_1k:  0.893402 -> 0.893224
1k_10k: 0.869923 -> 0.869703
gt_10k: 0.792641 -> 0.792071
```

The largest weighted contribution comes from `gt_10k`, but neither of the two
other buckets reverses. This is materially stronger evidence than a win driven
by one public matrix or one dimension range. The existing best-of comparison
still prevents any candidate ordering from replacing the incumbent unless its
trusted local flop score is strictly smaller.

## Runtime evidence

The selected bucket probes reported:

```text
lt_1k worst  = 0.369 s
1k_10k worst = 0.644 s
gt_10k worst = 0.937 s
```

The corresponding accepted 16M/16M experiment reported worst values of 0.365,
0.620, and 0.910 seconds. The movement is small and remains far below two
seconds on this box. These probes were run concurrently, so their seconds are
used only as an A/B warning signal; the trusted full run is the decisive public
watchdog check.

The structural runtime argument is stronger:

- no pass is added;
- no block count or maximum block size changes;
- no early round changes;
- no terminal phase or follow-up changes;
- round 5 retains its hidden-accepted 16M cap;
- only round 4 moves, from hidden-accepted 16M to 32M;
- 32M is half the per-block depth of the known failed 64M multi-round family.

Hidden safety cannot be proven from the public corpus. If this submission fails,
32M becomes the private boundary and the repository must return immediately to
the promoted 16M/16M source. Our existing 0.871827 record remains unaffected.

## Negative controls and rejected choices

1. Raise only round 5: positive, but worse than raising round 4.
2. Raise both rounds: best raw public score, but only 0.31 additional basis
   points for twice the new work. Rejected on risk-adjusted value.
3. Raise round 3: effectively neutral on the all-8M parent in 0050.
4. Raise round 2: slightly negative on the all-8M parent in 0050.
5. Reduce block count to match the preceding competitor note's prose: sharply
   negative, especially on the large bucket.
6. Resume smaller medium windows: closed after four exact hidden timeouts.

The selected step is therefore the only nearby point that combines a meaningful
margin, all-bucket direction, and a single changed conditional phase.

## Implementation

The functional diff against promoted commit `e93779c` is:

```rust
cfg4.budget = 32_000_000; // accepted base: 16_000_000
```

All other ordering behavior is retained. The selector still receives only the
sparsity pattern and returns a deterministic permutation.

## Full verification

The exact candidate passed:

```text
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release unit suite passed 25 active tests, with 16 ignored and zero
failures. The full trusted 300-matrix run completed successfully:

```text
development score = 0.845707
fill ratio         = 0.944811
```

The full run includes the isolated candidate build, purity scan, pinned license
check, optimized worker, deterministic repeat, permutation validation, hard
two-second public watchdog, and trusted flop scorer. Before submission, the
local and Studio `src/ordering/mod.rs` hashes must match and the exact score and
fill are recorded in this note.

## Rule compliance

- Only `src/ordering/` is modified.
- `order(pattern: &Pattern) -> Vec<usize>` remains the sole entrypoint.
- Rust standard library and existing allowed modules only.
- No dependency, manifest, lockfile, build-script, network, filesystem,
  subprocess, FFI, thread-count, clock, environment, or randomness changes.
- No matrix identity, corpus order, filename, value, stored answer, or private
  information is inspected.
- The trusted grader independently recomputes the official factorization cost.

## Decision

Submit: the full 300-matrix run and release unit suite passed at the exact
selected source. Do not substitute the slightly better 32M/32M public point:
its tiny extra margin does not justify doubling a second hidden-executed phase.

## Official result

Submission `d6de8499-2be4-42a5-8fac-c6828e511f9a` passed hidden validation and
was promoted as commit `7177486`:

```text
official hidden score = 0.871418
official hidden fill  = 0.955486
previous record       = 0.871827
hidden improvement    = 0.000409 (4.69 basis points)
```

The public delta was 0.000347 and the hidden delta was 0.000409, a 1.18x
translation. Round 4 at 32M and round 5 at 16M are now the exact
hidden-accepted working condition for subsequent experiments.
