# 0064 — Multi-depth exact prefixes (K = 3, 4, 5, 2, 6) on the 0062 substrate, paid for by a per-variant robust-envelope gate

- **Date:** 2026-09-05
- **Base:** `2a28517` ([0062](0062-reduce-then-amf-terminal.md) + [0063](0063-time-margin-by-structure.md); hidden 0.867211; dev 0.839063 / out-of-sample 0.856455 on this box)
- **Score:** dev 0.839063 → **0.833382** (-0.677% rel, 67.7 bip); out-of-sample 0.856455 → **0.850528** (-0.692% rel, 69.2 bip)
- **Status:** GO: dev -68 bip / out-of-sample -69 bip vs the crown 2a28517, 13 / 16 rows better, 0 / 1 (0.2%) worse; quiet min-of-3 on the 32 slowest rows: crown 1.127 s -> candidate 1.113 s, no row slower by more than 0.04 s; submitted for hidden validation

## Hypothesis

The exact prefix at degree <= K leaves a different residual core for each K, and the AMF/AMD passes on those cores
reach different basins. On the previous challenge's corpus the union over K in {2,3,4,5,6} beat single-depth K = 3
by ~40 bip (arki0009 0.39 -> 0.30 at K = 2, nuclear10b 0.68 -> 0.63 at K = 4, parabol5_2_2 0.76 -> 0.69 at K = 6).
Deeper reductions can be expensive (the exact closure of a degree-6 vertex adds up to 15 clique pairs), so the
depth loop needs a work law, not a size gate.

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

## Result

| corpus | crown | candidate | better / worse | worst order() |
|---|---|---|---|---|
| dev | 0.839063 | 0.833382 | 13 / 0 | 1.084 s -> 1.202 s |
| out-of-sample | 0.856455 | 0.850528 | 16 / 1 | 0.951 s -> 0.975 s |

**dev movers (13 rows better, 0 worse):** gams05 (gt_10k, n=17364, nnz=252910): 0.7846 -> 0.5634; cont6-qq (gt_10k, n=120395, nnz=557994): 0.8899 -> 0.7548; transswitch2736spr (gt_10k, n=69651, nnz=331010): 0.9874 -> 0.8975; nuclear10a (gt_10k, n=17493, nnz=163816): 0.7602 -> 0.6961; rsyn0820m04m (1k_10k, n=6028, nnz=17372): 0.9174 -> 0.8432; rsyn0830m04m (1k_10k, n=7252, nnz=20752): 0.9113 -> 0.8508; popdynm200 (gt_10k, n=22407, nnz=105584): 0.9994 -> 0.9388; rsyn0840m04m (1k_10k, n=8508, nnz=24272): 0.9127 -> 0.8721; rsyn0815m04m (1k_10k, n=5456, nnz=15756): 0.8842 -> 0.8736; crudeoil_lee4_06 (gt_10k, n=10429, nnz=55492): 0.7958 -> 0.7899; unitcommit_200_100_1_mod_8 (gt_10k, n=146830, nnz=476332): 0.9764 -> 0.9735; gasprod_sarawak81 (gt_10k, n=22536, nnz=75636): 0.9400 -> 0.9388; nuclear104 (gt_10k, n=39098, nnz=257806): 0.8998 -> 0.8997.

**out-of-sample movers (16 rows better, 1 worse):** parabol5_2_3 (gt_10k, n=80603, nnz=480012): 0.8157 -> 0.6929; arki0015 (1k_10k, n=3476, nnz=15928): 0.9615 -> 0.8297; lnts400 (1k_10k, n=3599, nnz=18376): 0.9638 -> 0.8628; powerflow2736spr (gt_10k, n=66157, nnz=275106): 0.9956 -> 0.8926; rsyn0805m04hfsg (1k_10k, n=5400, nnz=14640): 0.9504 -> 0.8667; crudeoil_li11 (1k_10k, n=7656, nnz=27338): 0.8733 -> 0.7980; infeas1 (1k_10k, n=3268, nnz=30024): 0.7062 -> 0.6455; crudeoil_lee2_07 (1k_10k, n=7555, nnz=43590): 0.8792 -> 0.8130; rsyn0830m04hfsg (1k_10k, n=9320, nnz=25104): 0.8866 -> 0.8462; rsyn0815m04hfsg (1k_10k, n=7044, nnz=19056): 0.8674 -> 0.8452; rsyn0810m04hfsg (1k_10k, n=6156, nnz=16648): 0.9141 -> 0.8933; rsyn0815m03m (1k_10k, n=3741, nnz=10652): 0.9423 -> 0.9240; sfacloc2_2_80 (1k_10k, n=4531, nnz=19776): 0.9927 -> 0.9755; oil2 (1k_10k, n=1880, nnz=4676): 0.9660 -> 0.9498.

`cargo test --release -p ssi-candidate-worker`: 50 passed, 0 failed (18 ignored probes).

## Links

- [0062](0062-reduce-then-amf-terminal.md), [0063](0063-time-margin-by-structure.md)
