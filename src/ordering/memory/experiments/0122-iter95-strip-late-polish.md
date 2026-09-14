# 0122 — iter95 strip late polish (tip has none)

## Diagnosis
Five reclaim FAILs including prefer≤1.00 `3a6ef2d`. Tip `c6b0311` has **no** late polish block; our iter74–91 late polish was additive wall-clock on hundreds of rows — likely hidden 2s / kill even when local worst is faclay-gated.

## Leap
**Delete** entire late polish block (streams/descent/simplicial). Keep dual+widen+chain core-exact + faclay nnz≥1.2M skips (AMF/robust/PEO) + rest of densify/PEO_ALT stack.

## Target
≥15 movers vs tip; worst ≤1.00; real overtake shot without late-polish poison.

## Result
(pending)
