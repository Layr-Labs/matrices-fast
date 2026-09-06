# 0082: the terminal-PEO ledger, priced by a measured cost law

- **Model:** Claude Opus 5 (1M context)
- **Harness:** Claude Code
- **Date:** 2026-09-06
- **Base:** `a2614af` (submission `1a4183b`, hidden 0.859834)
- **Local dev baseline on this host, worker rebuilt from that exact tree:** **0.827195**,
  fill 0.937411 (lt_1k 0.889730 / 1k_10k 0.863415 / gt_10k 0.753129) — reproduces the number
  [0081](0081-oversize-in-gate-peo-ledger.md) reports, so the two hosts agree on dev.

## The open question this answers

[0080](0080-peo-re-extraction-above-the-gate.md) and [0081](0081-oversize-in-gate-peo-ledger.md)
both charge a terminal-PEO round against a per-matrix allowance under

```
cost(round) = 5 * (n + nnz) + Lnnz          ledger = 2_500_000 units
```

and 0081 closes by naming the cost law itself as the next measurement: *"the 5:1 weighting between
`(n + nnz)` and `Lnnz` is inherited from the above-gate experiment's regression, and we have not
re-fitted it."* An allowance is only as good as its currency. A currency that under-weights a term
lets an input that is large in that term buy an unbounded amount of real time — which is the exact
shape of the three known hidden cap failures.

So we measured it, on this pipeline, over both branches.

## Method: one census, then an exact offline sweep

`leader_order`'s two PEO branches were instrumented (a measurement build only; nothing below ships)
to print, per round, `(branch, round, n, nnz, Lnnz, incumbent flops, final flops, wall seconds)`
plus the wall time of the symbolic prefix alone, and both ledgers were set effectively unbounded.
One `probe_timing_and_score` run over the 300 dev rows then yields **382 rounds** — 331 in-gate, 51
above-gate — joined to the harness's own `COUNTS` records.

The unbounded run is itself a result: dev **0.825734**, worst `order()` call **2.907 s** — *over the
2 s cap*. A ledger is not a refinement here, it is what makes the chain legal.

Two properties make that single run sufficient to price every design exactly, with no rebuild:

- The round sequence is deterministic given the incumbent permutation, and the ledger only decides
  where to stop. A smaller allowance therefore truncates the unbounded sequence to a **prefix**, so
  the exact final flop count of any `(currency, allowance)` pair is read straight off the census.
- Each round's wall time comes with it, so each design's added time is priced on the same rounds.

The resulting model reproduces the base tree's 0.827195 and the shipped candidate's 0.826107 to six
decimals against the trusted 300-matrix harness. It is bookkeeping over measurements, not a
simulation.

## The fit

Least squares of per-round milliseconds on the three terms:

| rounds | ms per M(n) | ms per M(nnz) | ms per M(Lnnz) | ratio | residual sd | residual max |
|---|---:|---:|---:|---|---:|---:|
| all 382 | 728.6 | 34.4 | 18.1 | **40 : 2 : 1** | 8.89 ms | 51.9 ms |
| in-gate 331 | 225.9 | 4.7 | 35.8 | 6 : 0.1 : 1 | 0.48 ms | 3.50 ms |
| above-gate 51 | 737.4 | 35.8 | 17.6 | 42 : 2 : 1 | 21.7 ms | 53.6 ms |

The per-vertex term dominates far more than the shipped 5 : 5 : 1 assumed, and `nnz` is nearly free
by comparison. Permuting the pattern and both bucket-MCS passes walk per-vertex lists in *permuted*
order, so their constant is set by cache misses rather than by arithmetic — which is also why the
in-gate rounds, all at `n <= 30_000` with `Lnnz` collinear, fit an apparently different shape at a
0.48 ms residual. The pooled fit is the one to budget with; the in-gate fit is what a within-branch
model would have got wrong.

A budget must bound the **tail**, not the spread. Fitting one scale factor per currency over the
same 382 rounds and reporting the worst over-run:

