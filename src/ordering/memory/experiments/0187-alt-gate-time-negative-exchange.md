# 0187 — The pipeline's cost/yield table, and the one stage that pays for the ladder

Direction: stop adding to the cap and start **subtracting** from it. Every
previous iteration priced an *addition* (the terminal ladder) against a 2 s
per-matrix wall cap; this one prices the stages the pipeline already runs, finds
the one that spends real time on the rows nearest the cap for no measured
return, and re-points its gate so the ladder's seconds come out of it.

## Instrument: per-phase seconds *and* per-phase yield on one run

`SSI_PROBE_PHASES=1` already prints, per row, `phase=seconds/cumulative_ratio`
where the ratio is running-best flops over the row's baseline. That is a
cost/yield table for the whole pipeline, and nothing in this session had ever
reduced it to corpus totals. `evidence/0187-tools/phase_table.py` and
`phase_rows.py` parse it (`phase_table.py <log>`); on the frontier log
`0161-tail-attribution-300.log` (299 rows, 115.5 s of phase time):

| phase | seconds | % of phase time | yield (Σ ratio drop) | rows | yield per second |
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
| 7.telos / 15.peel | 0.26 | 0.2 % | 0.0000 | 0 | 0.0000 |

`13.alt` (the alternate-seed PEO chains) is the outlier: 5.8 % of the pipeline's
wall time for 0.06 % of its ratio, and its three beneficiaries are all small
(mpbp_15 n=9858, syn40hfsg n=1022, maxcsp-ehi-85-297-71 n=2372). Split by size:

| band | rows | 13.alt seconds | yield |
|---|---|---|---|
| n < 1 000 | 146 | 0.58 | 0.0000 |
| 1 000 – 7 000 | 99 | 1.15 | 0.0029 |
| 7 000 – 10 000 | 9 | 0.54 | 0.0030 |
| 10 000 – 12 000 | 8 | 1.16 | 0.0000 |
| 12 000 – 50 000 | 30 | 3.24 | 0.0000 |

and the rows it costs the most on are exactly the rows the 2 s cap is closest
to: transswitch0300p n=11659 0.201 s, powerflow0300p 11251 0.188,
mpbp_34 11556 0.172, mpbp_35 11120 0.167, crudeoil_lee4_06 10429 0.111 — every
one of them with a measured yield of exactly zero.

## Change

`src/ordering/mod.rs`: `PEO_ALT_MAX_N` 50 000 → **10 000** (one constant, with
its `if` untouched, plus the measured rationale in the constant's doc comment).
No other stage is touched, no threshold is per-matrix, and the gate stays a
monotone predicate on `n`.

## Measured result (probe, full 300-row dev corpus, production path)

`evidence/0187-probe-altgate.log` vs the shipped density-shaped build
`evidence/0185-probe-density-shaped.log`:

* `COUNTS` lines (per-row flops): **300/300 byte-identical** — the change cannot
  move the score, and the dev score is unchanged at **0.792212** (bucket
  geomeans 0.8874 / 0.8391 / 0.6856).
* Corpus `order()` time **119.6 s → 115.8 s (−3.8 s, −3.2 %)**.
* The savings land on the tight rows (seconds removed, largest first):
  arki0013 0.162, chp_shorttermplan2d 0.161, mpbp_35 0.157, gasprod_sarawak81
  0.156, crudeoil_pooling_dt2 0.154, procurement1large 0.153, powerflow0300p
  0.153, crudeoil_pooling_dt3 0.150, transswitch0300p 0.147, popdynm200 0.141,
  mpbp_34 0.139 — 34 rows in total (every row with 10 000 < n ≤ 50 000).
* The terminal ladder is unchanged and still costs what 0185 measured in-run
  (draw price mean 0.0414 s / p90 0.0720 / max 0.0906 over the 238 in-window
  rows vs 0.0414 / 0.0717 / 0.0902 for the shipped build): the gate pays for it
  out of base time, it does not touch the draw.
* Ladder movers vs the frozen frontier (`0151-probe-baseline-ab30c0e.log`
  `COUNTS`): 13 rows, all improvements, none regressed; dominated by
  crudeoil_lee2_06 −5.58 % (n=6418), chimera_rfr-02 −0.99 %, multiplants_stg5
  −0.76 %, pooling_adhya4pq −0.50 %, rsyn0840m02m −0.34 %, sfacloc2_4_80
  −0.33 %, crudeoil_lee1_07 −0.29 %.

## Rejected by measurement in the same run: the window extension

The bucket arithmetic says a `gt_10k` row is worth 3.4× an `lt_1k` row per
percent (0.40×0.6856/45 against 0.30×0.8874/147), and the shipped ladder's
window `n ≤ 12 000` contains only 8 of those 45 rows — so the obvious way to
spend the freed seconds was to widen the window. It was measured and it is
empty. The first attempt (`SSI_TERM_FULL_N=20000` alone) was inert because the
window is a production `if` outside the existing test knob, so test-only
`SSI_TERM_WIN_N` / `SSI_TERM_WIN_NNZ` knobs were added (production still reads
12 000 / 200 000); with both set to 20 000:

* all 13 dev rows with 12 000 < n ≤ 20 000 take the draw (their time rises by
  0.005–0.095 s each, +0.47 s on the band), and
* **not one flop changes on any of the 300 rows** (`COUNTS` diff empty);
  `SCORE` stays 0.792212.

So the ladder's yield is a property of the `n ≤ 12 000` band, not of the bucket
weights: above 12 000 a 2e8 draw is pure cost. The window stays at 12 000.

## Frame correction recorded here

`parallel::phase_mark` and its `PHASE_MARKS` ledger are `#[cfg(test)]`
(`src/ordering/parallel.rs:355-371`), and the harness builds its worker with
`cargo build --release -p ssi-candidate-worker` (`scripts/local-candidate-build.sh`),
so the 25 per-phase `score(&best_perm)` evaluations a probe pays per row are
**not** in the graded path. Probe timings are a test-build frame; the graded
frame is the sandboxed worker. Every "adds X s / worst order() Y s" figure in
this ledger is therefore an upper bound on the graded cost, and the instrument
itself was never the headroom it looked like.

Tools: `evidence/0187-tools/phase_table.py` (corpus cost/yield table),
`evidence/0187-tools/phase_rows.py` (per-row breakdown, n-range aggregates).
