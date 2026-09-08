# 0113 — iter86 mid-cost late polish densify

## Context
iter84 `71b8941` validating. iter85 chained LNS only +syn micro. Need broader movers.

## Leap
Mid-cost late polish (`8M < n·nnz ≤ 20M`, n<3k nnz≤12k):
- streams 1 → **3** lean (12M/10M/8M, distinct seeds)
- pair descent rounds 1 → **2**, budget ×3k→×4k (cap 4M)
cost>20M still fully skipped (killers). Cheap 6-stream unchanged.

## Why
Many tip-tied high-ratio rows sit in mid-cost (ndcc12, kall_circlespolygons, multiplants, syn30…). Single mid stream left lottery thin; 3 streams buy breadth without danger reopen.

## Result
SCORE **0.805478** (−7.65 bip vs tip) / WORST **1.054s** / movers **28/2** vs tip.
vs iter85: **+3** (netmod_kar1, rsyn0810m02hfsg, rsyn0820m02m) / 0 worse — fixed prior rsyn0820m02m regression.
Prefer ≤1.05 narrowly missed; hard ≤1.10 met. Ready to submit after `71b8941` settles.
