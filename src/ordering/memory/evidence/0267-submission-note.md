# The combined value tree, and the first calibrated single-core cap frame

**Model:** deepseek-v4-flash

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**). `minScoreImprovementBips = 1`
≈ 8e-5 absolute at this score, so only candidates whose *hidden* delta clears 1 bip can promote:
the last submission of this lane (`039c8e2d`, 2 GiB + 45 000 ceiling) measured hidden **0.840725**
= −0.57 bip and was rejected sub-threshold.

## 0. Context, environment and prior work

The task is fill-reducing elimination ordering under a hard **2.0 s per-matrix wall cap** charged on
a child process (`src/watchdog.rs`), each matrix ordered **twice** in fresh processes with an
identical permutation required, inside a 4 GiB `RLIMIT_AS` bubblewrap worker (`src/sandbox.rs:40`).
Score = weighted mean of per-bucket geomeans of `flops(yours)/flops(AMD)` with weights
0.30/0.30/0.40 over buckets `lt_1k`/`1k_10k`/`gt_10k` — so one `gt_10k` row is worth ≈3.35 small
rows. Locally everything is run through the repo's own production frame: the harness binary
(`cargo run --release`) spawns `target/release/ssi-candidate-worker` per matrix under
`taskset -c 0-3`; every `SSI_*` seam and timer in `src/ordering` is `#[cfg(test)]`-gated and is
therefore *compiled out* of the graded worker, which is why no env-seam A/B in this lane's record is
a graded-frame measurement.

Prior work in this lane (all measured, all on record): the 2 GiB → 4 GiB allowance ladder, the
descent ceiling 25 000 → 45 000, the anchor gate, the pre-class exchange sites retired, the
min-fill big-restart clamp, and a graded-frame per-row wall census of the production worker. The
frontier is this lane's own 2 GiB tree; everything submitted since with the 4 GiB allowance was
cap-killed, and the one 2 GiB improvement (−0.57 bip) was rejected for being under 1 bip.

## 1. What this tree is

`src/ordering` constant changes only; no per-matrix keys, no identity lookups, no corpus data:

1. `PRODUCTION_EXCHANGE_LEDGER` = **3 GiB** (`3_221_225_472`). The 2 GiB and 4 GiB rungs have both
   been submitted; every 4 GiB tree was killed on the hidden 2 s cap, and the 2 GiB rung measures
   only −0.57 bip hidden. The 3 GiB rung is the middle one.
2. `rgreedy::MAX_N` = **45 000** (the descent ceiling; the 25 000 → 45 000 step is measured at
   −0.87 bip dev on four rows and +0.015 s on the corpus peak).
3. The **min-fill big-restart clamp**: the `n ≥ 2000 || nnz ≥ 10 000` third arm hands its largest
   restart count to the largest rows; clamping it 8 → 2 returned 0.38–0.53 s on each of
   `squfl015-060`, `crudeoil_pooling_ct3`, `chimera_selby-c16-01/02`, `squfl010-080` at unchanged
   ratios, for +1.8e-5 dev *at the 2 GiB rung*.
4. The **anchor gate** (the exact exchange family runs only where some other family already beat
   AMD) and the two **pre-class** exchange sites retired.

Exact commands used, in order:

```
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "iter67 ..."
```

Official local sandboxed harness, production worker, 300/300 rows, no FAIL:
**0.790243 / 0.923082** (buckets 0.8873 / 0.8375 / **0.6820**). Its per-matrix table is identical,
row for row, to the 3 GiB + gate run recorded at `results.tsv:1789307153` (**0 of 300 rows differ**),
so the clamp is value-free *at this rung* while keeping its wall cut. Against the promoted tree's
own dev point (0.790412) this is **−1.70 bip**.

## 2. Two device classes this tree's neighbourhood killed this iteration

* **"Keep the best sweep" is provably inert.** The window exchange's returned permutation is
  adopted only by a strict exact decrease (`mod.rs:5539`), and each `refine_window` acceptance is a
  strict decrease of the caller's own `Σ c_j²` (the eliminated-set fill graph is independent of the
  internal order, so only the window's own columns move). The exchange therefore *always* improves
  on the permutation it was handed, so the last completed sweep is already the argmin, and an
  aborted sweep's partial state is strictly better still. I implemented exact per-sweep bookkeeping
  (returning the eliminated column count out of `TripleWork::eliminate`), built it, and reverted it:
  the device could only ever be a no-op or a regression, and the rebuilt worker was verified
  behaviourally identical (same permutation hash and same wall on all ten crown rows).
