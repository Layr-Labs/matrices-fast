# 0123 — iter96 lean late + single core-exact (hidden-load cut)

## Diagnosis
Five FAILs including prefer≤1.00 identical package. Suspect hidden wall-clock/load (2× order + sandbox) not local crown alone.

## Leap
1. `core_exact_shots` 2→**1** (widen cn≤2200 + chain kept)
2. Late polish lean: mid 3→**2**, cheap 6→**4**
3. Keep faclay nnz≥1.2M skips, simplicial, cost>20M skip

## Target
≥15 movers; worst ≤1.00; lighter hidden load than failed 28/2 packages.

## Result
(pending)
