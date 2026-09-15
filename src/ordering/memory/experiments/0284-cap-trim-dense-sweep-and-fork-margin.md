# 0284 — cap trim: one dense-twin sweep and the lossless fork margin

- **Date:** 2026-09-15 (iter76)
- **Base:** `0696f9a` / submission `b69938a1` / PR #739.
- **Change:** bounded dense/hub twin `10/4/4 -> 10/3/4`; shared-basin fork
  AMD-anchor margin `10% -> 14%`.
- **Public measurement:** the dense-sweep arm scores **0.790166** versus
  iter75's **0.790142** on all 300 matrices. The fork-margin sweep shows that
  14% retains every known fork mover; 15% loses `gancns`.
- **Status:** submitted as `502f790c` / PR #740 and cap-failed after about
  83.422 seconds of Benchmark execution.

## Receipt that motivates a dose cut

Iter75 removed a redundant first reconstruction of sparse-pristine's mutable
adjacency and retained that single large image between sequential games. It was
an output-invariant optimization: fresh base and candidate probes had identical
flop records on all 300 rows, and two production-worker trials reduced aggregate
wall on the four affected sparse rows. Nevertheless, the hidden submission
failed the two-second per-matrix cap.

Submission `b69938a1-a450-4e2d-9a3f-d7a1c17a4771` opened PR #739 and workflow
`34906867100`. The final sandbox test build finished at
`23:04:32.538756Z`; `RUN FAILED` appeared at `23:06:05.913602Z`, about
**93.375 seconds** later. The preceding margin-restored composition failed at
about **93.79 seconds**. Removing hundreds of MiB of redundant sparse image work
did not materially move the hidden failure position.

That is negative attribution evidence. It does not say the representation
optimization was ineffective on rows that use it; the production receipts show
that it was effective. It says the binding hidden row probably does not use the
large sparse-pristine branch. A quick resubmission therefore needs to reduce a
different post-frontier device rather than continue tuning sparse allocation.

## Why the bounded dense twin is the next trim

The current composition differs from the promoted PR #719 tree in only a few
search devices. Of those, the dense/hub twin is the remaining site with an
exponentially wider exact DP shape than the promoted tree: width 10 rather than
8 for `1_000 <= n <= 5_205`, with four sweeps. Its original 24-row census found
three movers and chose `10/4/4`; the other tested schedules were close. The
site is strictly accepted, but the fourth sweep still consumes time even when
it finds no additional strict decrease.

A same-binary seam probe re-priced the three known movers. Returned flop counts
are:

| schedule | `chimera_mgw-c16-2031-01` | `chimera_rfr-02` | `chimera_lga-01` |
|---|---:|---:|---:|
| 10/4/4 (iter75) | **2,560,253** | **2,487,272** | **492,146** |
| 10/3/4 | 2,560,265 | 2,501,621 | 494,718 |
| 10/2/4 | 2,567,143 | 2,490,139 | 494,718 |
| 8/4/3 | 2,570,031 | 2,489,836 | 494,718 |

Three sweeps retain a strict improvement on every mover and give back only 12,
14,349, and 2,572 flops. Two sweeps has a larger loss on the c16 row. Returning
fully to 8/4/3 is not uniformly better and discards more of the measured width-10
device. `10/3/4` is therefore the smallest direct dose reduction supported by
the curve: one of four sweeps removed, or **25% fewer sweeps at the width-10
site**, with no gate expansion and no new work.

The full 300-row no-extra-score probe completed in 510.42 seconds. It reports:

```
lt_1k    count=147   geomean=0.8870
1k_10k   count=108   geomean=0.8373
gt_10k   count=45    geomean=0.6822
SCORE = 0.790166
WORST order() = 5.556 s
```

Against iter75's full record, the three dense movers change exactly as above.
One small fork row, `chimera_mgw-c8-439-onc8-001`, also returned 78,177 rather
than 78,229 in that full process. The dense twin cannot run at n=440, so this is
not attributed to the sweep change; the shared suffix fork uses concurrent
execution and has shown scheduling-sensitive selection in focused versus full
probe processes. The change is an improvement, not a regression. Focused runs
are used for the fork gate boundary below.

## The fork margin has four free percentage points

