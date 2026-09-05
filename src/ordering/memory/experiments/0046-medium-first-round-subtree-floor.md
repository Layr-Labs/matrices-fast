# Experiment 0046: medium first-round subtree floor

## Status

Rejected after submission: a hidden matrix exceeded the 2.0-second per-matrix
cap, so the grader killed `order()` before producing a score. Submission
`b925cc0a-77c2-458e-a89a-899469b5e941`, commit `93f7fd6`, failed in the
Benchmark step after 9m22s total workflow time. This is a deliberately bounded
change on top of the current hidden-accepted frontier: lower the medium tier's
first-round subtree floor from 32 to 24 while retaining every nominal work cap,
allocation, and behavior in the small and large tiers. Lower predicted
factorization flop ratio is better.

## Frontier and objective

The promoted frontier at commit `fed9a10` records:

```text
development score = 0.847732
hidden score      = 0.873711
```

Its first-round subtree allocation is size-tiered:

| tier | `min_s` | `max_s` | blocks x budget |
|---|---:|---:|---:|
| `n < 1,000` | 16 | 256 | 16 x 2M |
| `1,000 <= n < 10,000` | 32 | 128 | 16 x 2M |
| `n >= 10,000` | 32 | 384 | 16 x 2M |

The preceding deep-small experiments are closed. Multiple attempts increased
per-stream depth on small matrices, improved the public score, and exceeded the
hidden two-second cap. This candidate does not retain any of those attempts. It
starts from the hidden-passing 16-by-2M allocation and searches for a score gain
without increasing a budget, a block cap, or the maximum eligible subtree size.

## Hypothesis

The medium corpus has elimination trees where the default 32-vertex subtree
floor excludes useful small blocks. The later chain rounds already use a
16-vertex floor, so admitting first-round blocks between 24 and 31 vertices may
find inexpensive improvements before those vertices are transformed by the
first elimination pass.

The expected safety advantage is structural. Changing `min_s` changes only
which already-bounded subtrees can be considered. It does not deepen local
search, add streams, add rounds, enlarge a subtree, or raise the number of
blocks. The selected configuration remains capped at 16 blocks by 2M word
operations, with `max_s = 128`.

## Controlled sweep

All measurements below use the same source, corpus, release profile, promoted
allocation, and medium-bucket probe. The only swept field is the first-round
medium `min_s`. The probe contains all 108 public matrices with
`1,000 <= n < 10,000` and was run on an otherwise idle 16-logical-core,
64-GiB arm64 Mac Studio.

| medium first-round `min_s` | medium flop geomean | decision |
|---:|---:|---|
| 8 | 0.872115 | near-tie, admits the broadest set |
| 16 | 0.872120 | near-tie, broader than necessary |
| **24** | **0.872108** | **selected best measured result** |
| 28 | 0.872421 | most of the gain disappears |
| 32 | 0.872433 | promoted control |

The improvement is not monotonic because eligible subtrees compete for a fixed
16-block allocation and each accepted local result changes subsequent graph
state. Twenty-four is both the best measured setting and narrower than 8 or 16:
it admits fewer additional candidates while producing a slightly better flop
geomean. The 28 result provides a useful negative control showing that most of
the improvement comes from blocks in the 24-to-27 range.

The selected focused result was:

```text
1K_10K_GEOMEAN = 0.872108 (count 108)
1K_10K_WORST   = 0.510 s on arki0016
1K_10K_TOTAL   = 29.0 s
```

The promoted medium control is 0.872433. Holding the other bucket contributions
fixed, the weighted score projection is approximately 0.847635, an improvement
of about 0.000097 over the 0.847732 frontier, or roughly 1.15 basis points.
The official full-corpus result is `0.847635`, matching that projection.

## Exact implementation

Only the medium branch of `subtree_cfg_for(n)` changes:

```rust
} else {
    cfg.min_s = 24;
    cfg.max_s = MID_MAX_S;
    cfg.max_blocks = MID_BLOCKS;
    cfg.budget = MID_BUDGET;
}
```

The effective first-round configuration is therefore:

```text
if 1,000 <= n < 10,000:
    min_s      = 24
    max_s      = 128
    max_blocks = 16
    budget     = 2,000,000
    streams    = unchanged
```