| currency | ms per M units | worst measured/predicted | best | sd(log) |
|---|---:|---:|---:|---:|
| **`40 n + 2 nnz + Lnnz`** (shipped here) | **18.04** | **1.74x** | 0.32x | 0.538 |
| `5 n + 5 nnz + Lnnz` (0080/0081) | 26.22 | 2.75x | 0.23x | 0.515 |
| `16 n + nnz + Lnnz` | 33.26 | 2.13x | 0.39x | 0.459 |
| `n + nnz + Lnnz` (unweighted control) | 54.93 | 4.43x | 0.39x | 0.429 |

The unweighted control has the *tightest spread and the worst tail*, which is precisely why residual
sd is the wrong criterion for a work allowance. At 18.04 ms/M the shipped 4_500_000-unit ledger is
about **81 ms** of added terminal work per matrix and **141 ms** at the worst over-run observed —
two orders below the cap.

## The change

Three edits in `mod.rs`, none in `peo_extract.rs`. Both branches, both ledgers and the cumulative
per-round charging are kept exactly as 0080 and 0081 built them.

```rust
fn peo_entry_cost(n: usize, nnz: usize) -> u64 { 40*n + 2*nnz }          // saturating
fn peo_round_cost(n: usize, nnz: usize, lnnz: u64) -> u64 { entry + lnnz }
const PEO_LEDGER: u64 = 4_500_000;
```

1. `peo_round_cost` replaces `5 * (n + nnz) + Lnnz` in **both** branches.
2. One `PEO_LEDGER = 4_500_000` replaces `PEO_OVERSIZE_LEDGER` and `PEO_LARGE_LEDGER`.
   `PEO_LARGE_MAX_LNNZ` is tightened 20_000_000 -> 4_500_000: it is implied by the ledger (`Lnnz`
   enters at weight 1) and kept only as an allocation guard. `PEO_OVERSIZE_MAX_LNNZ` stays at
   1_000_000, and the in-gate free-region carve-out (`Lnnz <= peo_extract::MAX_LNNZ`) is untouched,
   so the promoted in-gate chain keeps its eight free rounds and none of its behaviour changes.
3. The above-gate entry becomes
   `n >= 16 && nnz <= PEO_LARGE_MAX_NNZ && peo_entry_cost(n, nnz) <= PEO_LEDGER`. `peo_entry_cost`
   is the part of the round cost knowable *before* the column counts exist, so a matrix that cannot
   afford one round is refused before paying `permute_pattern` + `EliminationTree::from_pattern` +
   `column_counts_gnp` for a chain that is then declined anyway.

Every gate stays structural: dimension, nonzeros, factor nonzeros. No matrix identity, no clock, no
environment and no filesystem input reaches the production path, and both cost functions are pure
functions of `(n, nnz, Lnnz)`, so the two-runs-must-agree gate is unaffected.

## Result

Complete 300-matrix trusted run, baseline then candidate:

| | `a2614af` | candidate |
|---|---:|---:|
| weighted flop geomean | 0.827195 | **0.826107** |
| fill tiebreak | 0.937411 | 0.937020 |
| lt_1k | 0.889730 | 0.889730 |
| 1k_10k | 0.863415 | 0.863403 |
| gt_10k | 0.753129 | **0.750416** |

**Seven strict movers, zero regressions, 293 ties.** Exact flop counts:

| row | n | nnz | flops `a2614af` -> candidate | rounds |
|---|---:|---:|---|---:|
| `gams05` | 17_364 | 252_910 | 3_481_397_874 -> 3_090_739_698 (**−11.22 %**) | 0 -> 1 |
| `pooling_sppc3pq` | 23_173 | 893_724 | 728_027_209 -> 711_741_100 (−2.24 %) | 0 -> 1 |
| `pooling_sppb5pq` | 18_529 | 674_470 | 218_002_440 -> 213_571_545 (−2.03 %) | 0 -> 1 |
| `maxcsp-ehi-85-297-71` | 2_372 | 207_996 | 1_112_735_756 -> 1_111_043_268 (−0.15 %) | 1 -> 2 |
| `nuclear10a` | 17_493 | 163_816 | 58_262_504 -> 58_257_464 (−0.01 %) | 1 -> 3 |
| `crudeoil_lee4_09` | 15_904 | 101_792 | 162_105_989 -> 162_095_297 (−0.01 %) | 2 -> 3 |
| `crudeoil_lee4_10` | 17_809 | 120_632 | 199_013_967 -> 199_007_973 (−0.00 %) | 1 -> 2 |

The first three are rows `5 * (n + nnz) + Lnnz` cannot reach: `gams05`'s factor is 3_198_854, so one
round costs 4.55M against a 2.5M allowance, and for the two `pooling` rows `5 * (n + nnz)` alone is
4.58M and 3.46M. Under the fitted currency the same rounds cost 4.40M, 4.48M and 2.95M. The last
four are extra depth the larger allowance buys on rows 0080 and 0081 already served.

**`gams05` alone is 7.96 of the 10.88 bips, and the three `gams`/`pooling` rows are 99 % of it.** The
mechanism cannot regress a row — a PEO of the incumbent completion `H` eliminates the original graph
into a completion contained in `H`, and acceptance still requires a bijection and a strictly lower
exact score from the independent scorer — so this is not over-fitting. But the *magnitude* is three
rows of two prefix families and should not be extrapolated to a rotated corpus.

## Where the allowance came from

Priced offline from the census, exactly, against the base's own baseline and tail. `dt` is the
change in terminal-chain wall time summed over the corpus, and per worst single row.

| design | dev | bips | worst row | corpus total |
|---|---:|---:|---:|---:|
| base `a2614af` | 0.827195 | — | — | — |
| `40n+2nnz+L` @ 4.0M | 0.827053 | 1.42 | +45 ms | −7 ms |
| `40n+2nnz+L` @ 4.4M | 0.826258 | 9.38 | +117 ms | +153 ms |
| **`40n+2nnz+L` @ 4.5M** | **0.826107** | **10.88** | **+117 ms** | **+238 ms** |
| `40n+2nnz+L` @ 5.0M | 0.826107 | 10.88 | +117 ms | +238 ms |
| `40n+2nnz+L` @ 6.0M | 0.826104 | 10.92 | +117 ms | +455 ms |
| `16n+nnz+L` @ 4.0M | 0.826100 | 10.95 | +117 ms | +619 ms |
| unbounded | 0.825734 | 14.61 | worst call **2.907 s** | — |

`[4.5M, 5.0M]` is a flat optimum and 4.5M is its lower edge. The comparison worth keeping is the
last two data rows: `16n+nnz+L` at 4.0M buys the same score with the same worst row for **380 ms
more corpus work**, because it admits `unitcommit_200_100_1_mod_8` (n = 146_830, +89 ms, 0.04 bips)
and pays symbolic counts on `cont6-qq` — both of which the fitted currency refuses at entry. Two
rows that the fitted currency does admit gain nothing and cost time (`pooling_sppc1pq` +33 ms,
`kissing2` +19 ms); no currency separates them from `gams05`, because both are cheaper than it in
every term.

## The entry refusal is free time, and it is why the ledger can afford `gams05`

Applied on its own, with the base's own currency and allowance, the entry test changes **no score at
all** and returns **216 ms** of corpus wall time: the base computes a full symbolic factorization on
every above-gate row it then declines, and those are the largest rows in the corpus —
`acopf_case9241pegase_qcqp` (n = 313_068) −61 ms, `faclay75` (n = 272_878) −58 ms, `gabriel10`
(n = 244_056) −42 ms, `unitcommit_200_100_1_mod_8` −21 ms, `cont6-qq` −15 ms — which are otherwise
among its cheapest. It buys no bips and cannot carry a submission alone; it is what pays for the
rounds that do.

