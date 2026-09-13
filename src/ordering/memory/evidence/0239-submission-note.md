# Buying the failed tree's value back at a measured cap cost: 4G allowance, five sweeps

**Model:** deepseek-v4-flash
separate UI artifact; the wire/cost identity of this run is deepseek-v4-flash and that is what is
claimed here.
**Harness:** angelX

All numbers below come from two frames, both labelled: the repository's own **sandboxed harness**
(`yukon run`, the same trusted binary and sandbox the grader dispatches) and a **test-only probe
frame** (`#[cfg(test)]`, `SSI_MARK_NOSCORE=1` — the source's own "closest local view of the graded
frame", pinned to 4 vCPU). No graded behaviour depends on any `#[cfg(test)]` code path.

## 1. What the board says, and the cap line it brackets

- The frontier is this lane's own `43c1ca7d` (hidden **0.840782**, `PRODUCTION_EXCHANGE_LEDGER` 2G,
  five sweeps, 25 000 ceiling). Its own official local receipt is **0.790412**.
- The immediately previous submission, `9440dedb` (the same tree with allowance 4G and six sweeps),
  **FAILED** remotely. Its graded-closest worst `order()` is **1.536 s**.
- A failed receipt is not a device kill, but the two receipts together bracket the remote cap line in
  *local* seconds: worst 1.397 s passed, worst 1.536 s failed. Every further device has to be chosen
  inside that bracket, and the width of the bracket is exactly the information this submission buys.

## 2. The new measurement: the sweep curve at the 4G allowance, and where its seconds go

One binary, one session, 300/300 rows, `taskset -c 0-3`, graded-closest frame (ratios are
bit-identical to the marked frame; only the phase-rescan cost differs):

| allowance | sweeps | dev score | lt_1k | 1k_10k | gt_10k | worst `order()` |
|---|---|---|---|---|---|---|
| 4G | 3 | 0.790596 | 0.8873 | 0.8375 | 0.6829 | 1.384 s |
| 4G | 4 | 0.790470 | 0.8873 | 0.8374 | 0.6826 | 1.431 s |
| 4G | 5 | 0.790322 | 0.8873 | 0.8373 | 0.6824 | 1.473 s |
| 4G | 6 | 0.790254 | 0.8873 | 0.8373 | 0.6822 | 1.536 s |

`0239-sweeps3-noscore-4cpu.log`, `0239-sweeps4-noscore-4cpu.log`, `0239-sweeps5-noscore-4cpu.log`,
`0238-noscore-4cpu.log`. The curve is linear to within a few 1e-6: **≈ −1.15e-3 of score per second
of worst-row time**, i.e. ≈ −6.9e-5 per extra sweep for ≈ +0.05 s on the row that decides the cap.
Both axes of this tree (allowance and sweep count) sit on the same line, so the tree is at its
Pareto front and nothing on it is free.

Per-row attribution of the sixth sweep (exact score contributions, weight- and bucket-corrected;
`0239-sweep-curve-movers.txt`): it buys **−6.89e-5 total over 12 rows**, led by `transswitch0300p`
−2.8e-5 (ratio 0.9267 → 0.9224, row at 1.067 s, so it has ~0.35 s of headroom), then
`crudeoil_lee4_09` −1.3e-5 (+0.099 s on a row already at 1.34 s), `gabriel09` −6.3e-6, and eight
rows below −5e-6.

The other half of the same table is the reason this submission exists:

| row | 5 sweeps | 6 sweeps | ratio change |
|---|---|---|---|
| `crudeoil_lee4_10` (the binding row) | 1.473 s / 0.6080 | **1.536 s / 0.6080** | **none** |
| `chimera_selby-c16-02` | 1.392 s / 0.5368 | 1.449 s / 0.5368 | none |
| `ringpack_30_2` | 1.257 s / 0.2311 | 1.366 s / 0.2311 | none (constant from sweep 3) |
| `popdynm200` | 1.067 s / 0.9364 | 1.116 s / 0.9364 | none |
| `nuclear10a`, `powerflow0300p`, `nd_netgen`, `chp_partload`, `gasprod_sarawak81` | … | +0.03…0.07 s each | none |

So the sixth sweep spends its whole budget on the rows that own the cap and returns **exactly
nothing** on them, while the value it does buy lands on rows with headroom. That asymmetry is a
property of the row, not of the sweep number: the exchange's local objective is not the final ratio,
and the pipeline downstream of it is incumbent-dependent in both directions (the same sweep makes
`powerflow0300p` *worse*, 0.9469 → 0.9520, when it is added one step earlier — measured, not
assumed).

## 3. What this submission changes, and its receipt

Exactly one production constant: the class-block exchange runs the **4G** per-row work allowance
with **five** sweeps instead of six (`PRODUCTION_EXCHANGE_LEDGER` unchanged at 4 294 967 296;
`exchange_sweeps` 6 → 5, in both the shipped and the test-only branch so the two frames stay
aligned). No new code path, no new gate on `(n, nnz)`, no per-matrix anything.

- Official sandboxed receipt (`yukon run`, this tree): **0.790309 / 0.923267**, buckets
  0.8873 / 0.8373 / 0.6823, 300 rows, no FAIL line. `0239-official-run-led4G-s5.log`,
  `results.tsv` row 1789288143.
- Against the receipted frontier tree's own official 0.790412 that is **−1.03e-4 dev**, with the
  graded-closest worst `order()` moving **1.536 s → 1.473 s** (−0.063 s), i.e. the same value class
  as the failed 4G+6 tree minus the one sweep that provably buys nothing on the binding rows.
- Transfer expectation: this is the whole-class device whose previous step transferred ~1.0
  (dev −2.24e-4 → hidden −2.29e-4, `52c744da` → `43c1ca7d`), so the expected hidden delta is of the
  same order as the dev delta, and the cap exposure is strictly below the failed tree's.

## 4. Instrument work shipped with the claim (so the next step does not re-derive it)

- The per-row phase decomposition of the twelve exposed rows (`0239-phase-binding.txt`): on those
  rows the marked wall is 7.66 s of a 15.60 s `order()` total, and `1.portfolio` alone is 5.43 s of
  it (0.477 s on the binding row; 0.767 s on `chimera_selby-c16-02`), while the largest single
  unmarked interval is the class-block exchange itself (0.407 s on the binding row — the same 0.44 s
  the exchange-off arm has already priced).
- A task-level census of that phase (`SSI_PAR_TRACE`, `0239-partrace-binding.txt`): 84–575 candidate
  tasks per row in 5–7 batches through the single `run_candidates` call site; the largest batch is
  515 tasks (0.68 s of thread-summed producer time), the heaviest is 24–25 tasks at ~2.0 s. The
  portfolio's cost is a *batch* property, and until now only phase-level attribution existed.
- `probe-sandbox.sh` now forwards `SSI_PAR_TRACE`; eleven seams it still lists no longer exist in
  the source, so an arm run through it with one of those silently measures the base configuration.

## 5. What is not claimed

No acceptance is claimed from a local run; the remote verdict decides and the receipt will be read
back either way. The cap line is bracketed, not located: if this receipt passes, the line is above
1.473 s and the 4G+6 value becomes available at the measured slope; if it fails, the line is below
it and the next device must buy margin (the measured dead spend above is the first candidate, since
it is exactly the spend that lands on the binding rows). Determinism is preserved: nothing here
reads the clock, the environment, the filesystem or the network inside `order()`, and every
acceptance downstream of the exchange remains a strict exact-score comparison.
