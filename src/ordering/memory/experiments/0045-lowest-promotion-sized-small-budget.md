# Experiment 0045: lowest promotion-sized sparse-small depth

## Status

Rejected after submission: the hidden run still exceeded the two-second cap.
The exact official development result was `0.847631`, fill
tiebreak `0.946430`. The promoted frontier is `0.847732` on development and
`0.873711` on hidden evaluation. Lower is better.

## Motivation

Two preceding submissions established a hard hidden-runtime boundary that the
public corpus did not expose:

1. Experiment 0043 used four blocks by 8M operations on every matrix with
   `n < 1,000`. It scored `0.848766` on the older frontier and passed the full
   public corpus, but hidden submission `794e1fc0` exceeded the two-second cap.
2. Experiment 0044 restricted the same four-by-8M search to patterns with both
   `n < 1,000` and `nnz <= 10,000`. It scored `0.847551` after rebasing onto
   the new promoted upper-tier allocation, but hidden submission `32961a0c`
   also exceeded the cap.

The second result is decisive: density gating alone is insufficient. At least
one sparse hidden small pattern makes an 8M local-search stream unsafe. This
experiment therefore reduces per-block depth and finds the lowest measured
budget that still produces a promotion-sized public improvement.

## Promoted base retained

The current promoted algorithm uses a size-tiered subtree configuration:

| tier | `max_s` | blocks x budget |
|---|---:|---:|
| `n < 1,000` | 256 | 16 x 2M |
| `1,000 <= n < 10,000` | 128 | 16 x 2M |
| `n >= 10,000` | 384 | 16 x 2M |

Experiment 0045 preserves the promoted medium and large behavior. Dense small
patterns also preserve the promoted 16-by-2M behavior. Only the following
structurally bounded branch changes:

```text
if n < 1,000 and nnz <= 10,000:
    min_s      = 16
    max_s      = 256
    max_blocks = 4
    budget     = 6,625,000
    streams    = 1
```

The changed path requests at most 26.5M word operations, 17.2% below the 32M
ceiling used by the promoted configuration and the two failed submissions.
There is no increase elsewhere.

## Budget boundary sweep

All measurements below use four blocks, `min_s = 16`, `max_s = 256`, one
stream, and the `n < 1,000 && nnz <= 10,000` gate. The focused probe covers all
147 public small-bucket matrices on the idle arm64 verification host.

| per-block budget | total requested work | small flop geomean | decision |
|---:|---:|---:|---|
| 6.000M | 24.0M | 0.893777 | safe-shaped but below promotion margin |
| 6.500M | 26.0M | 0.893640 | below promotion margin |
| **6.625M** | **26.5M** | **0.893556** | **selected lowest passing point** |
| 6.750M | 27.0M | 0.893318 | stronger score, less safety |
| 7.000M | 28.0M | 0.893274 | stronger score, less safety |
| 8.000M | 32.0M | 0.893287 | hidden timeout twice; rejected |

The curve is discontinuous because a local improvement appears only after a
search stream completes the relevant move sequence. The selection rule is
therefore simple: choose the lowest tested budget whose weighted overall score
clears approximately one basis point against the current frontier. Do not
choose the numerically best public point when two hidden runs already show
that excess depth is unsafe.

Additional shape checks did not provide a safer route:

- Five blocks by 6M is byte-for-byte score-identical to four blocks by 6M;
  the useful public trees never reach a fifth eligible block.
- Six blocks by 5M regresses the small geomean to `0.894012`.
- Setting `min_s = 8` at four blocks by 6M is score-identical to `min_s = 16`.

Those alternatives are not present in the candidate.

## Score

Focused small bucket:

```text
LT1K_GEOMEAN = 0.893556 (count 147)
LT1K_WORST   = 0.322 s on multiplants_stg1b
LT1K_TOTAL   = 11.0 s
```

Official full 300-matrix development result:

```text
OK    score=0.847631    fill=0.946430
```

Rounded bucket report:

| bucket | count | flop geomean | fill geomean |
|---|---:|---:|---:|
| `lt_1k` | 147 | 0.8936 | 0.9622 |
| `1k_10k` | 108 | 0.8724 | 0.9587 |
| `gt_10k` | 45 | 0.7946 | 0.9254 |

The current promoted development score is `0.847732`; the candidate improves
it by `0.000101`, about 1.19 basis points relative to the frontier. The entire
delta comes from the sparse-small branch. The medium and large buckets are the
new hidden-accepted promoted allocation.

## Runtime case

Public maximum time alone is not treated as proof of hidden safety. Both 8M
submissions had comfortable public small-bucket timings and still failed the
hidden cap. The relevant change in 0045 is the deterministic operation budget:

- per-block depth decreases from 8M to 6.625M, a 17.2% cut;
- maximum changed-path work decreases from 32M to 26.5M, also 17.2%;
- the `n < 1,000` and `nnz <= 10,000` gates remain;
- every pattern outside those two bounds uses the promoted hidden-passing
  allocation;
- no later chain round receives more work than the promoted source.

The exact source completes the full public harness on an idle 16-logical-core,
64-GiB arm64 host. The changed-path worst public measurement is 0.322 seconds.
The hidden grader remains authoritative, and the two prior negative receipts
are preserved rather than being presented as successes.

## Implementation details

`subtree_cfg_for` accepts both `n` and `nnz`. Each of its five existing call
sites passes the already computed nonzero count. This adds no graph traversal
or allocation. The function begins with the promoted tier configuration, then
overrides only `max_blocks` and `budget` for the bounded sparse-small case.

The selection is distribution-level rather than corpus-specific. It does not
read matrix names, values, right-hand sides, reference orders, scores, files,
environment variables, clocks, or randomness. It depends only on two basic
properties of the supplied sparsity pattern.

## Correctness and rule compliance

- Only `src/ordering/` is modified.
- Rust standard library and the existing allowed challenge modules only.
- No manifest, dependency, lockfile, build script, FFI, network, thread,
  subprocess, or filesystem changes.
- `order()` still returns a deterministic permutation of `0..n`.
- Each candidate replaces the best ordering only after a strict predicted-flop
  reduction.
- The scorer independently validates the bijection and recomputes cost.
- The configured sparse-small work maximum is below the repository's 32M test
  limit.

Verification receipts:

```text
cargo test -p ssi-candidate-worker --release probe_lt1k -- --ignored --nocapture
  PASS: 1 passed; small geomean 0.893556; worst 0.322 s

bash scripts/local-candidate-build.sh && cargo run --release
  PASS: 300 matrices; score 0.847631; fill 0.946430
```

The official run includes the purity scan and pinned `cargo-deny 0.20.2`
license check. Compilation emits four pre-existing dead-code warnings; there
are no build, license, permutation, determinism, or scorer failures.

## Decision

Submit 6.625M, not 6.75M or 7M. The latter two buy a larger public cushion but
move back toward the hidden-failed depth. This competition is gated by the
worst private matrix, so the lowest measured promotion-sized budget is the
correct risk-adjusted candidate.

If this still exceeds the hidden cap, close this entire deep-small-search axis
and revert to the promoted 16-by-2M small configuration. Further reductions
below 6.625M do not meet the public promotion bar, and repeated submissions
would no longer be evidence-driven.
