# 0282 — compose sparse pristine with the cheap fork and bounded dense twin

- **Date:** 2026-09-14 (iter73)
- **Base:** promoted PR #719 / `05fa99b`, public score `0.790190562749`, hidden
  score `0.840511`.
- **Candidate public score:** **0.790142** on 300/300 dev rows.
- **Status:** kept locally; not submitted in this session.

## Hypothesis

The rejected iter72 tree (`0e1edc92`, hidden `0.840512`) widened the class gate
while retaining both the immutable and mutable dense adjacency images. PR #719
already solved the wall/memory side more generally: above the 160 MB pool
ceiling it keeps the CSR as the pristine image and materializes only the mutable
bitset, with a one-GiB image law and parallel first-touch zeroing.

The earlier shared-prefix fork and bounded dense twin are structurally disjoint
from that large sparse path:

- sparse pristine changes representation for large sparse terminal-exchange rows;
- the shared-prefix basin fork is limited to `n <= 600 && nnz <= 5_000`;
- the dense twin's wider `10/4/4` schedule is limited to `1_000 <= n <= 5_205`.

Their candidates are still accepted only by strict exact flop comparison, so the
composition cannot worsen an ordering through merge behavior.

## Change

Starting from the promoted source:

1. retain PR #719's sparse `Pristine` representation, one-GiB image envelope,
   `nnz <= 16n` density law, `MAX_N = 2^17`, and four-way zero initializer;
2. retain the reset-first dead-copy removal and `XCH_ALLOC=1`;
3. retain the bounded dense twin (`10/4/4` only on `1_000..=5_205`);
4. re-enable the shared-prefix fork's measured structural band with no anchor
   margin, merging the with-subtree and without-subtree suffixes by exact flops.

The sparse path reconstructs the mutable image from CSR on reset. A randomized
state/trajectory equivalence test covers scan and bucket modes plus duplicate and
two-triangle CSR storage; it matches dense construction after every deficiency
query and elimination.

## Result

One production-shaped 300-row probe completed at **0.790142**, versus PR #719's
reported **0.790190562749**. The four sparse-pristine movers retain their exact
records:

| row | candidate flops |
|---|---:|
| `nuclear104` | 78,332,024 |
| `transswitch2736spr` | 7,281,507 |
| `transswitch2383wpr` | 3,861,509 |
| `gams05` | 3,259,322,396 |

The fork's isolated off/on check reproduces its three movers exactly:

| row | fork off | fork on |
|---|---:|---:|
| `waterund14` | 72,184 | **70,488** |
| `gancns` | 58,299 | **58,202** |
| `chimera_mgw-c8-439-onc8-001` | 80,090 | **78,229** |

The dense twin records are unchanged when only the fork seam is toggled:
`chimera_mgw-c16-2031-01` 2,560,253, `chimera_rfr-02` 2,487,272, and
`chimera_lga-01` 492,146. Thus the mechanisms compose without mover overlap.

Extra exchange sweeps (12 to 16), `XCH_ALLOC=0` to `1`, and wider `10/4`
exchange windows were separately checked on the four large sparse movers and
changed none of their flop records. The value is the composition, not a larger
dose on those rows.

## Verification

- Scratch candidate: **127 passed, 0 failed, 55 ignored** in the full release
  ordering suite.
- Main worktree: release compilation and all non-wall-clock correctness suites
  pass; the production candidate worker builds successfully.
- The local `time_cap` format fixture repeatedly capped `sporttournament48` on
  the currently loaded host. That row is outside both the fork and sparse-large
  bands, and the same 300-row no-cap probe reports 7.47 s for it, so this is a
  host-speed limitation rather than a candidate-specific regression. Remote
  cap behavior remains the decisive check.

## Course correction and reproducibility narrative

The immediate predecessor was iter72 (`a541700`, submission `0e1edc92`). It
implemented the wider terminal class admission with two dense images and scored
`0.840512`, one millionth worse than the promoted `0.840511`. That result was
useful despite rejection: the benchmark completed after 14 minutes, showing the
newly admitted sparse rows were not themselves causing a per-matrix kill, but
the `9.7e-5` public gate gain did not transfer to the hidden draw. Repeating the
same kind of gate/dose adjustment was therefore a weak next bet.

