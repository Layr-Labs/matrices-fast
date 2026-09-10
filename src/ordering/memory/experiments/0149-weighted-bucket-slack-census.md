# 0149 — Where the slack is in the buckets that pay (and where it is not)

## Census (`probe_mid_high_tie_census`, 153 rows with n >= 1000)

`1k_10k` — geomean 0.839747, 108 rows: **16 exact ties**, 12 near-ties, 70 rows
above the bucket geomean.
`gt_10k` — geomean 0.685397, 45 rows: **5 exact ties**, 0 near-ties, 29 above.

The tie mass is concentrated in facility-location and knapsack families:
`squfl*` (7 rows in `1k_10k`, 1 in `gt_10k`), `emfl*` (1 + 2), `knp5-4*` (2),
plus `chain400`, `camshape400`, `polygon75`, `hydroenergy1`, `kissing2`,
`watercontamination0303r`, `supplychainr1_053050`, `pooling_foulds5pq`.
21 rows in weighted buckets, worth roughly 17 bip if they could be moved 5%.

## Portfolio oracle (`probe_weighted_tie_oracle`, all 21)
0147 already proved swap-neighbourhood search is useless on ties, so this arm is
structural: **110 orderings per row** (11 metric variants x 5 dense_alphas x
aggressive on/off), plus `minfill_order`, plus four rounds of a deep subtree
chain on the best of them at `max_blocks = 64`, `budget = 16M`, `max_s = 1200`,
`streams = 4` — 0.06 to 7.08 s per row against a 2 s cap for the whole `order()`.

`ORACLE2_ROWS = 21`, `ORACLE2_ROWS_MOVED = 0`. Not one row moved, and on every
single row the winning arm was `ship` — no variant, no alpha, no min-fill, no
subtree round produced anything below AMD.

## Verdict
Ties are not slack anywhere in this corpus. Across 0147 and 0149 the entire tie
set — 54 rows in `lt_1k`, 21 in the weighted buckets — has now been attacked
from both directions, local and structural, at 3-10x the cap, with zero movers.
On these graphs AMD is at or indistinguishable from the reachable optimum, and
every future session should treat a 1.000 row as *done*, not as an opportunity.
This retires the standing "attack the ties" instinct that produced 0089, 0091,
0092, 0147 and now 0149.

## Where the remaining value actually is
The weak non-tie rows, and they cluster: the six largest graphs in `gt_10k` sit
between 0.93 and 0.98 while the bucket averages 0.685 —
`acopf_case9241pegase_qcqp` (n=313068) 0.9737 at 0.900 s,
`unitcommit_200_100_1_mod_8` (n=146830) 0.9764 at 0.572 s,
`transswitch2383wpr` (n=59853) 0.9785 at 0.683 s,
`faclay75` (n=272878) 0.9405 at 0.825 s,
`gabriel10` (n=244056) 0.9285 at 0.983 s,
`transswitch2736spr` (n=69651) 0.9179 at 0.556 s.

Every one is above `SUBTREE_CHAIN_MAX_N = 45_000`, so the chain is switched off
there by design — 0463a measured it moving the ratio by < 0.01 while costing
0.26-0.48 s. But that measurement was of the chain's *existing* geometry:
ranked blocks capped at `max_s <= 1200`, which on a 300k-vertex elimination tree
covers a vanishing fraction of the objective, exactly as the code comment says.
The untested question is not the gate, it is the geometry — whether blocks
scaled to the graph (`max_s` in the thousands to tens of thousands, few blocks,
ranked by contribution) do something on rows where 1200-vertex blocks did not.
These rows also carry 1.0-1.4 s of headroom each on this box.
