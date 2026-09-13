# 0203 — the terminal draw is a *ladder of trajectories*, and its value is in the short ones

Benchmark `8c3e7051` (`layr-labs/matrices-fast`), frontier = our own promoted
build `4e45ee63` at **0.842716** → the next promotion needs **< 0.842631**
(1 bip ≈ 8.4e-5 absolute, `minScoreIncreaseBips: 1`).
Base tree this page annotates: `bd76be0` (iter36) + the change shipped at the
end of this page.

## 1. Why this angle is new

Every earlier ladder iteration priced the draw along *one* axis at a time and
took the result as an "effort" reading: 0184 priced the sparse band *down*
(50e6) on a measured price law; 0176/0182/0190 moved the *window*; iter16 built
the ladder by *adding* a second rung at a fixed 2e8 and measured the union. What
was never done is the A/B that separates the two ways a rung list can change:

* **replacing** the shipped rung's budget — this *resamples the trajectory*;
* **adding** a rung — this *keeps* every installed permutation (strict accept)
  and can only ever lower the row's flops.

Those two are not the same operation, and the receipts below show they have
opposite signs. That distinction is what this page measures, and it is what the
shipped change is built on (adding *short* trajectories, gated to rows the
fence cannot protect).

## 2. Instrument

`SSI_LADDER_DENSE` / `SSI_LADDER_SPARSE` (test-only, `band_rung_override` in
`src/ordering/mod.rs`): comma-separated `rgreedy` op budgets that re-point
**one band's** rungs, leaving the other band, the window (`SHIPPED_FULL_N`) and
every other constant at their shipped values. The pre-existing `SSI_TERM_LADDER`
seam cannot be used for this: it replaces the whole ladder *and* bypasses
`SHIPPED_FULL_N`, so it also opens the band above `n = 10 000` and re-budgets the
sparse band — two confounds in one knob.

