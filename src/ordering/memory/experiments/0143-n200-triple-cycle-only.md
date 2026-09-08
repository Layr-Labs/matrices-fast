# 0143 — n≤200 3-cycle only (drop lottery after 197b79f FAIL)

- **Date:** 2026-09-08
- **Score:** crown 0.804873 → **0.804869** (−0.04 bip). 0.887817 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Ablation of `197b79f`.

## Hypothesis

`197b79f` failed hidden with 3-cycle + independent lottery on every n≤400 row. The lottery is a second full SmallScore pass. Keep 3-cycles only, n≤200, so pooling_adhya4tp (n=170) stays in and n=200–400 extras are gone.

## Result

pooling_adhya4tp still 20540. Lost the n=200–400 sliver vs 0142 (0.804866→0.804869). 1k_10k/gt_10k controls.
