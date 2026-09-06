# 0064 — Multi-depth exact prefixes (K = 3, 4, 5, 2, 6) on the 0062 substrate, paid for by a per-variant robust-envelope gate

- **Date:** 2026-09-05
- **Base:** `2a28517` ([0062](0062-reduce-then-amf-terminal.md) + [0063](0063-time-margin-by-structure.md); hidden 0.867211; dev 0.839063 / out-of-sample 0.856455 on this box)
- **Score:** dev 0.839063 → **0.833148** (-0.705% rel, 70.5 bip); out-of-sample 0.856455 → **0.854335** (-0.248% rel, 24.8 bip)
- **Status:** SUBMITTED as S10b = S9b + the ported 608a653f completion watcher at op budget 5M (0067): pinned 2-core ship check dev -70 bip / out-of-sample -25 bip vs the crown 2a28517, 66 / 86 rows better, 4 / 7 worse by <= 1%; pinned min-of-3 on the 32 slowest rows: crown 1.199 s -> candidate 1.076 s; whole-corpus min-of-3 worst 1.290 -> 0.942 s dev, 1.115 -> 1.067 s out-of-sample, 0 rows of the >= 0.8 s class over the graded envelope; flops identical across 3 runs; 56 tests

## Hypothesis

The exact prefix at degree <= K leaves a different residual core for each K, and the AMF/AMD passes on those cores
reach different basins. On the previous challenge's corpus the union over K in {2,3,4,5,6} beat single-depth K = 3
by ~40 bip (arki0009 0.39 -> 0.30 at K = 2, nuclear10b 0.68 -> 0.63 at K = 4, parabol5_2_2 0.76 -> 0.69 at K = 6).
Deeper reductions can be expensive (the exact closure of a degree-6 vertex adds up to 15 clique pairs), so the
depth loop needs a work law, not a size gate.

## Grader result of the first shipped shape (submission `fce9a9da`, commit 156798a): KILLED at the 2 s cap

The run died 3 min 7 s into a corpus the crown finishes in 7 min 25 s - a mid-corpus hidden row, not the early
killer of the morning. The shape that shipped ran the four extra reductions on four scoped threads and every core's
five passes on five more; on this 16-core bench that is nearly free, and the pinned 2-core quiet timing on the 32
slowest known rows still showed only +0.02 s (crown 1.231 -> 1.249 s). The grader has 2-4 vCPUs and the hidden row
that died is in a class our corpora do not make slow. **Local wall-clock of multi-threaded additions is not a grader
proxy.** Redesign: extra depths bounded and SEQUENTIAL (reduce-work budget in nnz-units, an extra-core ledger, three
passes per extra core run one after another), so the cost is the same on 2 vCPUs as on 16, and measured under
`taskset -c 0-1`.

## What changed

`mod.rs`: `REDUCE_NESTED_DEPTHS = [4, 5, 6]`, `REDUCE_NESTED_MAX_NNZ = 600_000`, `REDUCE_SHALLOW_MAX_NNZ = 150_000`,
`REDUCE_PAIR_BUDGET = 4_000_000`, `REDUCE_TOTAL_CORE_NNZ = 1_500_000`. The terminal block keeps the shipped K = 3 pass
exactly, then NESTS: the K = 3 core (a `ScoringPattern` built from `core_col_ptr / core_row_idx`) is reduced with
`reduce_checked` at degree 4, that core at 5, then 6; `order_chain` orders the innermost core five ways, ranks on
it, and composes the argmin back through every `splice` in the chain. A shallow K = 2 reduction from scratch runs
on cheap graphs. Gates: eliminate >= 10% of the parent core, core caps, the pair budget (fails closed), the ledger.