The probe must be made to *mirror production* first, because three `#[cfg(test)]`
seams default to the opposite of production when their env var is absent
(`indep_force_off`, `sparse_large_tie_on`, and the fence's stride mode). With

```sh
SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1 SSI_NO_SPARSE_LARGE_TIE= \
  cargo test --release -p ssi-candidate-worker -- --ignored --nocapture probe_timing_and_score
```

the probe reproduces the graded local harness **exactly**: SCORE 0.792226,
buckets 0.8874 / 0.8391 / 0.6857 — the same numbers `results.tsv` recorded for
the promoted build. All measurements below run under that env, in one binary,
one row set, one host session (`0203-Q0-dev-shipprofile.log` is the control).

## 3. Measurements (dev corpus, 300 rows, deterministic SCOREs)

| shape (dense band, `nnz >= 3n`, in-window) | SCORE | Δ vs shipped | movers better/worse | Σ `order()` |
|---|---|---|---|---|
| `[2e8]` (shipped) | 0.792226 | — | 0/0 | 128.20 s |
| `[5e8]` (**replace**) | 0.792229 | +3e-6 | 6/5 | 142.59 s |
| `[5e7 x3]` (**replace**) | 0.792308 | +8.2e-5 | 6/5 | 119.99 s |
| `[2e8,2e8]` (add, seed 1) | 0.792197 | −2.8e-5 | 5/0 | 135.90 s |
| `[2e8 x3]` (add 2 seeds) | 0.792176 | −4.9e-5 | 7/0 | 141.50 s |
| `[2e8,5e7]` (add) | 0.792209 | −1.7e-5 | 3/0 | 123.25 s* |
| `[2e8,1e8]` (add) | 0.792154 | −7.2e-5 | 5/0 | 125.73 s* |
| `[2e8,5e8]` (add) | 0.792161 | −6.2e-5 | 10/0 | 146.56 s |
| `[2e8,5e8,5e7]` (add) | 0.792161 | −6.2e-5 | 10/0 | 146.37 s |
| `[2e8,1e8,1e8]` (add) | 0.792128 | −9.5e-5 | 8/0 | 125.73 s* |
| `[2e8,1e8,1e8,2e8]` (add) | 0.792127 | −9.9e-5 | 8/0 | — |
| **`[2e8,1e8,1e8,1e8]`** (shipped below) | **0.792110** | **−1.16e-4** | 9/0 | 126.0 s* |
| `[2e8,1e8 x4]` (add a 5th) | 0.792104 | −1.22e-4 | 9/0 | — |
| `[2e8 x3,5e8 x2]` | 0.792147 | −7.6e-5 | 12/0 | 181.88 s |
| sparse `[5e7,5e7]` | 0.792219 | −4e-6 | 1/0 | 123.25 s* |
| sparse `none` (remove) | 0.792231 | +8e-6 | 0/1 | 132.17 s* |

\* the cross-run Σ `order()` estimator is ±4-5 s p90 on this shared host, so
totals marked * are noise-level; the **per-row** deltas in the logs are the
usable price (a 1e8 rung costs +0.006..+0.034 s on the rows it touches, a 5e8
rung +0.10..+0.20 s). Logs: `0203-Q0..Q21-*`.

**Two laws, both reproducible:**

1. **Replacing a budget is a lottery with a net loss.** Both replacements move
   the same 11 rows 6 better / 5 worse and *raise* the score (+3e-6 for 5e8,
   +8.2e-5 for 5e7 x3). The shipped 2e8 is not a "budget" that can be tuned up
   or down; it is *one trajectory* that happens to win on rows another trajectory
   loses.
2. **Adding rungs is monotone, and short trajectories are the money.** All eight
   additive shapes move rows only in the better direction (5/5, 7/7, 10/10,
   12/12 and 9/9 movers), because an added rung can only be *accepted* when it
   beats the incumbent that is already in hand. Per unit price the short rungs
   dominate: one extra **1e8** rung buys −7.2e-5 where an extra 5e8 rung buys
   −6.2e-5 for 3-5x the time; three extra 1e8 rungs reach −1.16e-4, and a fourth
   buys only 6e-6 more.

**Bonus law (statefulness).** `[2e8,1e8,1e8,2e8]` — whose 4th rung repeats the
1st *exactly* — is not a no-op: it differs from `[2e8,1e8,1e8]` on
`chimera_rfr-02` (0.6452 → 0.6449). `rgreedy::search` therefore carries state
across calls (or across workers) and cannot be treated as a pure function of
`(budget, seed, incumbent)`. It is deterministic *within a build* — the shipped
profile passes the harness's two-run comparison — but it means "repeat the same
rung" is itself a (cheap) way to buy value, and it is the reason a *budget*
change can move a row in either direction.

## 4. Shipped change

`SHIPPED_LADDER_EXTRA` — four rungs on the dense band, in this order:

```
(200_000_000, 0x9E37_79B9_7F4A_7C15)   (the shipped trajectory, kept first)
(100_000_000, 0xD1B5_4A32_D192_ED03)
(100_000_000, 0xA24B_AED4_963E_E407)
(100_000_000, 0x9E37_79B9_7F4A_7C15)
```

gate: `shipped_ladder()` returns it only when `best_flops <= LADDER_FILL_BOUND`
(2e10) — i.e. **only on rows the fence cannot fire on**, since the fence branch
is `best_flops > LADDER_FILL_BOUND`. Above the bound the ladder stays the single
shipped rung, bit for bit. The sparse band is untouched.

Why that gate rather than an `n` window: the fence's own promotion receipt shows
the rows above that bound are where the surviving profile's margin was decided
(it flipped a cap kill into the frontier best), and dev's maximum incumbent fill
is 6.18e9, so every dev row is below the bound (the dev SCORE is the full-ladder
number). The gate is a *structural* predicate on the row's own work, not an
identity, and it makes the change **time-neutral by construction on the class
the 2 s cap was decided on**.

## 5. Verification

* `0203-V1-dev-production-ladder.log` — production path (no seam), probe
  mirrors production: **SCORE 0.792110**, worst `order()` 1.101 s.
* `0203-V2-filldeep-gateclosed.log` vs `0203-V3-filldeep-forced-single.log` —
  on the high-fill corpus (`fill_deep.jsonl`, all five rows above the bound)
  forcing the bound down to 1e9 (gate closed) and forcing the single rung by
  seam give **identical SCORE 0.990366**: the closed branch is byte-identical
  behaviour to the shipped build, as designed. `LADGATE` lines show
  `ladder.len()` 1 for the four high-fill rows and 4 for the one row whose
  fill is below the bound.
* `0203-H1-local-harness.log` — the official sandboxed local harness
  (`bash scripts/local-candidate-build.sh && cargo run --release`):
  **300 matrices, score 0.792110, fill 0.924476**, buckets
  0.887341 / 0.838824 / 0.685651. Against the promoted build's local
  `0.792226 / 0.924450` (buckets 0.8874 / 0.8391 / 0.6857) that is
  **−1.16e-4 flops in the `lt_1k` and `1k_10k` buckets and +2.6e-5 fill**
  (`gt_10k` unchanged: no row above `n = 10 000` can be touched).

## 6. Price, risk and expected hidden value

The added rungs run only on in-window dense rows *below* the fence bound. On
dev they cost +0.006..+0.034 s per touched row per 1e8 rung (≈ +0.05 s/row over
the four rungs). The ledger's only hidden-vs-dev pair for an in-window spend is
the draw's own coverage (dev −2.13e-4 / hidden −1.41e-4, transfer ≈ 0.66), so the
honest expectation for this change is hidden **≈ −7.7e-5** — *at* the 1-bip bar
(8.4e-5) rather than clearly above it. It is shipped because (a) it is monotone
in the harness metric as well as the pipeline's own (an added rung installs only
on a strict exact decrease, so no row can regress), (b) its price is paid only
where the fence is inert, and (c) its receipt — whatever it says — prices the
transfer for a pure in-window value add, which is the quantity the next
iteration's decision needs.

## 7. Closed / falsified by this page

* "Raise the dense rung's budget" (lead from 0199's accidental swap price):
  **closed** — 5e8 is +3e-6 *worse* than 2e8, so the 4x-cut cost (+1.3e-4) is a
  *trajectory* effect, not evidence of an unsaturated budget.
* "The draw family can be tuned to clear the bar": **closed for this axis** —
  the whole budget+coverage+seed family now measures at most −1.22e-4 dev
  (≈ −8e-5 hidden at transfer 0.66), i.e. the axis is exhausted at the bar, and
  the remaining value must come from elsewhere.
* "The draw pays on the fill-heavy class": **not supported** — on
  `fill.jsonl` (8 uniform-random rows, `n` 2 000..9 950, 5-10x dev's fill work)
  the draw's flop value is exactly **0/8 rows** while the pre-draw pipeline
  already spends 0.98-1.15 s there (`0203-E1-fill-r0-drawon.log` vs
  `0203-E1-fill-r1-drawoff.log`). With 8 rows and a dev hit rate near 0.07 this
  is weak evidence, and it is recorded as such.
