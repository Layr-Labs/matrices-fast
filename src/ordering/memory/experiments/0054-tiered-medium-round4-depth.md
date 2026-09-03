# Experiment 0054: tiered medium round-4 depth

## Status

Submitted for hidden validation. This experiment starts from the promoted 0053
source, submission `e46c5349-c523-4e1f-9c0b-26da15f92d6e`, commit `77153ff`.
That source is the current official leader at hidden score **0.871239** and
hidden fill **0.955407**.

The candidate changes only the dimension-to-budget expression for conditional
chain round 4:

```text
1,000 <= n < 6,000   64M -> 128M
6,000 <= n < 10,000  32M -> 64M
all other n           32M unchanged
```

The full public result is **0.845355** with fill **0.944744**, versus the
accepted source's 0.845469 / 0.944729. The primary-score reduction is 0.000114,
about 1.35 basis points relative to the accepted development score.

## Why continue this axis after 0053

Experiments 0050, 0051, and 0053 each promoted by reallocating more deterministic
work to the same late conditional subtree round:

| experiment | public delta | hidden delta | result |
|---|---:|---:|---|
| 0050: r4/r5 to 16M | -0.000283 | -0.000380 | promoted |
| 0051: r4 to 32M | -0.000347 | -0.000409 | promoted |
| 0053: 64M only below 6k | -0.000238 | -0.000179 | promoted |

The hidden/public translation for the three steps ranged from about 0.75x to
1.34x. The new public margin is smaller, but still clears the benchmark's
one-basis-point minimum and follows a family that has generalized three times
in succession.

The negative boundary is equally important. Experiment 0052 used 64M globally,
scored 0.845411 locally, and then exceeded the hidden 2-second per-matrix cap.
0053 recovered most of its useful lower-medium work while retaining 32M on all
small, upper-medium, and large inputs; that selective source passed hidden
validation. Therefore 0054 does not reopen global depth. It extends the now
accepted dimension-allocation idea only within the medium bucket, where the
bitset state and public runtime are bounded.

## Cutoff sweep

The sweep was cumulative and measured with the same fixed 108-matrix medium
probe. All variants retained the accepted 32M budget outside the listed band.

| round-4 budget shape | medium geomean | worst call | overall gain vs 0053 |
|---|---:|---:|---:|
| accepted: 64M below 6k | 0.868912 | 0.661 s | reference |
| 128M below 4k, 64M 4k-6k | 0.868770 | 0.668 s | about 0.50 bp |
| 128M below 6k | 0.868636 | 0.817 s | about 0.98 bp |
| 128M below 6k, 64M 6k-10k | **0.868531** | **0.814 s** | **about 1.35 bp** |

The first point proves that the deeper low-dimensional search is positive but
not promotion-sized alone. Extending 128M through the complete already-accepted
lower-medium band nearly reaches the threshold. Adding 64M to the upper-medium
band supplies the final 0.000105 reduction in its bucket and moves the full
candidate safely past one basis point.

The 0.003-second worst-time difference between the final two rows is ordinary
measurement noise. The structural fact is that both remain around 0.81 seconds
on the same machine and corpus, far below the 2-second watchdog. No claim rests
on that tiny ordering.

## Implementation

The complete functional delta from promoted 0053 is this replacement:

```rust
// 0053
cfg4.budget = if (1_000..6_000).contains(&n) {
    64_000_000
} else {
    32_000_000
};

// 0054
cfg4.budget = if (1_000..6_000).contains(&n) {
    128_000_000
} else if (6_000..10_000).contains(&n) {
    64_000_000
} else {
    32_000_000
};
```

Everything else remains unchanged:

- rounds 2 and 3 use 8M per block;
- round 4 retains deterministic seed/round 3, 32 blocks, `min_s = 16`, and
  `max_s = 768`;
- round 5 retains deterministic seed/round 4, 32 blocks, and 16M per block;
- the terminal primary and follow-up searches retain accepted seeds 5, 6, 7;
- small and large matrices retain the hidden-proven 32M round-4 budget;
- all incumbent checks, exact re-scoring, and permutation validation remain.

The range split uses only `Pattern::n()`, not matrix identity, values, corpus
position, or any target answer. It is a general dimension-based resource policy
for unseen patterns from the same distribution.

