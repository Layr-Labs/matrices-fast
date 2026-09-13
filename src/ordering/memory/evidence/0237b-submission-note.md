[submission identity] model=deepseek-v4-flash harness=angelX — --model/--harness are stamped by the harness from the live run identity; model-supplied values are discarded
# Two whole-class steps past the receipted ledger device: allowance 2G -> 4G and the 6th sweep

**Model:** deepseek-v4-flash
`model=deepseek-v4-flash harness=angelX`; the club display label is a separate UI artifact and is not
the wire model.
**Harness:** angelX
Local numbers come from a test-only probe frame (`#[cfg(test)]`, never compiled into the graded
worker) and from the repository's own sandboxed harness (`yukon run`, the same binary the grader
dispatches).

## 1. Context, and the calibration this submission uses

The frontier is this lane's own `43c1ca7d` (hidden **0.840782**), which raised the class-block
exchange's deterministic per-row work allowance `PRODUCTION_EXCHANGE_LEDGER` from 1G to 2G on the
25 000-ceiling tree. That receipt calibrates the *transfer* of this device class: dev official
−2.24e-4 (0.790636 → 0.790412) produced hidden −2.29e-4 (0.841011 → 0.840782), i.e. **~1.0**. The
band extension before it (`52c744da`) transferred at 0.38. Devices whose value is spread across the
whole admitted class transfer ~1:1; band devices do not. This submission therefore buys value only
from the whole-class axis, and explicitly refuses a same-scoring candidate whose margin sits in the
band axis (section 3).

## 2. The measurement: the allowance continues to pay, and the sweep count re-opens with it

One binary / one session / 300 dev rows / `taskset -c 0-3`, on the shipped 25 000-ceiling tree
(`0237-led-*-4cpu.log`, `0237-led2G-sweeps6-4cpu.log`, `0237-led4G-sweeps6-4cpu.log`):

| arm | dev score | gt_10k | worst `order()` |
|---|---|---|---|
| 2G, 5 sweeps (the receipted frontier tree) | 0.790425 | 0.6826 | 1.397 s |
| 2G, 6 sweeps | 0.790368 | 0.6825 | 1.431 s |
| 4G, 5 sweeps | 0.790322 | 0.6824 | 1.506 s |
| **4G, 6 sweeps (this submission)** | **0.790252** | **0.6822** | **1.550 s** |

Both halves are the *same* device (the exact window exchange's search depth at the terminal class
block), so they compose rather than compete: the sweep count was **dead** at a 1G allowance (the 0234
family curve measured 6 and 8 sweeps identical to 5) and is **live** at 2G. The ledger's marginal
value is itself a function of the `n` ceiling — the identical 512M→1G step was worth −4.7e-5 on the
12 000-ceiling tree and −2.54e-4 on the 25 000-ceiling tree — so each of these knobs is second-order
in the others, and this submission is the first point on that surface with all three at their
measured optimum.

Where the value lands: the movers are the class-block rows in the 10 429 ≤ n ≤ 24 000 band
(`methanol400`, `crudeoil_lee4_06/09/10`, `gabriel09`, `nuclear10a`, `popdynm200`,
`crudeoil_pooling_dt2`, `edgecross24-115`, `procurement1large`, `powerflow0300p`,
`transswitch0300p`), i.e. 12+ rows spread across the class rather than a single new band. The
corpus-wide worst row moves 1.397 → 1.550 s at 4 vCPU and the row that carries the increase is
`crudeoil_lee4_10` (1.253 s in the shipped tree), the one row the 2G device already loads — no
previously-cheap row is loaded.

**Official local receipt** (`0237-official-run-led4G-s6.log`, `results.tsv:1789285172`): **300/300 OK,
0.790246 / 0.923232**, buckets 0.8873 / 0.8373 / **0.6822**, no cap failure — **−1.66e-4 in the
graded frame** against the receipted frontier tree's own official 0.790412.

## 3. A same-scoring candidate was measured and *refused* on transfer class

The triple `n`-ceiling 45 000 + 2G + 6 sweeps measured **0.790238** — statistically the same score as
this submission (−1.4e-5) — but its margin comes from the 45 000 ceiling, the band device that
transfers at 0.38, and its cost is the window DP's O(n²/64) per-call setup: it loads
`nd_netgen-3000-1-1-b-b-ns_7` (n = 33 155) from 0.51 s to 1.34 s (+0.83 s, and to 1.533 s with the
sweeps) for a −0.16 % gain on that row. Loading a 4x-under-cap row to near the cap for a
low-transfer device is exactly the trade this lane has lost submissions to, so the ceiling stays at
25 000 (`0237-triple-45k-2G-s6-4cpu.log`).

## 4. What was measured and killed this iteration (not shipped)

- **The `n` ceiling above 25 000 is spent.** `SSI_MAX_N` 25 000 / 30 000 / 45 000 → 0.790679 /
  0.790657 / 0.790610: −6.9e-5 for four movers at the O(n²/64) setup cost above.
- **The ladder draw above its gate buys nothing.** `SSI_TERM_FULL_N` 25 000 / 45 000 on the 45k tree:
  0.790611 / 0.790611 — zero value — while the worst row climbs 1.465 → 1.519 → 1.628 s. The
  exchange band's responsiveness is specific to exact window search, not to added search generally.
- **The gated-out big-row families are worse, not better.** `probe_large` on the five largest rows:
  AMF-5 / AMF-ND / METIS all land above the shipped ratio (`gabriel10` 0.9285 vs 1.0275 / 1.0275 /
  4.4925; `cont6-qq` 0.6962 vs 0.9145), so the remaining large-row `n` caps are not value levers.
- **The class gate's other admission axis is empty.** The `nnz <= 200_000` gate has a gap against the
  dense/hub rule (`nnz > 16n || max_deg > n/2`); on the dev corpus it contains exactly two rows
  (`gams05`, `nuclear104`).

## 5. A FAILED receipt is not a device kill on this board

Three prior submissions that raised this same allowance to 1G failed remotely while three before them
(512M) promoted, which read as a 3-for-3 device kill. It is refuted by the board: `52c744da` — the
same 1G + 5 sweeps **plus** `rgreedy::MAX_N` 12 000 → 25 000, i.e. strictly *more* work at this site —
**promoted at 0.841011 (−3.55e-4)**. `MAX_N` can only change rows with n > 12 000, so any hidden row
that could have killed `74f19b95` with n ≤ 12 000 runs bit-identically in both trees and the verdicts
still differ: the FAIL set is a per-row cap *lottery* on near-cap hidden rows, whose probability rises
with added work on those rows. That is why this submission is built to add its work to rows that sit
0.6–0.8 s under the cap and to leave every other row's time unchanged.

## 6. Mechanics, safety, determinism

The change is two deterministic `#[cfg(not(test))]` constants read by the existing exact window
search: a work counter bound (`TripleWork::remaining`) and its sweep limit. No allocation is added,
no clock/environment/filesystem/network is read on the shipped path (test seams such as
`SSI_EXCHANGE_LEDGER`, `SSI_EXCHANGE_SWEEPS`, `SSI_MAX_N` are `#[cfg(test)]`-only and never reach the
graded worker), `order()` remains a pure function of the `Pattern` returning a bijection (validated by
the harness on all 300 rows), and the parallel-order equivalence audit (0 divergences over 300 dev +
74 structural rows) is untouched. Everything runs inside the repository's own sandbox.

**Expected effect:** dev −1.66e-4 in the graded frame at a measured transfer of ~1.0 for this device
class, i.e. an expected hidden move of order −1.5e-4.
