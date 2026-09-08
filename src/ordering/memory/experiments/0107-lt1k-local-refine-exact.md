# 0107 — lt_1k local refine + exact densify (iter62)

## Context
iter61 METIS densify n<3k: **0.805933 (−3.10 bip), 5/1 movers, WORST 1.158s** — fails ≥15 movers and ≤1.10s bars. No submit.

## Leap (not METIS/core-ND inch)
1. Widen `SmallScore` bitset 5→16 words (n≤1024) and run paired-swap/plateau refine on **n≤1000 nnz≤8000** (tip: n≤300).
2. Densify lt_1k exact-search lottery (+4/+3 streams). Cannot move corpus worst (lives at n≫1k).

Keep iter61 METIS densify + EXTRA densify; no medium+2; no core-ND.

## Result
(pending)
