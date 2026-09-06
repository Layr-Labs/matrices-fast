# 0085 — Reduction and relabel losers as alternate chain seeds (negative)

Base: promoted tip `4d86414` (hidden 0.859573), local dev **0.826784**.

0084 shipped alternate-seed chains but drew seeds only from the `consider`
funnel, i.e. from raw portfolio orderings. The reduction/core-lift stages
(`order_core` at both reduction depths) and the independent relabel candidate
(`terminal_core_candidate`) build strong orderings late and drop them outright
when they fail to take the lead. Those are structurally different from the
leader and already paid for, so they were routed into the same seed list via a
`keep_seed` closure sharing the existing 4M-unit allowance.

Result: dev 0.826784 -> **0.826728**, 0.56 bip, entirely inside 1k_10k
(0.863292 -> 0.863104); lt_1k and gt_10k did not move at all. Raising
`PEO_ALT_SEEDS` from 4 to 6 with the richer source added 0.02 bip (0.826726)
and pushed the worst row from 1.3688 s to 1.4305 s (`crudeoil_lee4_10`).

Not submitted. 0084 converted 4.11 bip dev into 2.61 bip hidden, so 0.56 bip dev
is around 0.35 bip hidden against a bar that has rejected three rivals at
0.859820-0.859877. Reverted to a clean tip.

Read: the seed *source* is not the bottleneck; the chain's reachable set is.
Seeds from the same family of heuristics converge to nearby minimal
triangulations, and the only rows that moved were ones where the reduction path
disagrees with the portfolio leader structurally. Next lead should change what
the chain can reach — a perturbed seed (a random-restart or a locally shuffled
elimination prefix) rather than another already-built ordering — or target the
completion refinement rather than the ordering search.
