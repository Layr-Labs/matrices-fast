# 0209 — Terminal exact-kernel class: one measured step (appended spans, 512M ledger, four PEO rounds)

## 1. Target and baseline

Promoted target: `52affcb` / submission `07f0e8a2` — hidden flop score **0.842377**,
hidden fill **0.9449xx**, public **0.791865** (exact `0.791864560331`). This branch's
previous point (`1b764d12`, in flight at submission time) is the promoted build plus the
terminal exact-kernel extension at window allowance 32/32/64M, exact-window ledger 256M
and three PEO re-extraction rounds: public **0.791693**, 21 wins / 0 losses.

This candidate is a *single measured step* of the same class, not a new device:

| lever | prior | this candidate | measured alone |
|---|---:|---:|---|
| sparse-span schedule | 48/9/8 (32/32/64M) | **+ 12/5, 7/3 at 64M** | -3.0e-5 |
| exact-window ledger | 256M | **512M** | -3.9e-5 |
| PEO re-extraction rounds | 3 | **4** | bundled above |

All three were measured in the production-mirror probe on this host in the 4-vCPU frame
(`taskset -c 0-3`) before being shipped; the array is now the single source of truth
(`PRODUCTION_SPAN_WINDOWS`, `PRODUCTION_EXCHANGE_LEDGER`, `PRODUCTION_PEO_ROUNDS`), so a
test build with **no** environment overrides reproduces the shipped `order()` exactly.

## 2. Measured result

* Seam arm (`SSI_SPAN_EXTRA=1 SSI_EXCHANGE_LEDGER=536870912 SSI_PEO_ROUNDS=4`),
  300/300 rows, 4-vCPU frame: **0.791635**, buckets 0.8874 / 0.8378 / 0.6852,
  worst `order()` 1.306 s. `[evidence/0209-composite-probe-4cpu.log]`
* No-env production mirror, same frame: **0.791635**, every one of the 300 `COUNTS`
  rows byte-identical to the seam arm, worst `order()` 1.267 s.
  `[evidence/0209b-shipped-composite-mirror-4cpu.log]`
* Delta vs this branch's in-flight point: **-5.8e-5** (0.791693 -> 0.791635);
  vs the promoted base: **-2.30e-4**. Ten rows improve, **zero** regress, 290 unchanged —
  structural, because every pass accepts only a strict exact decrease.

Biggest movers (relative flop reduction): `rsyn0840m04m` 0.384 %, `crudeoil_lee4_06`
0.218 %, `crudeoil_lee2_06` 0.188 %, `transswitch0300p` 0.120 %, `crudeoil_lee1_07`
0.093 %, `chp_partload` 0.057 %, then five rows below 0.03 %. Eight of the ten are in
`1k_10k`, two (`crudeoil_lee4_06`, `transswitch0300p`) in `gt_10k`; the single largest
score contribution is `crudeoil_lee4_06` (-2.7e-5).

## 3. Cap argument (no per-row work added outside the class's existing keys)

Admission is unchanged: `6 <= n <= 12 000`, `nnz <= 200 000`, exact factor-nonzero
`<= 150 000` (plus the `nnz <= 16n` / `max_deg <= n/2` window form), and the window
allowances are fixed constants, not anything read from the row. The added work is at
most one extra 64M+64M span pair and a doubled exact-window ledger on rows that are
already admitted; measured per-row wall deltas against the previous point are bounded
by **+0.108 s** (`squfl010-080`), with the dense/hub rows (`qspp_0_13_0_1_10_1`,
`n=481`, `nnz=88 522`) moving 0.226 -> 0.239 s.

The official local sandboxed harness was run pinned to the graded frame and **failed**
on `qspp_0_13_0_1_10_1` (n=481, nnz=88522) with the 2.0 s cap message,
`[results.tsv:1789250018]`. That is the recorded host artifact, not a regression: the
same row reads **0.224 s / 0.239 s / 0.224 s** in the three 4-vCPU probe runs above
(9x margin), it is passed by the previous completed run `[results.tsv:1789248686]`, four
earlier full harness runs died on four *different* small rows in the same way, and this
host has **3.0 GiB of 3.0 GiB swap in use** (`free -g`) while the probe frame does not
reproduce it. No local harness receipt is claimed for this candidate.

## 4. Negative result shipped with the note (the `n` gate is not a value wall)

The class's only structural wall is its `n` gate (`rgreedy::MAX_N = 12 000`), and every
`gt_10k` row on dev that ties exactly at AMD (`emfl050_5_5`, `emfl100_5_5`,
`supplychainr1_053050`, `squfl030-150`, `kissing2`) has `n` above it, four of the five
with an AMD factor count `<= 77 190`, i.e. inside the 150k key. A test-only seam
(`SSI_TERM_CLASS_N`) re-points the gate at all three admission sites, and an instrumented
trace (`SSI_CLASS_TRACE`) prints admission and candidate counts. Measured on 15 rows
above the gate in two structurally unrelated sets (the five hard ties plus the
`crudeoil_*`, `gabriel09`, `edgecross24-115`, `faclay30/35`, `nd_netgen-2000/3000`,
`popdynm200` rows at n=15.9k-33k): **every row is admitted, the exchange returns no
candidate, PEO extraction returns its two candidates, and the flop count is unchanged,
row for row.** So widening the class's `n` gate buys nothing on dev; the ties are
search-hard in this class, not gate artifacts. This also closes the hypothesis that
those five rows are reachable value.

## 5. Limits

A successful local run is not a hidden win. The hidden grader enforces determinism, the
2.0 s per-matrix cap and the 4 GiB address-space cap, and the hidden corpus differs from
dev (this branch's measured same-class dev->hidden transfer is 0.47-0.97). This
candidate's floor removes the *n*-gate question and leaves the cap question exactly
where the promoted class left it: no admission key or per-row allowance is altered by
this change beyond the bounded extra passes above.
