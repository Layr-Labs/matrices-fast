# iter75 — the exchange ledger's *charge shape* is shipped, and the probe's code frame is corrected

Model: **deepseek-v4-flash
Editable path touched: `src/ordering` only (`rgreedy/window_dp.rs`, `mod.rs`, plus the local
evidence ledger under `src/ordering/memory/`).

Local score of this tree, official sandboxed harness (`bash scripts/local-candidate-build.sh &&
cargo run --release`): **0.790277 / 0.923142, 300/300 rows OK, no FAIL** (the tree this
replaces reads 0.790282 / 0.923147 on the same box). The truthful reading of that number is
below.

## 1. What changed

1. **The ledger's charge *shape* is now live in the graded build.**
   `window_dp.rs::xch_charge_scale` existed but its only consumer was `#[cfg(test)]`-gated, so
   every production charge was exactly 100 % — the whole rung was unreachable by a submitted
   tree. Production now charges **75 % of the modelled price for window components of width
   `k >= 11`** (`PRODUCTION_XCH_BIGK_MIN = 11`, `PRODUCTION_XCH_BIGK_PCT = 75`) and the
   multiplier is unconditional.
2. **The probe's env-seam defaults were not the production program** (test-only fix, recorded
   for the ledger): `exchange_sweeps` defaulted to 6 while production compiles 12, and
   `preclass_win` / `preclass_step` defaulted to *true* while production compiles *false*. Any
   probe run that did not set those two seams measured a different program — including the
   iter72 price of the charge shape itself, which is why that price was wrong. Defaults now
   equal the production arms; the seams still override.

## 2. Value, in three frames (the important part)

| frame | baseline | this tree | delta |
|---|---|---|---|
| probe, seams pinned to production (300 rows) | 0.790254 | 0.790203 | −5.1e-5 |
| official local sandboxed harness | 0.790282 | 0.790277 | **−5.0e-6** |

The two frames do not agree, and the reason is measured, not guessed. Comparing the official
harness's per-row table with the probe's on the *same* tree, 4 of 300 rows diverge
(`gabriel09` 0.897 vs 0.8695, `crudeoil_pooling_dt3` 0.691 vs 0.7056, `popdynm200` 0.932 vs
0.9364, `gasprod_sarawak81` 0.910 vs 0.9118, all in the 40 %-weight `gt_10k` bucket); their
log-differences sum to exactly the −2.8e-5 score offset. Both frames are internally
deterministic (the worker gives identical permutation md5s under `ulimit -v 4G`, 1 core, 4
cores, 24 cores; the probe is stable across affinities), so the *test* build is a behaviourally
different program on those 4 rows.

Consequence for this device: in the worker frame, of the 13 rows the probe moves, only **two**
change their permutation (`ndcc12`, `chimera_selby-c16-01`). The probe's largest movers
(`procurement1large` −0.0014, `crudeoil_lee4_09` −0.0006, `crudeoil_pooling_dt3` −0.0006,
`crudeoil_lee4_06` −0.0005, `mpbp_48` −0.0005, `crudeoil_lee1_07` +0.0013) are
permutation-identical in the graded frame. So I claim a **small** graded-frame gain
(−5e-6 dev), not the probe's −5.1e-5; a value device of this shape must be confirmed by a
worker-frame permutation change or by an official run, and that is the standing rule this
submission is recorded under.

## 3. Wall (true frame: production worker, contract patterns, 4-core, 3 reps, medians)

| row | current tree | this tree | delta |
|---|---|---|---|
| `chimera_selby-c16-01` | 1.110 | 1.130 | +0.020 |
| `chimera_selby-c16-02` | 1.160 | 1.170 | +0.010 |
| `arki0016` | 1.280 | 1.280 | 0.000 |
| `arki0013` | 1.230 | 1.220 | −0.010 |
| `crudeoil_lee4_09` / `_10` | 1.200 / 1.210 | 1.190 / 1.200 | −0.010 |

12 of the 17 measured rows are equal-or-faster; the corpus maximum does not move. No row is
made materially more cap-expensive: the ledger is a *fixed* budget, so discounting the wide
components simply re-allocates it, and the narrow components are refused earlier.

## 4. Correction to the record: the true-frame max row is `arki0016`

`arki0016` (n = 7 993, nnz = 37 208 — a *small* row) reads 1.27/1.28/1.29 s in the true 4-core
worker frame, above `arki0013` 1.23 and every row of the previous crown census (whose worst was
1.20-1.23 on `crudeoil_lee4_09`/`_10`/`arki0013`). The probe agrees independently (its slowest
row is `arki0016` 1.3996 s, then `rsyn0840m04m` 1.33, `chp_partload` 1.31,
`transswitch0300p` 1.28). With the same-day pass/kill pair (this lineage's tree promoted, the
3 GiB arm's 1.31 s row cap-killed) the hidden-slowdown bracket tightens from f ∈ (1.53, 1.64] to

    f ≤ 2.0 / 1.28 = 1.5625   and   f > 2.0 / 1.31 = 1.53   ⇒   f ∈ (1.53, 1.5625]

i.e. the local 4-core survival line is ≈1.28 s, not 1.24-1.31, and the shipped lineage's margin
is 0.02-0.03 s on `arki0016`. This candidate does not spend that margin (§3), but any further
schedule-depth rung must be funded on that row first.

## 5. Reproduce

```sh
bash scripts/local-candidate-build.sh && cargo run --release        # official local harness
env SSI_EXCHANGE_SWEEPS=12 SSI_PRECLASS_WIN=0 SSI_PRECLASS_STEP=0 SSI_MARK_NOSCORE=1 \
  taskset -c 0-3 cargo test --release -p ssi-candidate-worker -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score      # probe, production frame
# worker-frame wall: production worker on contract .pat, taskset -c 0-3, 3 reps
```
Evidence ledger: `src/ordering/memory/evidence/0275-probe-code-frame.txt`.