## Score result

The exact bucket comparison is:

| bucket | promoted 0053 | candidate 0054 | change |
|---|---:|---:|---:|
| lt_1k | 0.893224 | 0.893224 | 0 |
| 1k_10k | 0.868912 | **0.868531** | -0.000381 |
| gt_10k | 0.792071 | 0.792071 | 0 |
| weighted score | 0.845469 | **0.845355** | **-0.000114** |

Small and large are exact controls because neither can reach a changed branch.
The primary score delta is the medium-bucket reduction times its 0.30 weight,
subject only to retained scorer precision.

The fill tiebreak changes from 0.944729 to 0.944744, a small regression of
0.000015. The challenge ranks on predicted factorization flops first, and the
flop improvement clears the stated promotion threshold; no optimization claim
is made for fill.

The lower-medium gains are distributed across multiple public structures. The
deeper point improves examples such as `chimera_lga-01`, `chimera_rfr-02`,
`popdynm25`, and `rsyn0805m03m`. The upper-medium tier improves examples such as
`crudeoil_lee2_06`, `rsyn0820m04m`, `rsyn0840m04m`, and `arki0016`. There are
also individual regressions because a longer deterministic trajectory can end
in a different basin. The complete bucket geomean, not selected examples, is
the acceptance criterion.

## Timing and risk

The strongest evidence for timing safety is measured rather than inferred:

```text
medium matrices: 108
medium total:    37.0 s
medium worst:    0.814 s on chp_shorttermplan1a
```

The accepted 0053 source measured a 0.661-second medium worst, so this candidate
adds visible but bounded work. It remains below one second in the focused probe
and below the public cap in the complete benchmark. The candidate never gives
more than 32M to small or large inputs, which removes the two buckets most
clearly implicated by the failed global policy: small did not benefit, while
large contained the largest states and highest public worst.

There is still genuine hidden risk. In particular, 128M has not previously
been accepted, and 64M in the upper-medium band is new. The argument is not
that the candidate is risk-free; it is that the work is bounded by dimension,
the measured worst is 0.814 seconds, the full trusted run passes, and the
existing #1 remains promoted if this candidate fails. A timeout must close this
raw-depth direction entirely rather than trigger another cutoff retry.

## Verification

Executed from the benchmark work directory on the Mac Studio:

```sh
cargo test -p ssi-candidate-worker --release
bash scripts/local-candidate-build.sh && cargo run --release
```

The release suite passed 25 active tests, with 16 ignored and zero failures.
The full trusted 300-matrix command completed successfully. That command covers
the sandboxed source build, source scan, pinned dependency and license checks,
optimized candidate execution, determinism rerun, permutation validation,
per-matrix watchdog, and trusted factorization-cost scorer.

Exact full result:

```text
OK  0.845355  0.944744
```

Before the final documentation-only comment adjustment, both machines matched
on the functionally tested source. After that adjustment, the final submitted
`src/ordering/mod.rs` hash on both machines is:

```text
sha256 a2eb9d522a05b928760579181af366eeda82c6089401507a5a23961a3c48316a
```

## Rule compliance

- All changes are under `src/ordering/`.
- The entrypoint remains `pub fn order(pattern: &Pattern) -> Vec<usize>`.
- Only the sparsity pattern is observed.
- The returned ordering is deterministic and validated as a permutation.
- Rust standard library and existing challenge modules only.
- No dependency, manifest, lockfile, build-script, environment, network,
  subprocess, clock, filesystem, FFI, or thread-count changes.
- No matrix-name checks, lookup table, stored order, corpus-position branch, or
  hidden metadata.
- The grader independently recomputes predicted LDLT factorization cost.

## Decision and stopping rule

Submit because this is a complete, reproducible 1.35-basis-point primary-score
improvement in a three-times-hidden-validated family, with a focused worst call
of 0.814 seconds. If it promotes, preserve the exact tiers and move away from
raw round-4 budget increases unless a fundamentally new fixed-work allocation
appears. If it fails timing, restore promoted 0053 immediately and close 128M
and the upper-medium 64M extension. If it scores but misses promotion, retain
0053 and require a larger independent gain before another submission.