**Two rejected shapes on the way.** (a) Re-reducing the FULL graph at every depth (order 3,2,4,5,6; pair budget
20M; ledger 3M): dev 0.830591 / out-of-sample 0.848464, 19 / 25 rows better, none worse - but +0.25-0.5 s on
giants that never win (gabriel10, acopf_*, faclay75, kissing2), +0.43 s on chimera_rfr-02 (n = 2032: the degree-5/6
closure burned the pair budget), worst 1.38 s. (b) The same with an nnz <= 600k gate, pair budget 4M, ledger 1.5M,
order 3,4,5,2,6: dev 0.830591 / out-of-sample 0.848766, still +0.56 s on unitcommit_200_100_1_mod_8 (476k nnz,
four full-graph reductions of 0.14 s each for a 9.8k-node core); quiet min-of-3 on the 32 slowest rows: crown
1.168 s -> 1.338 s. The per-depth census says the wins live at K = 4 and 5 (gams05 K = 5 alpha 0.5, nuclear10a
K = 4, cont6-qq K = 5), and the cores are small - so reduce the core, not the graph.

**Nested variant (measured, rejected):** reducing the K = 3 core again at 4 -> 5 -> 6 (composing the splices; K = 2
from scratch on nnz <= 150k) is cheap but reaches fewer basins: dev 0.835483 / out-of-sample 0.853127 (12 / 16 rows
better) versus 0.830591 / 0.848466 for the from-scratch depths, and still read a 1.44 s worst (gams05 +0.36 s,
transswitch2383wpr +0.55 s). The from-scratch depths are the lever; their cost is paid elsewhere:

**Robust AMD envelope, per-variant gate above 150k nnz.** On the 44 rows above 150k nnz gating any ONE of the five
robust variants (non-aggressive alpha 10 / 5 / 2, dense-disabled non-aggressive, dense-disabled aggressive) changes
no row - their wins are redundant - while gating all five (the rejected `ROBUST_MAX_NNZ = 150k`) loses parabol_p and
arki0005. Keeping exactly one (the non-aggressive alpha = 10 variant) above 150k and gating the other four recovers 0.3-0.5 s on gams05 /
nuclear104 / unitcommit-class rows.

## Shape shipped from this page (S-series, all measured pinned to two cores)

After the killed threaded shape, the extras were rebuilt bounded and sequential and gated by BAND: a small band
(nnz <= 60k, where a reduction and its three proxy-ranked passes cost a few ms) and a dense mid-large band
(nnz > 200k and nnz >= 6 n, where the robust-envelope gate frees 0.2-0.5 s); the band between them is the crown's
slowest class (crudeoil_lee4_10 120k, arki0013 160k) and stays exactly as the crown has it. The sparse-large class
(powerflow / transswitch / cont6, nnz/n ~ 4) is excluded: AMD is cheap there, the gate frees little, and the extras
were +0.2-0.4 s net. Two margin levers ride along: the per-variant robust-envelope gate (keep only non-aggressive
alpha 10 above 150k nnz; any single variant is redundant there, all five are not) and the harness round's
dense-set twin skip (skip a plain-AMD alpha pass whose dense-deferred count repeats one already run; bit-identical
on all 891 rows; -2.4 s over the 32 slowest dev rows). The gate is wrapped OUTSIDE the twin-skip ledger so a gated
pass is never recorded as "seen".

## Result

| corpus | crown | candidate | better / worse | worst order() |
|---|---|---|---|---|
| dev | 0.839063 | 0.833148 | 66 / 4 | 1.289 s -> 1.001 s |
| out-of-sample | 0.856455 | 0.854335 | 86 / 7 | 1.115 s -> 1.156 s |

