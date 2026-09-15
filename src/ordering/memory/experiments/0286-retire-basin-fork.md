# 0286 — retire the basin fork after the cost curve fails hidden

- **Date:** 2026-09-15 (iter78)
- **Base:** iter77 `75b2cf4` / submission `b84237f6` / PR #743.
- **Change:** the shared-basin fork is off in production and becomes an
  explicit opt-in test seam.
- **Public score:** measured on all 300 rows, **0.790255990203**.
- **Status:** verified; production candidate build clean, final release suite
  `127 passed / 0 failed / 56 ignored`.

## Why the fork is now retired

The shared-prefix fork was designed to obtain two terminal basins without
duplicating the expensive common prefix. It runs the normal suffix with the
stage-4 subtree cascade and, only when that cascade strictly changes the
checkpoint, runs a second suffix from the pre-subtree ordering. A final exact
flop comparison selects the result. The mechanism is value-positive on three
public rows and cannot worsen an accepted ordering.

Its cap history is nevertheless decisive. The current composition has now
failed hidden evaluation with four different admissions:

| iteration | fork admission | hidden result |
|---|---|---|
| iter73 | structural band, no AMD margin | cap failure |
| iter74 / iter75 | incumbent at least 10% below AMD | cap failure |
| iter76 | incumbent at least 14% below AMD | cap failure, 83.422 s |
| iter77 | incumbent at least 20% below AMD | cap failure, 84.509 s |

The 20% point deliberately removed `gancns`, the most expensive measured mover,
and excluded the entire pre-suffix ratio band `[0.80, 0.86)`. Its hidden failure
arrived at essentially the same corpus position as the 14% point. Further
margin tuning would either repeat the same experiment or remove the c8 Chimera
mover at 21%. The useful causal test is the endpoint: no second suffix in
production.

This is also supported by the strongest positive control available. The last
tree from this lane that completed the hidden corpus was iter72, before the fork
composition. It scored 0.840512, one millionth behind the promoted 0.840511.
The promoted sparse-pristine source also completed without this fork. Retiring
the fork brings the suffix topology back to those completing trees while
retaining the later sparse representation and bounded dense-twin work.

## Exact public price

Before changing production, the iter76 binary was run on the complete 300-row
public corpus with `SSI_SHARED_BASIN_FORK=0`. The official bucket calculation
was:

```
lt_1k    count=147   geomean=0.887274030819
1k_10k   count=108   geomean=0.837329958574
gt_10k   count=45    geomean=0.682186983464
SCORE = 0.790255990203
```

Against iter76's complete 300-row arm, exactly three `COUNTS` records change:

| row | n | fork on | fork off | lost flops |
|---|---:|---:|---:|---:|
| `waterund14` | 333 | 70,488 | 72,184 | 1,696 |
| `gancns` | 548 | 58,202 | 58,299 | 97 |
| `chimera_mgw-c8-439-onc8-001` | 440 | 78,177 | 80,090 | 1,913 |

The other **297 rows are bit-identical in their flop records**. The fork-off
score is `+8.983e-5` worse than iter76 and `+8.682e-5` worse than iter77. That is
roughly one promotion threshold, so this is a cap-first diagnostic submission,
not a claim that the score trade is free.

The candidate remains within `6.54e-5` of the promoted source's public
provenance (`0.790190562749`). Public deltas have not predicted the rotating
hidden corpus reliably: iter72 was publicly better than both yet tied the crown
to six decimal places. A completion is therefore more informative than another
fork margin point even if the result is score-rejected.

## What remains enabled

The production change affects only the fork admission function. It retains:

- the promoted one-image sparse-pristine representation and one-GiB resource
  law;
- the output-invariant first-reset reuse and sparse mutable-image retention;
- `XCH_ALLOC=1`, which continues scanning after an unfundable component;
- the width-10 dense/hub twin on `1,000 <= n <= 5,205`, now with three sweeps;
- the restored `PEO_ALT_MAX_N=50,000` window and the terminal class budgets.

This separation matters for attribution. If iter78 completes, the fork was the
remaining cap trigger on this daily corpus and the bounded dense twin can be
priced on a completed tree. If iter78 still fails at the same position, the
fork is exonerated and the next controlled removal is the dense twin, the only
other newly composed value device absent from iter72.

## Implementation

Production `shared_basin_fork_band(...)` now compiles to constant `false`. The
fork implementation remains in ordinary `src/ordering/` code for research, and
the test build admits it only when `SSI_SHARED_BASIN_FORK=1` is explicitly set.
`SSI_FORK_MARGIN_PCT` continues to select its test-arm margin, defaulting to the
last measured 20% point.

Making the test arm opt-in aligns the ordinary probe with the submitted worker;
an unset environment now means fork-off in both builds. This avoids the earlier
frame trap where a plain test probe silently exercised work absent from
production. The production worker does not read the environment and remains a
pure deterministic function of the pattern.

No search budget, ordering primitive, score comparison, or memory allocation
policy changes. The candidate only removes the alternate suffix, its fresh
score workspace, and the final two-way merge. The base suffix is the exact path
that iter77 already returned whenever the fork was not admitted.

## Final verification

The production candidate built successfully through the repository candidate
build script. The final release suite on the exact fork-free source reported:

```
test result: ok. 127 passed; 0 failed; 56 ignored
```

The suite includes ordering determinism, bijection, sparse-pristine state and
trajectory equivalence, exact window differential checks, and real-corpus
tests. A final focused probe with no fork environment variable returned the
expected fork-off counts: `waterund14` 72,184, `gancns` 58,299, and the c8
Chimera row 80,090. The trusted parent release build and `git diff --check` also
completed successfully.

## Evidence

- `.session-backup/iter76-dense-s3-full.log`
- `.session-backup/iter77-fork-off-full.log`
- iter77 hidden receipt: PR #743 / workflow `34980518901`
- [0285 margin-20 receipt](0285-cap-priced-fork-margin.md)
- [0276 original production-worker fork price](0276-shared-prefix-fork-and-bounded-twin.md)

## Links

- [0285 cap-priced fork margin](0285-cap-priced-fork-margin.md)
- [0284 dense sweep and 14% margin](0284-cap-trim-dense-sweep-and-fork-margin.md)
- [0282 sparse-pristine composition](0282-sparse-pristine-fork-twin-composition.md)
