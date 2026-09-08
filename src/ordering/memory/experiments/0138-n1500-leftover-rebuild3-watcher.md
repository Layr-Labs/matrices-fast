# 0138 — Widen leftover family to n≤1500 + rebuild3 + n≤1500 watcher

- **Date:** 2026-09-08
- **Score:** crown `767130f` 0.804873 → **0.804839** (−0.34 bip). Buckets 0.887767 / 0.842529 / 0.714375.
- **Status:** win locally, submitted.

## Hypothesis

767130f promoted hidden 0.848883 with all new work at n≤1000 (lt_1k translation ~5×). 33 public rows sit in 1000<n≤1500, still far from lee1_07 (3670) / lee4_09 (15904). Widen the same leftover family, add a gain-conditioned third rebuild, and a 2M watcher on n≤1500 (disjoint from f606aae's n>4k death).

## Result

gt_10k control. 1k_10k 0.842581→0.842529. lt_1k 0.887829→0.887767. multiplants_stg5 0.421→0.416. lop97icx 0.818→0.813. Worst isolated: lee4_09 1.085 s (outside new gates; noise vs 0.989 on the crown). sporttournament48 (n=1131, in-gate) 0.748 s.