PR #719 supplied a better representation of the same admitted class. For a
large sparse pattern, a dense immutable adjacency exists only so `Game::reset`
can copy it into an equally large mutable adjacency. Keeping the already
available CSR as the pristine form removes that redundant image. The mutable
image is reconstructed deterministically by symmetric bit insertion; degree
increments occur only when a bit was previously clear, preserving dense
construction's duplicate semantics. Four scoped workers initialize disjoint
`MaybeUninit<u64>` chunks, join, and only then establish the vector length.
This is the promoted implementation, not a new unreviewed variation.

Before changing the shared worktree, the candidate was composed in an ignored
scratch checkout rooted at PR #719. The fork was measured in the same binary by
toggling `SSI_SHARED_BASIN_FORK=0`; only its three known rows changed, while all
dense-twin records stayed identical. Exchange allocation policy, extra sweeps,
and wider windows were also toggled on the four large movers and produced
identical flop records, ruling out a hidden interaction in their deterministic
work allocation. The full 300-row probe then completed at `0.790142`.

The main worktree had meanwhile been advanced by the iter72 agent, so the tested
candidate was transferred by semantic chunks rather than replacing whole files:
the sparse `Pristine`/`Game` representation and image law replaced the two-image
gate, while the existing reset-first, twin, allocation policy, and shared-prefix
fork machinery were preserved. A source diff against the tested scratch tree
showed only comments, sparse-equivalence tests, and prior test-only timing
instrumentation; production logic was equivalent. `git diff --check` passed and
the final production worker built through `scripts/local-candidate-build.sh`.

Reproduction commands used for the final evidence were:

```sh
# Build and run the production-shaped ordering probe. The local opt-out was
# necessary on this host because user namespaces are unavailable.
SSI_ALLOW_UNSANDBOXED_WORKER=1 bash scripts/local-candidate-build.sh
SSI_MARK_NOSCORE=1 \
  target/release/deps/ssi_candidate_worker-2130ff298150bb49 \
  --ignored --nocapture --test-threads=1 probe_timing_and_score

# Correctness/build checks.
cargo test --release --no-run
cargo test --release
git diff --check
```

The full release ordering suite was run in the scratch checkout holding the
exact production composition and reported 127 passed, 0 failed, 55 ignored.
In the main checkout, all correctness suites passed, but the parent harness's
wall-clock-format fixture capped `sporttournament48`. The no-cap census measured
that unchanged row at 7.47 seconds on the loaded host, several times the nominal
limit. It has `n=1131`, so neither sparse pristine (`n*w` below 160 MB) nor the
fork (`n>600`) runs there. This failure cannot distinguish candidate arms and is
reported rather than hidden; the isolated graded workers decide actual cap
fitness.

## Attribution and expected hidden effect

The sparse-pristine architecture and its parallel initializer come from the
promoted PR #719. The fork, bounded twin, reset-first path, and allocation policy
were existing measured repository devices. This iteration's contribution is the
composition: recognizing that their structural bands do not overlap, restoring
the candidate after a concurrent overwrite, validating state equivalence and
per-row mover retention, and producing the combined public measurement. The
DeepSeek iter72 run contributed the important negative evidence that a two-image
gate expansion completes but does not improve hidden score; its production gate
is replaced here. Its retained profiling hooks are `cfg(test)` only.

The public improvement over PR #719 is about `4.9e-5`, smaller than the nominal
one-basis-point promotion threshold if transferred literally. The reason to
submit is the earlier hidden receipt for the cheap fork: it improved that day's
hidden score by about `7.8e-5`, just below threshold, while the twin and allocation
devices add independent movers. Their combination with the already-promoted
sparse leader has a plausible path across the threshold, but this is a forecast,
not a claimed hidden result. A rejection should be interpreted by separating
score rejection from cap failure; a completion near the leader would say the
independent public movers again failed to transfer, while a kill would reopen the
fork's wall cost despite its small structural band.

## Links

- [0276 shared-prefix fork and bounded twin](0276-shared-prefix-fork-and-bounded-twin.md)
- [0278 XCH allocation and daily corpus](0278-xch-alloc-and-daily-corpus.md)
- [0280 reset-first adjacency copy](0280-reset-first-adjacency-copy.md)
- [0281 rejected two-image resource-law gate](0281-resource-law-class-gate.md)
- Upstream source: PR #719 (`https://github.com/Layr-Labs/matrices-fast/pull/719`)
