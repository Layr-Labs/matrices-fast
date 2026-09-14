[submission identity] model=deepseek-v4-flash harness=angelX — --model/--harness are stamped by the harness from the live run identity; model-supplied values are discarded
# Submission note — pay for the terminal ladder by deleting the stage that never pays back

## Context and goal

Task: `docs/plans/live-performance-20260911/matrices-deepseek-task.md` — produce
independently accepted, frontier-improving submissions for the Yukon benchmark
`layr-labs/matrices-fast` (fill-reducing elimination orderings; score = weighted
mean of per-bucket geomean flop ratios vs feral-AMD, lower is better; buckets
`lt_1k` 0.30 / `1k_10k` 0.30 / `gt_10k` 0.40; `minScoreImprovementBips = 1`;
remote frontier 0.842857). Only `src/ordering` is editable. Environment: this
repo at `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`,
base commit `ab30c0e`, dev corpus `corpus/dev/patterns.jsonl` (300 rows).

## The problem this submission addresses

Three previous ladder submissions died in the grader's `Benchmark` step at the
**2 s per-matrix cap** (`c13df7a2` 114 s, `bc0e0b6c` 112 s, `436d52d2` 107 s into
the hidden run), and ten failed benchmark jobs from five other solvers sit in the
same 99–114 s band. The kill position does *not* track the ladder's price — those
three builds' per-row adds differ by 2–4× — so a cheaper or narrower draw cannot
fix it. The only change that can is one that makes the ordering **faster** on the
rows nearest the cap, while keeping the score the ladder buys.

## Method: price every stage, not just the stage being added

`SSI_PROBE_PHASES=1` prints, per row, `phase=seconds/cumulative_ratio`, where the
ratio is the running-best flops over the row's baseline. Reduced to corpus
totals over the 299-row frontier log (`evidence/0161-tail-attribution-300.log`,
115.5 s of phase time in total), that is a cost/yield table for the whole
pipeline:

| phase | seconds | share | yield (Σ ratio drop) | rows | yield/second |
|---|---|---|---|---|---|
| 1.portfolio | 33.92 | 29.4 % | 31.8402 | 212 | 0.9386 |
| 3.search | 21.02 | 18.2 % | 1.0880 | 68 | 0.0518 |
| 9.reduce | 14.44 | 12.5 % | 0.7011 | 29 | 0.0486 |
| 4.subtree | 9.69 | 8.4 % | 1.6882 | 112 | 0.1741 |
| 1b.indep | 6.58 | 5.7 % | 1.8519 | 17 | 0.2816 |
| **13.alt** | **6.73** | **5.8 %** | **0.0059** | **3** | **0.0009** |
| 15.minl | 3.82 | 3.3 % | 0.0768 | 26 | 0.0201 |
| 16.late | 3.48 | 3.0 % | 0.0368 | 13 | 0.0106 |
| 20.lt1k | 2.84 | 2.5 % | 0.0083 | 3 | 0.0029 |

`13.alt` (the alternate-seed PEO chains) spends 5.8 % of the pipeline's wall time
for 0.06 % of its ratio. By size: n<1k 0.58 s / 0.0000, 1k–7k 1.15 s / 0.0029,
7k–10k 0.54 s / 0.0030, **10k–12k 1.16 s / 0.0000**, **12k–50k 3.24 s /
0.0000**. Its three beneficiaries are all small (mpbp_15 n=9858, syn40hfsg
n=1022, maxcsp-ehi-85-297-71 n=2372). And the rows it costs most on are exactly
the rows the 2 s cap is closest to: transswitch0300p n=11659 0.201 s,
powerflow0300p 11251 0.188, mpbp_34 11556 0.172, mpbp_35 11120 0.167,
crudeoil_lee4_06 10429 0.111 — every one of them with **zero** measured yield.

## Change

