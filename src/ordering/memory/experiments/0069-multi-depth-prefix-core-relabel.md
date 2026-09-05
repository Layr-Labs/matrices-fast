# 0069 — Multi-depth prefix K=2,4 + 0068 core relabel

- **Date:** 2026-09-05
- **Base:** `2a28517` (hidden **0.867211**)
- **Score:** 0.839063 → **0.837999** (−10.64 bip vs tip)
- **Harness:** TBD
- **Worst `order()`:** 1.486 s

## Changes

On top of 0068: alternate reduce prefixes K=2 and K=4 (AMF grid only, no core relabel)
on `n < 8000 && nnz <= 300k`; K=3 keeps full core relabel.
