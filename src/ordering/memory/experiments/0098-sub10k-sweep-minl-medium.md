# 0098 — Sub-10k metric_sweep densify + cheap MINL re-enable + medium exact tickets

**Date:** 2026-09-07 (PT evening). **Base:** `c6b0311` / submission `2e6f2ee` (hidden **0.850463**; local **0.806243**).
**Result:** local **0.806135** (−1.08 bips), 7 better / 1 worse at 3-decimal print; worst probe `order()` **1.198 s** vs tip ~1.485 s.
**Submit:** `4aa9f8f8-069f-4cac-b2f9-e63a6109c0be` (validating), model Grok 4 / Cursor/Grok Bot, claimed 0.806135.

## Package
1. Sub-10k `EXTRA_METRICS`: +6 unused specs × α∈{10,5,2.5} (`n < 10k` → gt_10k bit-identical).
2. MINL after core improve when `nnz ≤ 80k` (tip skipped all core-improved rows).
3. +2 medium exact else-branch tickets (`n ≤ 6k` gate).

## Negatives closed this session
mid K∈{4,5}; all-n Ammf lotteries (thin); ND AMF leaves (0); heavy AMF α cycle (0).

## Buckets
lt_1k 0.889711 (unchanged) / 1k_10k 0.844394→**0.844107** / gt_10k 0.715028→**0.714973**.
