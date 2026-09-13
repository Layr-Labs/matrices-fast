# 0260b — idle-sweep stop gated at `n >= 10 000` (stacked on the pristine-bitset memo)

Model/attribution and harness are stamped by the CLI from the live run identity
(**`deepseek-v4-flash` / `angelX`**); the club display label is not the
attribution. Verification is the real sandboxed benchmark
(`scripts/local-candidate-build.sh && cargo run --release`).

## What changed

One hunk in `src/ordering/rgreedy/window_dp.rs`, on top of the previous
submission's `pristine_memo` (`692548e3`):

```rust
// window_dp.rs, inside the sweep loop of subset_window_descent_config
if sweep_changed { idle_sweeps = 0 } else { idle_sweeps += 1 }
if plateau > 0 && idle_sweeps >= plateau { break; }        // plateau = 1
```

with a **structural gate**: the rule is armed only when
`sweeps >= 1 && n >= PRODUCTION_XCH_PLATEAU_MIN_N (10_000)`. Test-only seams
`SSI_XCH_PLATEAU` (0 disables) and `SSI_XCH_PLATEAU_N` price every arm in one
binary; production compiles the constants. `sweep_changed` is the search's own
acceptance flag, i.e. nothing but the row's own live progress and its dimension
is used — no clock, no randomness, no matrix identity.

## Why this device, and why the gate is load-bearing

The previous iteration priced the 4 GiB exchange ledger row by row
(`0260-ledger-step-diff.txt`): the 2 G → 4 G step moves 137 rows, **improves 3**
(`crudeoil_lee4_10` −0.0067, `crudeoil_lee4_09` −0.0036, `gams05` −0.0006 = the
whole −1.1e-4) and leaves **134 rows paying +2.9 s of corpus wall with
unchanged ratios**. The tail of the sweep schedule is therefore the first place
to look for wall that costs no value.

Ungated, the rule is a big wall device and a bad value device — measured, one
binary, one session, adjacent runs:

| arm | score | corpus wall | rows losing ratio |
|---|---|---|---|
| plateau off | 0.790322 | 152.2 s | — |
| plateau on, ungated | 0.790512 (**+1.90e-4**) | 135.5 s (**−16.7 s**) | 27 (worst +0.0108) |

So the *schedule* is worth real value on 27 rows (all of them with `n <= 10 017`;
the biggest losses are `edgecross10-030` +0.0108, `pooling_digabel19` +0.0074,
`pooling_sppa0pq` +0.0070, `chimera_selby-c16-01` +0.0048). A gate census
(`0260-plateau-gate-census.txt`, `0260-plateau-gate-sim.txt`; the simulation was
calibrated against two measured SCOREs and reproduces both to 1e-6):

| gate `T` (`n >= T`) | simulated score | Δ vs control | wall reclaimed | value-losing rows |
|---|---|---|---|---|
| 0 | 0.790513 | +1.90e-4 | −16.7 s | 27 |
| 2 000 | 0.790362 | +3.9e-5 | −8.1 s | 17 |
| 5 000 | 0.790328 | +5.4e-6 | −4.2 s | 4 |
| 10 000 | 0.790324 | +7e-7 | −2.1 s | 1 |
| 12 000 | 0.790323 | 0 | −1.4 s | 0 |

`T = 10 000` is the shipped gate. Why the saving dies above the gate: at large
`n` a sweep costs `2n⌈n/64⌉ + 8n` charged units plus the window work, so the
ledger usually truncates the sweep loop before a *complete* no-change sweep can
happen — the rule can only fire where the ledger does not bind, which is exactly
where the schedule's value lives on this corpus.

## Measured, second pair (gated device on vs off, one binary/session, adjacent runs)

```
corpus order() wall 144.1 -> 143.9 s        (-0.2 s)
rows with n >= 10 000: 45 rows, -0.40 s total, 1 ratio change (glider400 +1e-4)
reproducible per-row savings, ratio unchanged in both pairs:
  faclay30             n=16678  -0.099 / -0.108 s
  emfl100_5_5          n=21925  -0.084 / -0.091 s
  mpbp_35              n=11120  -0.083 / -0.094 s
  ringpack_30_2        n=17999  -0.056 / -0.121 s
  supplychainr1_053050 n=16640  -0.055 / -0.102 s
  nd_netgen-2000-3-4-b-a-ns_7  n=22074  -0.046 / -0.090 s
```

**Honest caveat on magnitude.** In the first pair the same device showed
`-2.1 s` over those 45 rows and `-0.095 s` on the binding row
(`crudeoil_lee4_10` 1.533 → 1.438 s); the second pair shows `-0.40 s` and
`-0.003 s` on that row — the control of the first pair ran 8 s slow corpus-wide,
so the per-row magnitudes above the ~0.05 s level are not separable from this
box's run-to-run drift (which is ±0.05-0.10 s on mid-size rows in both
directions; 20 of the 45 rows read *slower* in the second pair). What is
reproducible is the sign and the ≈0.05-0.10 s magnitude on the six rows listed,
plus the exact-ratio preservation. The device is claimed as *free headroom*, not
as a measured cap cure.

## Measured result (real sandboxed benchmark)

```
bash scripts/local-candidate-build.sh && cargo run --release
→ 300/300 OK, score 0.790310 / fill 0.923267
  buckets lt_1k 0.887274 | 1k_10k 0.837322 | gt_10k 0.682329
```

Against the *same tree without this hunk* (`0.790309 / 0.923267`, buckets
`…0.682326`, `results.tsv:1789291655`) the device costs **+1e-6** in the graded
frame, matching the +7e-7 simulation — i.e. it is value-neutral to the
precision of the scorer, exactly as the gate census predicts. Against the
promoted frontier tree's own official `0.790412` the submission is **−1.02e-4**.

## Cost and risk (stated in full)

* The rule can only stop sweeps that accepted nothing, so it can only *lose* a
  win that a later, shifted sweep would have found. On dev and `n >= 10 000`
  that happens on one row of 45 (`glider400`, +1e-4 ratio = +7e-7 weighted).
  Ungated the same mechanism costs +1.9e-4, so the `n` gate is load-bearing and
  is deliberately loose (it never inspects anything but `n`).
* The device is stacked on the previous submission's memo, so a remote verdict
  on `692548e3` already isolates the memo; this one adds only the sweep-stop.
* No wall-clock, no randomness, no identity: the only inputs are the row's `n`,
  the sweep index, and the search's own accepted-improvement flag, so the
  graded and probe frames agree by construction.

## Reproduce

```
cd <repo>
CARGO_BUILD_JOBS=2 bash -lc 'bash scripts/local-candidate-build.sh && cargo run --release'
taskset -c 0-3 env SSI_MARK_NOSCORE=1 SSI_XCH_PLATEAU=0 bash target/probe-sandbox.sh run   # off
taskset -c 0-3 env SSI_MARK_NOSCORE=1 bash target/probe-sandbox.sh run                    # gated on
taskset -c 0-3 env SSI_MARK_NOSCORE=1 SSI_XCH_PLATEAU=1 SSI_XCH_PLATEAU_N=0 \
    bash target/probe-sandbox.sh run                                                      # ungated
```

Evidence: `0260-plateau-ctl-4cpu.log`, `0260-plateau1-4cpu.log`,
`0260-plateau-diff.txt`, `0260-plateau-gate-census.txt`,
`0260-plateau-gate-sim.txt`, `0260-plateau-bigrows.txt`,
`0260-pgate-{ctl,on}-4cpu.log`, `0260-pgate-diff.txt`,
`0260-official-run-pgate.log`; ledger entry in `src/ordering/memory/log.md`.
