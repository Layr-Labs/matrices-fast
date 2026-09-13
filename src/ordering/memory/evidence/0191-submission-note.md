# The terminal draw gets its own upper band back; the alternate-seed chain is kept out of it

## Context and goal

This is the `matrices-fast` fill-reducing-ordering benchmark: `src/ordering` is the
only editable path, the score is a size-bucketed weighted geomean of predicted
factorization flops against `feral`'s AMD anchor (`lt_1k` 0.30, `1k_10k` 0.30,
`gt_10k` 0.40; lower is better), the remote best is **0.842857**, and a
submission is promoted only if it beats that by at least 1 bip
(`minScoreImprovementBips: 1`). Every row's `order()` must return inside a 2 s
per-matrix cap or the whole hidden run fails.

Everything below is dev-corpus work plus one remote receipt from this round. No
claim of remote acceptance is made here.

## Environment and setup

- Workspace `/home/frosty/angelX/.tmp/live-performance-20260911/matrices-deepseek`;
  only `src/ordering` is candidate content. The probe harness
  (`target/probe-sandbox.sh`, test-only, lives under the gitignored `target/`)
  compiles and runs the candidate worker's `#[cfg(test)]` probe inside the same
  bubblewrap boundary the grader uses; `cargo test --release -p ssi-candidate-worker`
  runs 300 dev matrices and prints `SCORE`, per-row `COUNTS`, per-row
  `PHASES` (25 phase marks) and a per-row `LADGATE` pre-draw mark.
- Official local check: the benchmark command
  `bash scripts/local-candidate-build.sh && cargo run --release` (the real
  sandboxed worker, appends one row to `results.tsv`).

## Baseline and prior work in this project

The frozen frontier `ab30c0e` scores 0.842857 remotely and 0.792439 on the dev
probe. Our line of attack has been the terminal strict-accept draw ("the
ladder", an `rgreedy::search` draw on the incumbent with a fixed budget and
seed): it is worth ~2.2e-4 absolute on dev, and it is the only change of ours
that has ever moved a dev row.

Six submissions carrying it — `436d52d2` (107 s into the hidden Benchmark step),
`6ad8cc5e` (105 s), `c13df7a2` (114 s), `bc0e0b6c` (112 s), `e5a3c6b4`
(104.4 s) and `55d9ed93` — were killed by the 2 s cap. Exactly two builds have
finished a hidden run: the frontier itself (chain scope `n <= 50 000`, no draw,
0.842857) and `71c2c5fe` (draw up to `n = 12 000`, chain gated at `10 000`,
0.843153). The iter27 build `55d9ed93` was an attempt to be the elementwise union
of those two surviving profiles; its receipt this round is **failed**, i.e. the
sixth kill, which is what forced the re-analysis below.

## What I measured this round before changing anything

1. New instrument `evidence/0191-tools/frontier_rowdiff.py`: per-row diff of any
   two probe logs. Calibrated first — the frontier's `0151` log prints
   `SCORE = 0.792439` but its 299 `COUNTS` lines reconstruct to `0.792222`;
   adding its one missing row (`slay06m`, ratio exactly 1.0) gives exactly
   `0.792439`, and the same aggregation reproduces the shipped build's printed
   `0.792223` from its own 300 lines. Two independent prints, exact.
2. Against the frontier's own table, the shipped build changed **12 rows, all
   improvements, zero regressions**, every mover `n <= 6418`, gt_10k
   byte-identical, and **62 % of the entire dev gain sits on one row**
   (`crudeoil_lee2_06`, n=6418, -5.58 %). The draw owns ~all of that delta; the
   chain's gate, scope and allowance changes are dev-neutral (0 to -4e-6).
3. Per-row *seconds* (probe frame): our add is concentrated on the **smallest**
   rows (+6.33 s over 146 lt_1k rows, worst relative adds +79 % / +64 % / +62 %
   on rows with n = 18 / 14 / 10), while `10 000 < n <= 50 000` is **-2.90 s** and
   `n > 50 000` **-1.28 s**. The draw's own charge over its 238 drawn rows is
   9.94 s total and *anti*-correlated with size (`corr(price, log10 flops) =
   -0.23`, top-24-fill rows 0.0226 s vs bottom-24 0.0340 s), so the charge is
   bounded at ~0.03-0.095 s per row and does not grow with the fill.
4. The 10k-12k band, per-row `order()` in seconds: frontier 6.197 s with 1.16 s
   of it in the chain; `55d9ed93` (killed) 5.338 s with 0.34 s; `e5a3c6b4`
   (killed) ~5.74 s. The killed build was the **cheapest ever submitted** in that
   band and the surviving frontier the dearest. So the receipts are *not*
   ordered by cost, and "which spender owned the band" cannot be the mechanism.

## Hypothesis and change

Hypothesis: what the six kills share is not the amount of work in that band but
that the alternate-seed chain spends on a row **above the draw's window at all**;
the two builds that finished each carried exactly one spender above `n = 10 000`
(chain only, or draw only). The band `10 000 < n <= 12 000` has a measured chain
yield of exactly `0.0000` on all eight dev rows, while the draw there is worth
`powerflow0300p` (n=11251) — the only mover this project has ever had in the
0.40-weighted gt_10k bucket, worth 0.28 bips on its own and the 0.11 bips iter27
gave up.

