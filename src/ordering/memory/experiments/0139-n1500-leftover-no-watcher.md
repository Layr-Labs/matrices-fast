# 0139 — n≤1500 leftover only (drop watcher + rebuild3 after 186c507 FAIL)

- **Date:** 2026-09-08
- **Score:** crown 767130f 0.804873 → **0.804857** (−0.16 bip). 0.887829 / 0.842529 / 0.714375.
- **Status:** win locally, submitted. Ablation of 186c507.

## Hypothesis

186c507 failed hidden with leftover n≤1500 + rebuild3 + 2M watcher. Drop the two etree-rebuild passes (watcher, rebuild3). Keep leftover widen (1k_10k 0.842581→0.842529) and SmallScore n≤1024.

## Result

lt_1k identical to 767130f (0.887829). 1k_10k moved. gt_10k control. Simplicial add-on measured 0.804857 bit-identical — reverted. Worst 0.992 s, cap rows out of new gates.
