# 0097 — Fill gate on the completion descent; gdonninelli's sub-10k lotteries restored

**Date:** 2026-09-07. **Base:** `fcb74a7` (submission `e7f15a87`, hidden 0.851366; dev 0.807259 on the 2-vCPU pod, worst 1.634 s).
**Result:** dev **0.806788** (−5.8 bips), 12 rows better / 9 worse; worst same-box `order()` 1.601 s. Harness 300/300 rows OK (bijection + determinism gates, `order()` twice per row), score 0.8068 (lt_1k 0.8897 / 1k_10k 0.8455 / gt_10k 0.7156, fill tiebreak 0.9306), 4 min 38 s wall on the pod.

## Why a restore

`e7f15a87` was built on `7f5a20d` and validated after gdonninelli's `c0f8ab1`
(hidden 0.85157, page 0096-sub10k-lotteries) had been promoted; the graft of
my `src/ordering/` replaced theirs, so their block left the branch. It is put
back verbatim here (same seeds `40_000 + w·1000 + r`, same `n < 10_000` gate,
same 120k/nnz budget). Their numbers: dev −5.17 bips (all 1k_10k) on their
host, hidden −1.4 bips. On this tree (dev, pod): −3.85 dev bips, all in `1k_10k` (`rsyn0815m04m` 0.8522 → 0.7860, `chp_shorttermplan1a` 0.8276 → 0.8039, `mpbp_46` 0.9213 → 0.8985, `crudeoil_lee1_07` 0.7917 → 0.7732, `rsyn0840m04m` 0.8305 → 0.8158; `rsyn0830m04m` +0.70 and `wastewater05m1` +0.60 flip back — 11 rows better, 7 worse), `gt_10k` bit-identical as their gate promises, +7.5 s of corpus time all under `n < 10k` (the slowest such row, `arki0016`, at 1.37 s against the corpus worst of 1.54 s). Two of the fourteen pairs (`SqPure@10`, `DegDivNvSqrtWf@10`) duplicate families my previous submission added to the all-n loop with other seeds, so on this tree they are extra draws, not new families.

## The fill gate

The 0096 profile: on `crudeoil_lee4_10` (L = 695k nonzeros, 635k fill edges)
the 40M budget covers ~110k edges of the first scan, the descent is cut short
after removing a handful of edges, and the realized order wins 0.01 % for
~0.1 s on the corpus's slowest row. `MINL_MAX_LNNZ` 1.5M → 600k removes the
scan exactly where coverage is under ~15 %: on dev only that row and two other
fruitless partial scans (`pooling_sppc1pq`, `maxcsp-ehi-85-297-71`, −0.10 s
each). Kept: `crudeoil_lee4_09` (558k, −0.18 bips), `transswitch2736spr`
(536k, −0.57 bips). Cost: +0.18 bips (`arki0013` 0.5852 → 0.5862 and three
rows ≤ 0.02). A 400k gate drops transswitch2736spr and is rejected.

## Four more all-n families (shipped)

`DegPlusDegme@10`, `DegDivNvDegme@10`, `SqDiv@1`, `DegP075@1` in the all-n
relabelled loop: on the sub-10k-restored tree −2.2 bips, 3 wins / 1 loss
(crudeoil_lee4_09 −1.76, netmod_kar1 −0.42), +0.06 s on the slowest row
(1.541 → 1.601 s, still under the base's 1.634 s). Measured on the un-restored
tree they were 4 wins / 5 losses: dev cannot rank these lotteries; the loop's
hidden history (dukemawex: −1.0 dev → −9.5 hidden) can.

## Not shipped

- Relabel budget 240k: −3.5 bips on one row (crudeoil_lee4_06), +0.14 s near the cap.

## Movers vs the base

wins: `rsyn0815m04m` 0.8522 → 0.7860, `chp_shorttermplan1a` 0.8276 → 0.8039, `mpbp_46` 0.9213 → 0.8985, `crudeoil_lee1_07` 0.7917 → 0.7732, `crudeoil_lee4_09` 0.6951 → 0.6815, `rsyn0840m04m` 0.8305 → 0.8158, `netmod_kar1` 0.7969 → 0.7845, `lop97icx` 0.8262 → 0.8204, `rsyn0805m03m` 0.9044 → 0.9026, `rsyn0810m04m` 0.7865 → 0.7859, `rsyn0815m02hfsg` 0.9332 → 0.9328, `chimera_mgw-c8-439-onc8-002` 0.9028 → 0.9025, then 0 rows under 0.1 bip; losses (all in the lottery's own 1k_10k rows, best-of reshuffles under different draws): `wastewater05m1` 0.7274 → 0.7492, `rsyn0830m04m` 0.7920 → 0.8122, `arki0013` 0.5852 → 0.5862, `popdynm25` 0.7997 → 0.8002, `maxcsp-ehi-85-297-71` 0.8036 → 0.8041, `rsyn0810m02hfsg` 0.9516 → 0.9521, and 3 rows under 0.05 bip
