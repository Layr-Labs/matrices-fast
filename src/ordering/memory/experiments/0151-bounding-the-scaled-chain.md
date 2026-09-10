# 0151 — Bounding 0150 after it failed hidden validation

## What was wrong with 0150 as submitted
`budget` in `SubCfg` is **per block**. At `max_blocks = 16`, `budget = 32M` and
six unconditional rounds, one row could request ~3G operations, and the cascade
paid for every round even when the row never improved. Dev never showed that
because the four non-responding rows above the cap are all outside the gate —
but the hidden corpus has rows in the 45k-150k band we have never seen, and a
failed submission (no score at all) is what a per-matrix timeout looks like here.

## The fix: break on the first non-improving round
`probe_huge_row_bounded`, five variants, all breaking on the first round that
does not strictly improve:

| variant | board bip | added time (in-gate) | worst row |
|---|---:|---:|---:|
| A r3 / 8M / 8 blocks | 0.33 | 0.34 s | 0.16 s |
| B r3 / 32M / 16 blocks | **0.64** | 0.79 s | **0.29 s** |
| C r6 / 8M / 8 blocks | 0.33 | 0.36 s | 0.19 s |
| D r4 / 16M / 16 blocks | 0.59 | 0.74 s | 0.28 s |
| E r6 / 32M / 16 blocks | 0.66 | 1.35 s | 0.50 s |

Non-responding rows now cost **0.03-0.10 s** instead of 0.17-0.67 s — the break
is worth far more than any budget reduction, because the danger was never the
responders. Dropping the budget to 8M (A, C) halves the cost but loses half the
gain; extending to six rounds (E) buys 0.02 bip for double the time. **B** is
the pick: three rounds, 32M, 16 blocks, break on stall.

Retained per-row: `unitcommit_200_100_1_mod_8` 0.958% of its original 0.960%,
`transswitch2736spr` 0.064% of 0.576%, `transswitch2383wpr` 0.029% of 0.177%.
The transswitch rows lose most of their gain because their improving rounds were
not consecutive — the break gives up exactly that pattern, which is the price of
bounding the downside.

## Full dev

| | tip | 0150 (unbounded) | 0151 (bounded, shipped) |
|---|---:|---:|---:|
| score | 0.792300 | 0.792249 | **0.792257** |
| gt_10k | 0.685397 | 0.685270 | **0.685289** |
| fill | 0.924373 | 0.924330 | 0.924338 |

0.43 bip retained of the original 0.51, with the worst in-gate added cost down
from 0.52 s to 0.29 s and the unbounded tail removed entirely. `lt_1k` and
`1k_10k` remain bit-identical.
