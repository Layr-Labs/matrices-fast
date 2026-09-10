# 0147 — The `lt_1k` tie set: census, then an oracle 3x over the cap

## Why
`lt_1k` is the weakest bucket on the synced tip (0.887390 against 0.839747 and
0.685397), and it is the cheapest one: its slowest row, `multiplants_mtg1b`,
runs 0.762 s of a 2.0 s cap on this box. If slack exists anywhere it should be
visible here.

## Census (`probe_lt1k_tie_census`, 147 rows)
- `LT1K_TIES = 54` rows whose shipped ordering only ties raw AMD
- `LT1K_TIES_N_LE_20_PROVABLY_OPTIMAL = 10` — settled by exact subset DP; AMD
  and the portfolio are both at the minimum of `flops_of`, so those 1.000s are
  not lost value
- `LT1K_TIES_UNEXPLORED = 44`, holding 10.08 s of `order()` time between them
- `LT1K_NEAR_TIES = 12` more rows within 2% of AMD

Moving the 44 unexplored ties to an average 0.97 would be worth roughly 27 basis
points on the board. That is the whole prize in this bucket, so it is worth one
decisive test rather than a series of tunings.

## Oracle (`probe_lt1k_tie_oracle`, 15 of the 44)
Perturb + `plateau_refine` + `paired_swap_refine` restarted from the shipped
ordering, **6 s per row** — 3x the entire per-matrix cap, 8-20x what the row
actually spends. 328 to 1700 local-search iterations per row depending on size.

- `ORACLE_ROWS = 15`
- `ORACLE_ROWS_MOVED = 0`
- accepted improvements: **0 on every row**, not one iteration in ~13,000

## Verdict
The tie set is locked. Not "converged at the current budget" — the local search
never found a single improving neighbour, at any perturbation strength, with
three times the cap to spend. These orderings sit at a hard local optimum of the
swap neighbourhood, and 10 of the 54 are provably global.

`lt_1k` is therefore not where the remaining points are, and no amount of extra
budget, restarts or tickets aimed at it will produce any. Anything that moves
this bucket has to be a structurally different ordering — a different objective
producing a different elimination tree — not more search around the current one.

The open lead worth the next cycle is on the other side of the corpus: 0145's
own follow-up, running the subtree stage on an accepted independent-set lift,
gated to rows whose `order()` is under ~0.6 s. That is `gt_10k` (weight 0.40,
0.685397) where the tip is actively moving, and it is a flops change with a
measurable, gateable time cost.