No `nnz` gate, matrix identifier, lookup table, hidden answer, or corpus-specific
special case is used. Selection depends only on the pattern dimension already
used by the promoted tier configuration.

## What remains unchanged

- Small matrices retain `min_s = 16`, `max_s = 256`, and 16 blocks by 2M.
- Large matrices retain `min_s = 32`, `max_s = 384`, and 16 blocks by 2M.
- Medium `max_s`, `max_blocks`, `budget`, stream count, and later-round behavior
  are unchanged.
- Later chain rounds already override `min_s` to 16 in the promoted algorithm;
  this patch affects only the first round.
- Graph construction, candidate scoring, strict best-order replacement,
  deterministic tie behavior, and permutation return logic are unchanged.
- The previously rejected small deep-search variants are absent.

This scope matters for hidden-runtime risk. The current frontier passed hidden
evaluation with the same operation ceiling. The candidate does not reproduce
the failure mode of experiments 0043 through 0045, which raised small-matrix
per-block depth. It merely lets the existing fixed allocation consider a few
more medium subtrees.

## Runtime argument

The clean focused sweep showed a 0.510-second maximum on `arki0016` at the
selected threshold. That is descriptive public evidence, not a guarantee about
the hidden corpus. The stronger safety argument is the unchanged deterministic
work ceiling:

- maximum eligible medium subtree remains 128 vertices;
- maximum accepted first-round block count remains 16;
- per-block word-operation budget remains 2M;
- there are no additional streams or chain rounds;
- the small and large code paths are byte-for-byte behaviorally unchanged;
- only the minimum eligible first-round medium block size changes from 32 to 24.

The broader 8 and 16 thresholds were not selected even though their scores were
close, because they admit more candidate subtrees without a measured scoring
advantage. Twenty-four is the narrowest tested threshold that captures the full
public improvement.

## Full-corpus verification

The final exact full-corpus command was:

```text
bash scripts/local-candidate-build.sh && cargo run --release
```

It completed all 300 matrices successfully:

```text
OK    score=0.847635    fill=0.946272

bucket    count     flop_geomean     fill_geomean
lt_1k       147           0.8939           0.9624
1k_10k      108           0.8721           0.9580
gt_10k       45           0.7946           0.9254
```

The exact score improves the current `0.847732` frontier by `0.000097`, about
1.14 basis points relative. The run uses the repository's isolated candidate
build, purity scan, pinned `cargo-deny 0.20.2` license check, release worker,
deterministic repeat check, permutation validation, and trusted flop scorer.

The focused compile and probe completed successfully. The final release unit
suite passed 25 tests, with 16 intentionally ignored research probes and zero
failures. Compilation reports four pre-existing dead-code warnings for unused
relabel constants/function and two unused score variants. There are no new
dependency, license, purity, permutation, determinism, scorer, or build failures.

## Rule compliance

- Only files under `src/ordering/` are changed.
- `order(pattern: &Pattern) -> Vec<usize>` remains the sole candidate entrypoint.
- Rust standard library and the challenge's existing allowed local modules only.
- No manifest, lockfile, dependency, build-script, FFI, network, subprocess,
  filesystem, clock, randomness, or environment-variable behavior is added.
- The algorithm sees only the supplied sparsity pattern.
- It returns a deterministic permutation of every vertex.
- The trusted grader recomputes predicted factorization cost from the returned
  permutation; no score is self-reported by the candidate.

Verification receipts:

```text
cargo test -p ssi-candidate-worker --release
  PASS: 25 passed; 0 failed; 16 ignored

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: 300 matrices; score 0.847635; fill 0.946272
```

## Decision

Do not resubmit `min_s = 24` with the full 16-by-2M allocation. Although it
clears the promotion-sized public threshold and keeps the nominal cap unchanged,
the lower floor increases the number of eligible blocks on at least one hidden
matrix enough to violate the wall-clock limit. Any continuation must reduce the
actual medium allocation materially below 32M, not merely rely on an unchanged
nominal ceiling. Do not include the hidden-unsafe deep-small variants, and do not
broaden the medium floor to 8 or 16.

Official failure receipt:

```text
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```