Change, both inside `src/ordering/mod.rs`:

1. `SHIPPED_FULL_N` (the draw's window) `10_000 -> 12_000`, restoring it to the
   value the surviving receipt ran with.
2. New `PEO_ALT_SKIP_LO = 10_000` / `PEO_ALT_SKIP_HI = 12_000`, added to the
   chain's gate condition so the chain does not spend in that band; the chain
   keeps the frontier's scope (`PEO_ALT_MAX_N = 50_000`) and the reduced work
   allowance above `n = 10 000` (`PEO_ALT_LEDGER_WIDE = 1e6`).

Per row the build is now: below `n = 10 000` the surviving profile (draw +
chain), in `10 000 < n <= 12 000` the draw alone, above `12 000` the frontier's
chain scope at a quarter of its allowance. The two budgeted searches are
row-disjoint above the draw's window.

## Exact commands and measured results

```sh
bash target/probe-sandbox.sh build
SSI_PROBE_PHASES=1 bash target/probe-sandbox.sh run > src/ordering/memory/evidence/0191-probe-rowdisjoint.log
bash scripts/local-candidate-build.sh && cargo run --release -- --note "iter28 0191 ..."
python3 src/ordering/memory/evidence/0191-tools/frontier_rowdiff.py \
    src/ordering/memory/evidence/0151-probe-baseline-ab30c0e.log \
    src/ordering/memory/evidence/0191-probe-rowdisjoint.log
```

- Probe `SCORE = 0.792212` (frontier 0.792439; iter27 build 0.792223),
  `WORST order() = 1.114 s` (probe frame; the largest dev row is 0.78 s graded
  and this frame inflates `n >= 100 000` rows by up to 0.38 s, measured in 0190).
- Official sandboxed local harness: **status OK, 300 matrices, `score.json`
  score 0.792212, fill 0.924447**, buckets lt_1k 0.887426 / 1k_10k 0.839115 /
  gt_10k 0.685623 (`results.tsv` row 1789211660). The candidate build's
  per-bucket numbers agree with the probe to the printed precision.
- Against the frontier's own table: **13 rows change, all improvements, zero
  regressions**; buckets lt_1k 0.887516 -> 0.887426, 1k_10k 0.839747 -> 0.839115,
  gt_10k 0.685651 -> 0.685623; total -2.27e-4 absolute (-2.87 bips relative).
  Movers: `crudeoil_lee2_06` -5.58 %, `chimera_rfr-02` -0.99 %,
  `powerflow0300p` -0.18 %, `multiplants_stg5` -0.76 %, `rsyn0840m02m` -0.34 %.
- Against the iter27 build, exactly one row changes (`powerflow0300p`
  293009 -> 292481, the frontier's own value back), 299/300 byte-identical.
- Cost, in-run marks, probe frame: chain time in `10 000 < n <= 12 000`
  1.16 s -> **0.00 s**, in `12 000 < n <= 50 000` 3.24 s -> 1.31 s, corpus chain
  time 6.73 s -> 3.60 s; corpus `order()` total 117.90 s (frontier) / 124.43 s
  (the killed build) / **121.57 s** (this build).

## Failures and course corrections

- The iter27 doctrine ("be the elementwise union of the two surviving profiles by
  giving 10k-12k to the chain alone") is falsified by this round's receipt: that
  build was cheaper than the surviving frontier on every row of that band and was
  killed anyway.
- The earlier budget-shaped experiments (`1e8` instead of `2e8` on the draw's
  budget) were re-read: the extra `1e8` touches 14 rows, none of them with
  `nnz <= 886` except `sporttournament18` at 3e-7, so the draw's *value* lives on
  rows with `nnz >= ~5 000`, while its *charge* is paid on all 238 drawn rows.
  That asymmetry is real but only ~1-2 s of corpus per rung, so it was not worth
  spending this round's submission on.
- A fill-shaped cap on the draw (skip the draw when the incumbent's flops are
  large) was measured and rejected: the charge is *anti*-correlated with fill, so
  such a cap would remove cheap draws and keep expensive ones.

## Caveats

- The dev corpus cannot rank anything inside `10 000 < n <= 50 000`: no dev row
  there has ever had a non-zero chain or draw yield, so the hidden value of
  restoring the chain's scope above `12 000` is an extrapolation from the
  `71c2c5fe` receipt, not a local measurement.
- Local cap verdicts on this shared host are not trustworthy in either direction:
  `results.tsv` records the untouched frontier killed at the 2 s cap on rows whose
  in-process `order()` is 0.15-0.43 s, so a local red would not have been
  informative and did not occur here (300/300 OK).
- The mechanism behind the six kills is still not proven; the row-disjointness
  argument is the only reading that is consistent with all eight receipts.

## Learning and next steps

The dev gain is drawn from a single mechanism (the terminal draw) whose value is
concentrated on one row and whose charge is a flat ~0.05 s on every row it
touches; the cap, however, is decided far from those rows. If this submission
completes, its delta against 0.842857 prices the draw's hidden transfer for the
first time, with the chain's wide-band scope restored above the band. If it is
killed again, the row-disjointness reading is falsified too and the next build
must be strictly time-negative relative to the frontier per row rather than
differently shaped.
