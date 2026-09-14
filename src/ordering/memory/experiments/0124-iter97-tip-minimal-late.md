# 0124 — iter97 tip+minimal late (hidden 2× load)

## Diagnosis
Five FAILs + watching iter96 `8346ffe`. Tip has no late polish / no chained LNS.
Hidden wall-clock (2× order + sandbox) likely kills even ≤1.00 local packages with heavy late/core streams.

## Leap
1. Drop chained LNS entirely (iter85)
2. Mid-core streams 3→**2**
3. Late polish leaner than iter96:
   - skip cost>**16M** (was 20M)
   - mid: **1** stream @8M + 1-round descent
   - cheap: **3** streams + simplicial 3M
   - no mid simplicial
4. Keep core_exact_shots=1, faclay nnz≥1.2M skips

## Target
≥15 movers; worst ≤0.95–1.00; survive hidden double-run.

## Result
Local **0.805574** / worst **0.982 s** / **19/3** movers. Submitting after `8346ffe` FAIL.
