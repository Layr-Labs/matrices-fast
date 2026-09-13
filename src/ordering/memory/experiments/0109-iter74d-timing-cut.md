# 0109: iter74d timing-cut of iter72 family

**Date:** 2026-09-08 ~02:48 PT. **Base tip:** `c6b0311` / `2e6f2ee` (hidden **0.850463**; local **0.806243**).

**Score:** 0.806243 → **0.805536** (−7.07 bip). Movers **24/4**. Worst **1.087s** (confirm 1.093).

**Submit:** `b6040276` validating.

## Cuts
- Skip residual-core exact on danger (n≥1800 nnz≥9k)
- Late polish: n<3000 nnz≤12k; skip cost=n·nnz>20M
- Densify danger: switches {150,300} only
- Medium exact: danger → tip 4-ticket floor
- PEO_ALT: skip n≥2500 nnz≥9k

## Context
iter72: 0.805384 / ~29/0 / 1.196s FAIL (crudeoil_pooling_ct3 / chimera_selby).
