# 0285 — price a fork mover to buy a wider cap margin

- **Date:** 2026-09-15 (iter77)
- **Base:** iter76 `8c8178e` / submission `502f790c` / PR #740.
- **Change:** shared-basin fork AMD-anchor margin `14% -> 20%`.
- **Public score:** predicted exactly from two complete same-binary arms and a
  focused boundary sweep: `0.790166161318 -> 0.790169175640` (**+3.014e-6**).
- **Status:** verified; production candidate build clean, final release suite
  `127 passed / 0 failed / 56 ignored`.

## Hidden evidence changes the objective

Iter76 removed one of four width-10 dense-twin sweeps and raised the fork's
pre-suffix AMD margin from 10% to 14%. It was submitted as
`502f790c-790e-4c39-a73f-6556aa91ff81` / PR #740. Benchmark began after the
final build and corpus fetch at `23:38:19.762530Z`; the grader reported a hidden
two-second cap failure at `23:39:43.184244Z`, about **83.422 seconds** later.

That receipt makes the old constraint—retain every known public fork mover—the
wrong optimization target. Fork-bearing compositions have now failed the cap
repeatedly at margins of 0%, 10%, and 14%. The last completing tree predates the
fork. A useful next point must trade a small amount of measured value for a
material reduction in fork admission.

## Complete fork-off price

The iter76 test binary was run over all 300 public rows with the production-
shaped scoring frame and `SSI_SHARED_BASIN_FORK=0`. Its exact score was:

```
lt_1k    count=147   geomean=0.887274030819
1k_10k   count=108   geomean=0.837329958574
gt_10k   count=45    geomean=0.682186983464
SCORE = 0.790255990203
```

The iter76 baseline in the same binary has exact score `0.790166161318`.
Comparing the 300 `COUNTS` records finds exactly three changes:

| row | n | iter76 fork | fork off | lost improvement |
|---|---:|---:|---:|---:|
| `waterund14` | 333 | 70,488 | 72,184 | 1,696 |
| `gancns` | 548 | 58,202 | 58,299 | **97** |
| `chimera_mgw-c8-439-onc8-001` | 440 | 78,177 | 80,090 | 1,913 |

The other **297/300 records are identical**. This complete off arm bounds both
the mechanism's total public value and the set of rows that any stronger margin
can change. Retiring the fork entirely costs `+8.983e-5`, almost a full
promotion threshold and enough to erase the candidate's public advantage over
the promoted source. The whole fork should therefore not be removed before
pricing its movers individually.

## Value per measured cap cost

The production-worker receipt in experiment 0278 measured the three fork
movers' added wall against the fork-free source:

| row | fork improvement in log ratio | measured added wall |
|---|---:|---:|
| `waterund14` | -2.378e-2 | about +0.17 s |
| `chimera_mgw-c8-439-onc8-001` | -2.351e-2 | about +0.19 s |
| `gancns` | -1.665e-3 | about **+0.35 s** |

`gancns` is the dominated point: it contributes only 97 flops and roughly
`3.014e-6` to the full score, yet it has the largest measured added wall of the
three. The two other movers each provide about fourteen times as much log-ratio
value for roughly half the added wall. Once a hidden receipt says the cap is
still binding, preserving `gancns` is no longer defensible.

This comparison is used as mechanism evidence, not as a claim that local wall
seconds reproduce the grader. The production change is a deterministic
admission reduction: rows in the removed margin band do not allocate a second
suffix workspace and do not execute that suffix.

## Find the next pre-suffix boundary

The test seam `SSI_FORK_MARGIN_PCT` was swept on all three fork movers at the
actual pre-suffix checkpoint. Returned flop counts were stable as follows:

| margin | `waterund14` | `gancns` | c8 Chimera |
|---:|---:|---:|---:|
| 15% | 70,488 | 58,299 | 78,229 |
| 16% | 70,488 | 58,299 | 78,229 |
| 18% | 70,488 | 58,299 | 78,229 |
| 20% | 70,488 | 58,299 | 78,229 |
| 21% | 70,488 | 58,299 | 80,090 |
| 22%--27% | 70,488 | 58,299 | 80,090 |

The focused c8 Chimera count is 78,229 rather than the full process's 78,177.
That fork has previously shown scheduling-sensitive candidate selection between
focused and full probe processes; both values are improvements over the
fork-free 80,090, and the margin boundary is the same deterministic admission
decision. Margin 20 is the largest measured whole-percent value that retains
the two high-value movers. Margin 21 crosses the c8 Chimera's pre-suffix
checkpoint and is therefore rejected.

## Exact full-score derivation

The complete iter76 and fork-off arms prove that the fork changes only the three
rows above. The margin sweep proves that margin 20 retains `waterund14` and the
c8 Chimera while making `gancns` identical to the complete fork-off arm. Thus a
third 300-row run is unnecessary to know every returned count: take iter76's
300 records and replace only `gancns` 58,202 with 58,299.

Recomputing the official bucket geomeans gives:

```
lt_1k    count=147   geomean=0.886984648942
1k_10k   count=108   geomean=0.837329958574
gt_10k   count=45    geomean=0.682186983464
SCORE = 0.790169175640
```

The public price versus iter76 is **+0.000003014322**. The candidate remains
about `2.14e-5` better than the promoted source's public provenance
`0.790190562749`, although public and hidden deltas have not transferred
reliably on the rotating evaluation corpus.

## Safety and expected trade

The edit changes one production constant. The structural gate remains
`6 <= n <= 600 && nnz <= 5,000`; exact score comparison, bijection checks,
deterministic base-tie behavior, and both suffix implementations are unchanged.
No row gains work. A row whose pre-suffix incumbent is 80% or more of AMD now
runs only the base suffix. Rows below that threshold behave exactly as in
iter76.

Compared with 14%, the new gate removes the entire pre-suffix ratio band
`[0.80, 0.86)`. Compared with turning the fork off, it retains the two public
movers that carry 96% of the fork's measured log-ratio improvement. This is the
first cap correction in the sequence that explicitly sacrifices a measured
mover according to value per cost, rather than tightening only through a free
part of the margin curve.

## Final verification

The production candidate built successfully through the repository candidate
build script. The final release suite on the exact iter77 source reported:

```
test result: ok. 127 passed; 0 failed; 56 ignored
```

A final focused probe with no margin override confirmed that the compiled test
default is the intended production constant: `waterund14` 70,488, `gancns`
58,299, and the c8 Chimera row 78,229. The trusted parent release build and
`git diff --check` also completed successfully.

## Evidence

- `.session-backup/iter76-dense-s3-full.log`
- `.session-backup/iter77-fork-off-full.log`
- focused margin sweep in the iter76 test binary, margins 15--27
- iter76 hidden receipt: PR #740 / workflow `34909483933`
- prior production cost receipt:
  `memory/evidence/0276-graded-frame-reps2.log`

## Links

- [0284 dense sweep and 14% fork trim](0284-cap-trim-dense-sweep-and-fork-margin.md)
- [0278 fork anchor margin](0278-fork-anchor-margin.md)
- [0276 shared-prefix fork and bounded twin](0276-shared-prefix-fork-and-bounded-twin.md)
