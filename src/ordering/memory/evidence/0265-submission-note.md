# The allowance rung the remote record has never seen — and the first device on this lane that is provably value-free

**Model:** deepseek-v4-flash

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**, its tree = 2 GiB allowance +
`MAX_N` 25 000). `minScoreImprovementBips = 1` ≈ 8e-5 absolute at this score level.

This tree carries **one constant change and one new device**:

1. `PRODUCTION_EXCHANGE_LEDGER` `2_147_483_648 -> 3_221_225_472` (2 GiB -> 3 GiB). Three rungs of
   this ladder are measured on dev (2 GiB 0.790325, **3 GiB 0.790243**, 4 GiB 0.790175); only the
   2 GiB and 4 GiB rungs have ever been submitted, and every 4 GiB submission was cap-killed
   remotely. This is the rung in between, **never submitted**, and its value above the promoted
   tree is −1.70 bip on dev.
2. The **anchor gate**: the exact exchange family now runs only on rows where some other family has
   already beaten the AMD anchor (`best_flops < amd_flops` at `mod.rs:5469` and `:5558`).

## 1. Why this is worth a slot: the value of the 3 GiB step is three rows, measured

Diffing the two official per-matrix tables (production frame, same corpus, 300/300 OK) of the
2 GiB arm and the 3 GiB arm: the whole step is **crudeoil_lee4_10** 0.6150→0.6110,
**crudeoil_lee4_09** 0.6170→0.6140 and **mpbp_48** 0.4750→0.4740 — nothing else on the corpus
moves. −8.2e-5 dev against the 2 GiB arm, −1.70e-4 against the promoted tree's own dev point.

## 2. Why it is safe to spend the slot: the hidden frame's slowdown is now bracketed

The 2 s cap is charged on **wall clock, per matrix, on a child process** (`src/watchdog.rs:85`
starts `Instant::now()`, `:111` kills on `start.elapsed() > cfg.time_cap`), and the grader runs
every matrix **twice** in fresh processes. No number in this lane's record had ever been taken in
that frame: the probe frame is a *different binary* (every `SSI_*` seam and timer here is
`#[cfg(test)]`-gated — `mod.rs:5441`, `:5466`, `window_dp.rs:100` — and is therefore compiled out
of the graded worker), and the official run prints `(capped)`.

I built the missing instrument (one run of `target/release/ssi-candidate-worker` per dev row,
`taskset -c 0-3`, `/usr/bin/time`) and measured the ladder in the graded frame:

| ledger | corpus wall | worst row |
|---|---|---|
| family off | 114.4 s | 0.840 s |
| 2 GiB (promoted-day tree) | 123.2 s | **1.010 s** `procurement1large` |
| **3 GiB (this tree)** | 124.2 s | **1.100 s** `crudeoil_lee4_09` |
| 4 GiB (every submission cap-killed) | 127.2 s | **1.250 s** `methanol400` |

The only same-day pass/fail pair on the board is 2 GiB + 25 000 (promoted) vs 4 GiB + 25 000
(cap-killed), so with those two local maxima the hidden frame's slowdown factor is bracketed at
**1.60 < f ≤ 1.98**. On that model this rung lands at 1.76–2.18 s — the honest risk — which is
exactly what the device below is for. The probe frame, by the way, prices this tree's worst row at
1.269 s (`crudeoil_lee4_10`), a row that costs 0.74 s in the graded frame; its median per-row
inflations is 1.18x, so every cap decision taken on it was taken in the wrong frame.

## 3. The anchor gate: free margin, proved twice

`amd_flops` is captured the instant the AMD seed is scored (`mod.rs:1768`) and every adoption in
the pipeline is a strict exact decrease, so on a row still **at** the anchor no family has adopted
anything: skipping the exchange there removes only work that could not have been installed. Two
independent measurements:

* with the family compiled out of the production frame, **not one of the 300 dev rows ends at the
  anchor while ending below it with the family on** — the family is never any row's first improver;
* the official A/B, gate on vs off at fixed 3 GiB: **0 of 300 per-matrix rows differ**, score and
  tiebreak identical to all six printed digits (`results.tsv:1789307153` vs `:1789305238`).

What it returns (same census, per row): on the 2 GiB frame the corpus wall drops 123.2 → 121.1 s,
with `emfl100_5_5` −0.28, `chain400` −0.24, `supplychainr1_053050` −0.19, `chain200` −0.15,
`squfl020-150` −0.12, `squfl015-080persp` −0.12, `emfl100_3_3` −0.11, `hydroenergy1` −0.11 …
Every row that gets faster is one that ties AMD (ratio 1.0000, 81 such rows); **no row that gets
slower is anchor-tied**, i.e. the gate provably did not execute differently on them. The spend is
therefore the family's cost on anchor-tied rows — 0.90 s of the family's 8.8 s — and it is free.
On a hidden row that ties AMD (the shape the harness's own failure text names: *"nnz/n≈53 — if it
is dense, the cost is in order() itself; gate expensive paths by BOTH n and nnz"*) that is
0.09–0.28 s of the 2.0 s budget handed back with zero score risk.

## 4. Evidence

`src/ordering/memory/evidence/0265-graded-frame-census.txt` (instrument + ladder + family price),
`0265-anchor-gate-ab.txt` (the gate's construction and both receipts),
`0265-official-run-gate3G.log` (this tree: 300/300 OK, no FAIL, **0.790243 / 0.923082**,
buckets 0.8873 / 0.8375 / 0.6820), `0265-official-run-ledger0.log` (family-off arm, 0.7918).
Per-row censuses: `.scratch/widthcensus/census{4cpu,led0,L3G,L4G,gate,gate3G}.tsv`.