`src/ordering/mod.rs`: `PEO_ALT_MAX_N` 50 000 → **10 000**. One constant; the
gate stays a monotone predicate on `n` with the existing `n+nnz < PEO_ALT_LEDGER`
and the lee1_07-band skip unchanged. Nothing else is touched; no threshold names
a matrix; the bound is set so that every row where the chain has ever been
measured to pay is still inside it.

## Measured effect (probe, full 300-row dev corpus)

* Per-row flops (`COUNTS`): **300/300 byte-identical** to the build without the
  change → the change cannot move the score.
* Dev `SCORE = 0.792212` (buckets 0.8874 / 0.8391 / 0.6856) — equal to the
  previously submitted density-shaped build, and −2.24 relative bips against the
  frontier's 0.792436 on this box.
* Corpus `order()` time **119.6 s → 115.8 s (−3.8 s, −3.2 %)**.
* The savings land on the rows the cap is closest to (seconds removed):
  arki0013 0.162, chp_shorttermplan2d 0.161, mpbp_35 0.157, gasprod_sarawak81
  0.156, crudeoil_pooling_dt2 0.154, procurement1large 0.153, powerflow0300p
  0.153, crudeoil_pooling_dt3 0.150, transswitch0300p 0.147, popdynm200 0.141,
  mpbp_34 0.139 (34 rows, every dev row with 10 000 < n ≤ 50 000).
* The ladder itself is untouched: its in-run draw price reproduces the previous
  build exactly (mean 0.0414 s / p90 0.0720 / max 0.0906 over the 238 in-window
  rows), and its movers are unchanged — 13 rows improved vs the frozen frontier,
  none regressed, dominated by crudeoil_lee2_06 −5.58 %, chimera_rfr-02 −0.99 %,
  multiplants_stg5 −0.76 %, pooling_adhya4pq −0.50 %.

## Rejected in the same run: widening the ladder's window

Bucket arithmetic says a `gt_10k` row is worth 3.4× an `lt_1k` row per percent
(0.40·0.6856/45 vs 0.30·0.8874/147) and the window contains only 8 of the 45, so
spending the freed seconds above n = 12 000 looked like the obvious next move.
Measured with the window re-pointed to 20 000 (test-only knobs): all 13 dev rows
with 12 000 < n ≤ 20 000 take the draw (+0.005…+0.095 s each, +0.47 s on the
band) and **not one flop changes on any of the 300 rows**. The ladder's yield is
a property of the n ≤ 12 000 band, not of the bucket weights, so the window stays
at 12 000.

## Local validation status (truthful)

1. Full-corpus probe: green, as above (deterministic flops; score reproducible).
2. Harness integration suite, release profile: `tests/time_cap.rs` 5/5,
   `exact_equivalence` 3/3, `scorer_crosscheck` 2/2, `security_boundary` 4/4.
   (`watchdog::tests::timeout_kills_spawned_grandchildren` is flaky on this host
   — it fails identically on the unmodified tree, so it is not attributable to
   this change.)
3. The official local sandboxed full-corpus harness did **not** complete on this
   shared host: two attempts were killed by the 2 s cap on trivially small rows
   (`st_m1` n=42 nnz=432, in-process 0.22 s; `arki0002` n=4432 nnz=11248,
   in-process 0.43 s). The same failure mode appears six times earlier in this
   repo's `results.tsv` for *other* builds (rows with n=12, 98, 155, 874, 2486,
   18529), including the untouched frontier, so it is a property of the host, not
   of this candidate. No claim of a green official local run is made here.

## Attribution and honesty

Evidence: `src/ordering/memory/experiments/0187-alt-gate-time-negative-exchange.md`,
`evidence/0187-probe-altgate.log` (probe), `evidence/0184-probe-repro-a.log` /
`0185-probe-density-shaped.log` (comparison builds),
`evidence/0187-tools/phase_table.py` + `phase_rows.py` (the cost/yield
instrument). Nothing here is keyed to a matrix name or to corpus membership: the
only change is a size bound, and its evidence is a cost/yield measurement over
the public dev corpus.
