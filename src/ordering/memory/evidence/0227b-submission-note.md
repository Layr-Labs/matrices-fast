# The class block's exchange window is a trajectory set, not a knob: (8,4,3) -> (12,4,5) is -1.23 bips in-frame

**Model / effort:** DeepSeek V4 Flash, single-agent loop (angelX harness).
**Base:** our own promoted `e07fe7ae` (9 sparse-span windows, hidden 0.841768) plus the
deferred-lift parity device (submitted minutes earlier as `41baf1ce`). This note prices the
next change on top of both.
**Local score:** `yukon run` 300/300 **OK 0.791383 / 0.924003** (`results.tsv:1789265092`);
the same tree with the exchange back at its old shape reads **0.791480** in-frame.

## 1. Why this site

The class block (`terminal_exchange`, gated `n <= 12 000 && nnz <= 200 000 && nnz <= 16n &&
max_deg <= n/2`) spends its ledger on `rgreedy::subset_window_descent_step(n, cp, ri, perm,
width, sweeps, offset_step, ledger)` — an exact window search over the current permutation's
suffix graph. The shipped shape was `(8, 4, 3, 512M)`. Two facts made the *shape* worth a sweep:

* the same primitive's **sparse-span** sibling paid 3.0e-5 for widening in iter47 — this is the
  one schedule in the build where "wider" had ever paid;
* the window solve keeps components in their original slots and leaves the suffix graph
  unchanged, so a wider shape is **not** a superset of a narrower one. This is a *set of
  trajectories*, and the only way to price it is to run it.

## 2. Sweep (production frame `SSI_INDEP_FORCE=1`, parity on, one binary, one session, 300 dev rows, `taskset -c 0-3`)

| shape | SCORE | worst `order()` | best single row |
|---|---:|---:|---|
| `(8,4,3)` shipped before this change | 0.791480 | 1.371 s | — |
| `(10,4,3)` | 0.791467 | 1.362 s | — |
| `(12,4,3)` | **0.791369** | **1.562 s** | crudeoil_lee4_10 1.562 s |
| `(14,4,3)` | 0.791460 | 1.435 s | — |
| `(16,4,3)` | 0.791619 | 1.347 s | *worse than shipped* |
| `(12,2,3)` | 0.791501 | 1.442 s | *loses the value: sweeps matter* |
| `(12,4,5)` **<- shipped here** | **0.791383** | 1.435 s | crudeoil_lee4_09 1.435 s |
| `(12,4,3)` at `SSI_EXCHANGE_LEDGER=256M` | 0.791494 | 1.487 s | *the ledger is not the cost* |

Exact per-row diff `(8,4,3) -> (12,4,5)`-class shapes are broad: `(12,4,3)` improves **23 rows /
regresses 3** (`pooling_sppa0pq` −2.15 %, `crudeoil_lee2_06` −0.93 %, `chimera_selby-c16-01`
−0.63 %, ...; regressions `crudeoil_lee4_06` +0.36 %, `transswitch0300p` +0.36 %, `mpbp_35`
+0.03 %) for −1.11e-4 net. The window shape changes *which* local optimum the later stages start
from, so both signs appear — the same path-dependence the parity note documents one stage later.

## 3. Why `(12,4,5)` ships rather than the nominally better `(12,4,3)`

`(12,4,3)` is 1.4e-5 better on dev but pushes `crudeoil_lee4_10` (n=17809, nnz=120632) to
**1.562 s**, against 1.231 s at step 5 — 0.33 s of peak-row margin bought for 1.4e-5. The 2 s cap
is the one budget the frontier has already been killed on (1b764d1, 54646b9, 7bba860, ...), so the
step-5 shape is what ships: 88 % of the value, worst row 1.435 s vs the shipped 1.371 s (+0.06 s).

## 4. Change

`PRODUCTION` constants at the single exchange site: `exchange_width` 8 -> **12**,
`exchange_step` 3 -> **5** (`exchange_sweeps` stays 4, ledger stays 512M). The test seams
`SSI_EXCHANGE_WIDTH/_SWEEPS/_STEP` now default to the production values, so a no-env probe build
reproduces the graded behaviour — verified: no-env probe reads 0.791383, the same digit as the
arm with the env var set explicitly.

```rust
let candidate = rgreedy::subset_window_descent_step(
    n, &pattern.col_ptr, &pattern.row_idx, &best_perm,
    exchange_width, exchange_sweeps, exchange_step, exchange_ledger);
```

## 5. Receipts and caveats

* Probe A/B (one binary, one session, production frame, 4 vCPU): `(8,4,3)` 0.791480 ->
  `(12,4,5)` **0.791383** = −9.7e-5 (−1.23 bips) against the parity tree, i.e. −2.03e-4
  (−2.57 bips) against the promoted 9-window frontier's dev reading (0.791586).
* Official local harness: **300/300 OK 0.791383 / 0.924003** (`results.tsv:1789265092`).
* The first two `yukon run` attempts FAILed the 2 s cap on **tiny** rows — `clay0204m`
  (n=222) and `graphpart_clique-70` (n=280) — which the pinned probe measures at **0.307 s** and
  **0.321 s**. The third attempt, taken when the host's load average fell from 5.1 to 1.9,
  completed 300/300. So those local FAILs are host contention in the spawn-inclusive wall-clock
  cap, not this change (both rows are far below the exchange's gate).
* Honest risk: the exchange shape raises the peak row by ~0.06 s (and the rejected variant by
  0.19 s). If the graded machine is materially slower than this box, that is the exposure.
* Crates: no dependency changes; `src/ordering` only; suite 123 passed / 0 failed / 55 ignored.

## 6. Reproduction

```bash
SSI_INDEP_FORCE=1 SSI_EXCHANGE_WIDTH=8                      taskset -c 0-3 bash target/probe-sandbox.sh run  # 0.791480
SSI_INDEP_FORCE=1 SSI_EXCHANGE_WIDTH=12 SSI_EXCHANGE_STEP=5 taskset -c 0-3 bash target/probe-sandbox.sh run  # 0.791383
yukon run                                                                                                   # 0.791383
```

Evidence: `memory/evidence/0227-xchg-width{8,10,12,14,16}-4cpu.log`,
`0227-xchg-w12-s4-t5-4cpu.log`, `0227-xchg-w12-s4-t5-shipped-4cpu.log`,
`0227-xchg-w12-ledger256M-4cpu.log`, `0227-yukon-run-xchg12-retry2.log`, `results.tsv:1789265092`.

## 7. Next

The same sweep shape applies one site earlier (`SSI_FOLLOWUP_*` spans, already extended) and to
the sparse-span schedule, whose next group of four widths is priced at −1.4e-5 in-frame and is the
natural companion for a follow-up submission (~0.18 bip dev, ~+0.05 s peak).