## Why weighting `n` at 40 is a safety property

0081 records that three hidden failures share one shape: *a limit was widened so larger inputs
entered an expensive path, with nothing bounding what the largest such input could cost.* Under
`5 * (n + nnz) + Lnnz` at 2.5M, `n` costs 5 units, so an `n`-heavy nonzero-light pattern is admitted
up to `n + nnz <= 500_000` and then pays whatever a 500k-vertex reconstruction costs — on this
corpus about 300 ms, on a rotated corpus unknown. Under `40 n + 2 nnz + Lnnz` at 4.5M the entry cost
alone reaches the allowance at **n = 112_500**, so a very large pattern is refused by structure
however sparse its factor turns out to be, and every term is separately bounded
(`n <= 112_500`, `nnz <= 2_250_000`, `Lnnz <= 4_500_000`).

So this is a *larger* nominal allowance that is *tighter* on exactly the inputs implicated in the
known failures, and looser only on the mid-size `Lnnz`-heavy rows where the score is. That
asymmetry is the whole result.

The counter-risk is real and unmeasured: an `n`-heavy row above 112_500 vertices that the base would
have served, and that the entry refusal now turns away, loses whatever it would have gained. No dev
row is in that set — zero regressions — but that is the mechanism by which this change could give
score back on a corpus we cannot see.

## Correctness and timing

69 tests pass (68 inherited plus one new). The new test,
`peo_ledger_entry_cost_bounds_and_admits_the_in_gate_region`, asserts that the entry cost is a true
lower bound on the round cost at every extreme including `usize::MAX` / `u64::MAX` (otherwise the
pre-symbolic refusal could turn away a round the ledger would have afforded), that a charged in-gate
round at the envelope corner `(30_000, 180_000, 1_000_000)` is still affordable so the oversize path
is not dead code, and that admission is bounded in `n` alone. The trusted 300-case run completes
with bijection, determinism and watchdog checks green.

## Timing

Five interleaved pinned trials per arm (A, B, A, B, ...) on an otherwise idle 16-core box, both arms
`taskset -c 0-1` — the grader's shape — inside bubblewrap with no network and a tmpfs `/tmp`, one
arm at a time. Per-row figures are medians of five. The base arm's own per-row range across its five
trials is median 9.3 ms, p95 29.8 ms, max 79.8 ms, which is the resolution of everything below.

| | `a2614af` | candidate |
|---|---:|---:|
| per-trial corpus maxima (s) | 1.239 / 1.255 / 1.249 / 1.226 / 1.250 | 1.267 / 1.270 / 1.274 / 1.260 / 1.267 |
| **median corpus maximum** | **1.249 s** | **1.267 s** (+17.5 ms) |
| corpus total, summed per-row medians | — | **−241.7 ms** |
| rows slower by more than +25 ms | — | **5**, all five of them score movers |
| dev score, every trial | 0.827195 | 0.826107 (bit-identical in all five) |

Slower (all score movers):

| row | base | candidate | delta |
|---|---:|---:|---:|
| `gams05` | 1.075 s | 1.198 s | **+123.6 ms** |
| `pooling_sppc3pq` | 0.854 s | 0.949 s | +94.3 ms |
| `nuclear10a` | 0.927 s | 0.970 s | +43.2 ms |
| `pooling_sppb5pq` | 0.299 s | 0.334 s | +34.5 ms |
| `maxcsp-ehi-85-297-71` | 0.219 s | 0.253 s | +34.0 ms |

Faster, from the entry refusal, none of them changing score:

| row | n | base | candidate | delta |
|---|---:|---:|---:|---:|
| `acopf_case9241pegase_qcqp` | 313_068 | 0.337 s | 0.250 s | **−87.0 ms** |
| `gabriel10` | 244_056 | 0.733 s | 0.678 s | −54.8 ms |
| `faclay75` | 272_878 | 0.685 s | 0.634 s | −51.2 ms |
| `unitcommit_200_100_1_mod_8` | 146_830 | 1.066 s | 1.018 s | −47.7 ms |
| `cont6-qq` | 120_395 | 0.273 s | 0.252 s | −20.8 ms |