**dev movers (66 rows better, 4 worse):** gams05 (gt_10k, n=17364, nnz=252910): 0.7846 -> 0.5634; mpbp_35 (gt_10k, n=11120, nnz=40790): 0.4227 -> 0.3713; mpbp_34 (gt_10k, n=11556, nnz=40860): 0.4301 -> 0.3818; chp_shorttermplan2d (gt_10k, n=16364, nnz=52108): 0.5607 -> 0.5187; chp_partload (1k_10k, n=5211, nnz=16740): 0.8126 -> 0.7739; procurement1large (gt_10k, n=14416, nnz=41068): 0.6493 -> 0.6202; pooling_adhya4pq (lt_1k, n=170, nnz=932): 0.7116 -> 0.6843; rsyn0805m03m (1k_10k, n=2868, nnz=8270): 0.9536 -> 0.9181; maxcsp-langford-3-11 (lt_1k, n=660, nnz=29646): 0.3256 -> 0.3135; pooling_sppa9pq (1k_10k, n=5030, nnz=120730): 0.6530 -> 0.6340; gasprod_sarawak81 (gt_10k, n=22536, nnz=75636): 0.9400 -> 0.9224; gabriel09 (gt_10k, n=21688, nnz=89702): 0.9858 -> 0.9677; rsyn0840m04m (1k_10k, n=8508, nnz=24272): 0.9127 -> 0.9026; tls6 (lt_1k, n=413, nnz=2860): 0.9574 -> 0.9480. Worse: rsyn0830m04m: 0.9113 -> 0.9118 (+0.05%); korcns: 0.9268 -> 0.9323 (+0.60%); powerflow0118p: 0.9681 -> 0.9682 (+0.01%); supplychainr1_022020: 0.9999 -> 0.9999 (+0.00%).

**out-of-sample movers (86 rows better, 7 worse):** arki0015 (1k_10k, n=3476, nnz=15928): 0.9615 -> 0.8242; rsyn0805m04hfsg (1k_10k, n=5400, nnz=14640): 0.9504 -> 0.8438; lnts400 (1k_10k, n=3599, nnz=18376): 0.9638 -> 0.8627; crudeoil_li11 (1k_10k, n=7656, nnz=27338): 0.8733 -> 0.7827; crudeoil_lee2_07 (1k_10k, n=7555, nnz=43590): 0.8792 -> 0.7931; pooling_foulds5stp (1k_10k, n=1830, nnz=12524): 0.9476 -> 0.8706; rsyn0810m04hfsg (1k_10k, n=6156, nnz=16648): 0.9141 -> 0.8561; edgecross20-040 (1k_10k, n=9503, nnz=38518): 0.7538 -> 0.7187; rsyn0830m04hfsg (1k_10k, n=9320, nnz=25104): 0.8866 -> 0.8495; mpbp_04 (1k_10k, n=4734, nnz=20800): 0.9162 -> 0.8793; gastrans582_cold13_95 (1k_10k, n=7895, nnz=22248): 0.9800 -> 0.9486; rsyn0810m03m (1k_10k, n=3273, nnz=9362): 0.9500 -> 0.9228; crudeoil_lee1_10 (1k_10k, n=5479, nnz=34920): 0.8895 -> 0.8691; gastrans040 (1k_10k, n=1160, nnz=3354): 0.9719 -> 0.9507. Worse: arki0005: 0.8009 -> 0.8025 (+0.20%); ex8_3_8: 0.6650 -> 0.6663 (+0.20%); gabriel05: 0.7159 -> 0.7162 (+0.04%); graphpart_2pm-0099-0999: 0.9761 -> 0.9857 (+0.99%); gsg_0001: 0.9195 -> 0.9249 (+0.60%); rsyn0815m03hfsg: 0.9051 -> 0.9054 (+0.04%); sporttournament36: 0.6912 -> 0.6912 (+0.00%).

`cargo test --release -p ssi-candidate-worker`: 56 passed, 0 failed (18 ignored probes). Pinned interleaved min-of-3 on the 32 slowest rows: crown worst 1.199 s -> candidate worst 1.076 s, no row slower by more than 0.04 s.

## Links

- [0062](0062-reduce-then-amf-terminal.md), [0063](0063-time-margin-by-structure.md)