* **The 81 anchor-tied rows are not gate-blocked.** 81 of the 300 dev rows end at ratio exactly
  1.0000, and 76 of them are *not* fill-free-optimal, i.e. they look like a large untouched value
  pool. Re-using the repo's own `probe_census` on nine of the heaviest tied rows (four of the five
  `gt_10k` tied rows, plus `meanvar-orl400_05_e_7`, `polygon75`, `knp5-44`, `squfl020-150`,
  `emfl100_3_3`), **every one of the 110 isolated generators is ≥ 1.0000**, with the best always an
  AMD variant. The isolated candidate set is exhausted on those rows, so "lift a gate and let a
  blocked family win" would have bought nothing there.

## 3. The new instrument, and the honest risk

Every cap number in this lane's record was taken at `taskset -c 0-3` on a 24-core box, which left
the hidden frame's slowdown as a 1.818–1.980 bracket. I re-measured the crown rows at **one core**
(`taskset -c 0`, `/usr/bin/time -f "%e %P"`, one worker process per row, alternating the two
ledger binaries row by row so both arms share the session):

| row | 1 core 3 GiB | 1 core 2 GiB | 4 core 3 GiB | 4 core 2 GiB |
|---|---|---|---|---|
| crudeoil_lee4_09 | **1.72** | 1.55 | 1.09 | 1.01 |
| procurement1large | 1.59 | 1.54 | 1.05 | 1.07 |
| methanol400 | 1.38 | 1.21 | 1.08 | 0.91 |
| mpbp_48 | 1.36 | 1.34 | 0.96 | 0.94 |
| chimera_selby-c16-02 | 1.32 | 1.32 | 0.84 | 0.84 |
| crudeoil_pooling_dt3 | 1.29 | 1.17 | 1.02 | 0.92 |
| crudeoil_lee4_10 | 1.28 | 1.19 | 0.82 | 0.74 |
| nd_netgen-3000-1-1-b-b-ns_7 | 1.24 | 1.24 | 0.96 | 0.96 |
| chimera_selby-c16-01 | 1.19 | 1.19 | 0.79 | 0.78 |
| gams05 | 0.75 | 0.76 | 0.48 | 0.49 |

Three facts fall out of that table. The 4-core → 1-core stretch is ≈1.22 on the binding row and
row-specific (1.28–1.58 across the ten), so the hidden runner is *more* loaded than one core of
this box: the 2 GiB tree that **completed the hidden corpus twice** reads a 1-core max of 1.55 s and
the 3 GiB tree that was **killed** reads 1.72 s, which places the 2 s line at ≈ 1.64–1.67 s in
1-core units. **This tree sits 3–5 % over that line**, so I am shipping it as a *calibrated risk
test* of the instrument, not as a safe bet: if it passes, the 1-core line is wrong; if it is killed,
the instrument's line is right and the next candidate must return ≥0.08 s of 1-core wall on
`crudeoil_lee4_09`.

The same table also prices the allowance rung **per row, in the frame the cap is charged in**: one
extra GiB costs +0.17 s of 1-core wall on `crudeoil_lee4_09` and `methanol400`, +0.12 on
`crudeoil_pooling_dt3`, +0.09 on `crudeoil_lee4_10`, +0.05 on `procurement1large`, and **0.00** on
`chimera_selby-c16-01/02`, `mpbp_48`, `nd_netgen` and `gams05`. The rung is free exactly where the
call is truncated by something other than the charge (sweep limit or idle plateau) and expensive
exactly where the charge is the binding constraint — which is why a rung that is worth 0.8 bip of
score can be either free or fatal depending on which row the hidden corpus happens to load.

## 4. Why the slot is worth spending anyway

Every measured value pool left in this tree is ≤0.9 bip dev (the ledger rung −0.82, the ceiling
−0.87, the sweep axis −0.27 per two sweeps), and the 2 GiB profile that is *known* to survive the
cap is measured sub-threshold on the hidden frame. This tree is the only measured combination whose
dev delta is large enough to clear 1 bip hidden if it survives, and the daily corpus rotation means
the hidden killer row is not necessarily the row that decides the local census.

## 5. Caveats and next steps

The 1-core line is a *local* emulation, not the grader: it assumes the hidden slowdown is
load-like, and the row-specific stretch (1.28–1.58) is itself measured on ten rows only. Nothing in
this submission keys on matrix identity: the only structure-dependent constants are the ledger, the
descent ceiling and the restart clamp. Next, in order: (a) read this submission's receipt and either
trust or discard the 1-core line; (b) instrument the exchange's own per-call wall split
(`build_adj`/`Game::new`/`Game::reset` versus the window DP) on the binding row and make the reset
path cheaper at identical output — that is the only remaining source of ≥0.08 s on
`crudeoil_lee4_09` that does not touch the rows the 3 GiB rung pays for; (c) if the sweep axis must
be cut instead, cut it only for rows whose call is charge-truncated, never globally.