The first cap correction raised the shared-basin fork's AMD-anchor margin from
0% to 10%. That removed known expensive near-anchor non-movers, but left all
rows whose pre-suffix incumbent was 85--90% of AMD exposed even though no public
fork mover requires that band. Rather than infer the boundary from the final
returned ratio, the margin was swept through the test seam on the actual
pre-suffix checkpoint.

For margins 10%, 11%, 12%, 13%, and **14%**, focused probes reproduce:

```
gancns                         58,202
chimera_mgw-c8-439-onc8-001   78,229
```

At 15%, `gancns` returns 58,299 instead. The final output ratio had made 15%
look admissible, but admission happens earlier in the suffix pipeline; the seam
measurement is authoritative. Production is therefore pinned at **14%**, the
largest whole-percent measured value that retains every known fork mover. This
removes the entire unpriced 86--90% band without changing any public mover in
the focused frame. The other mover, `waterund14`, also remains at 70,488.

This gate is not presented as a proof of hidden completion. Hidden identity is
redacted, and a strongly below-anchor small row may still pay the fork. It is a
lossless public tightening chosen at the real internal checkpoint, and it adds
completion margin independently of the dense sweep cut.

## Production build and wall receipt

The production candidate was built with the repository's candidate build
script. A production-worker A/B ran three repetitions per arm on the three
dense movers. Ratios confirm the expected production outputs, but frame wall
was noisy:

| row | iter75 median | iter76 median | production ratios |
|---|---:|---:|---|
| c16 | 3.0902 s | 3.7766 s | 0.769289916 / 0.769293522 |
| rfr | 3.8823 s | 4.1260 s | 0.644297374 / 0.648014307 |
| lga | 2.5030 s | 2.1993 s | 0.736724554 / 0.740574744 |

Aggregate medians move the wrong way by 6.6%, while aggregate minima improve
by 4.8%. This is not a usable timing claim. The honest wall claim is structural:
the candidate executes three width-10 sweeps where iter75 executes four. No
claim is made that this interleaved worker frame resolves the end-to-end delta.

## Safety and expected trade

Both changes only remove work. The dense twin keeps its existing dimension and
density gates, exact score check, bijection check, ledger, width, and offset
step. A candidate found in any of the first three sweeps is unchanged; only a
strict improvement first discovered in sweep four can be lost, which is exactly
what the full score measurement prices. The fork change narrows admission and
leaves the base suffix untouched. It cannot worsen the base suffix's ordering;
it can only decline to run the alternate suffix on the newly excluded band.

The public price of the dense cut is **+2.4e-5** score. The margin cut retains
all measured fork movers through 14%. The resulting tree sacrifices a small
fraction of the iter75 public advantage in exchange for two independent sources
of cap margin outside the sparse branch that the last receipt effectively
exonerated.

The final release suite reports **127 passed, 0 failed, 56 ignored**. The
production candidate build also completed successfully, and `git diff --check`
is clean.

## Hidden receipt

The exact iter76 tree was submitted as
`502f790c-790e-4c39-a73f-6556aa91ff81` and opened PR #740. The final test build
and corpus fetch completed before the Benchmark command began at
`23:38:19.762530Z`; the grader reported

```
RUN FAILED: hidden matrix: order() exceeded the 2.0s per-matrix cap and was killed
```

at `23:39:43.184244Z`, about **83.422 seconds** later. This is the fourth
consecutive cap failure of the composition and the third after adding an AMD
margin to the fork. The 10% to 14% tightening and the dense sweep removal did
not produce a completion. The next iteration therefore prices away a measured
fork mover rather than continuing to demand that every public mover survive a
cap correction; see [0285](0285-cap-priced-fork-margin.md).

## Evidence

- `.session-backup/iter76-dense-s3-full.log`
- `.session-backup/iter76-production-worker-ab.log`
- `.session-backup/probe-iter75-final`
- `.session-backup/worker-iter75-final`
- `.session-backup/worker-iter76-trimmed`
- iter75 public receipt: PR #739 / workflow `34906867100`
- iter76 public receipt: PR #740 / workflow `34909483933`

## Links

- [0282 sparse-pristine composition](0282-sparse-pristine-fork-twin-composition.md)
- [0283 sparse first-reset and buffer retention](0283-sparse-first-reset-and-buffer-retention.md)
- [0285 cap-priced fork margin](0285-cap-priced-fork-margin.md)