Three things worth reading off this. **The corpus maximum barely moves**: `crudeoil_lee4_10` sets it
in both arms and takes +17.5 ms, and `gams05` — the +123.6 ms row — lands at 1.198 s, still *below*
the base's own 1.249 s maximum, so the change does not push out the row nearest the cap. **The
corpus is net faster by 241.7 ms**: the entry refusal returns more on the five largest rows than the
seven movers spend. And on the 293 rows whose score did not change, the delta is median −1.5 ms with
a p95 of +6.3 ms, so there is no systematic drift hiding in the tail.

The census predicted the movers well (`gams05` +117 ms predicted against +123.6 measured) and
**under**-predicted the entry-refusal savings, because it charged a refused row only for the
symbolic prefix it measured inside the loop and not for the surrounding allocation and pattern
permutation that the refusal also skips.

## Also measured, and not pursued

- Raising the in-gate oversize factor cap `PEO_OVERSIZE_MAX_LNNZ` above 1_000_000: no dev row
  reaches it (the census's largest in-gate factor is 714_536), so it changes nothing locally while
  widening a limit on the hidden corpus. Left alone deliberately.
- The three rows 0080 prices out stay priced out, now by structure rather than by allowance:
  `acopf_case9241pegase_qcqp`, `faclay75` and `gabriel10` need 15.1M, 13.7M and 12.1M entry units
  against a 4.5M ledger. Their unbounded chains do gain (0.4 % to 2.3 %) at 0.29-0.34 s **per
  round**, and the unbounded corpus maximum of 2.907 s is what admitting them looks like.

## Next, and a correction to `0080`'s stated next step

`0080` concludes that recovering the three expensive rows "needs a cheaper reconstruction, **or one
reconstruction shared between the watcher and the chain**, rather than a larger allowance." The
census prices the second half of that and it does not hold. Both `completion::refine_limited` call
sites sit inside `n >= 16 && n <= 30_000 && nnz <= 180_000`, so the duplicated reconstruction is
confined to in-gate rows — and in-gate rounds are cheap precisely because that gate keeps them
small:

| | rounds | total time | median | p95 | max |
|---|---:|---:|---:|---:|---:|
| in-gate (branch 1) | 331 | **1.415 s** | 1.3 ms | 15.3 ms | 33.7 ms |
| above-gate (branch 2) | 51 | 6.947 s | 52.2 ms | 340.5 ms | 343.0 ms |

The 235 in-gate rows whose first round gains nothing spend **0.314 s in total across the whole
corpus**, median 0.6 ms each. Sharing saves a fraction of that. Meanwhile `gabriel10`,
`acopf_case9241pegase_qcqp` and `faclay75` are *above* the gate, where the watcher never runs, so
there is nothing to share on them at all, and one round there costs 0.336 s, 0.315 s and 0.294 s.
83 % of the corpus's terminal-PEO time is 51 above-gate rounds, not 331 in-gate ones.

So the remaining route to those rows is a **cheaper** reconstruction, not a shared one: both paths
materialise a `Vec<Vec<u32>>` adjacency of the entire completion while MCS needs only degree-bucket
access, and a streaming or lazily-consumed completion attacks the 0.3 s per-round cost directly.
That is an algorithmic change. Sharing remains correct hygiene — conditional on the watcher having
failed, since a winning watcher legitimately reconstructs a different completion — but it is not a
lever.

Second, the fit here is one host and one corpus. `Lnnz` at 18 ms/M against `n` at 729 ms/M is a
cache-miss ratio, so it should be re-fitted on any host whose memory hierarchy differs before the
allowance is trusted as a millisecond bound rather than as a relative one. The **ratio** is what
should travel between boxes.
