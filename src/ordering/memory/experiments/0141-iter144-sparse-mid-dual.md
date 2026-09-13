# 0141 — iter144 sparse mid-band dual same-tree + n≤1000 completion

- **Date:** 2026-09-08
- **Score:** tip 0.804873 → **0.804807** (−0.66 bip). Worst **1.000s**. **15** tip movers.
- **Status:** submitted `3b0c31a5-bb29-4308-b62d-a88adbbee8ba` validating.
- **Gate:** dual same-tree n∈(1k,7k) nnz≤17k; LT1K=1000; density-split SS; completion 4M.

## Hypothesis
Broad mid-n cfg_agg SIGKILLs; n≤1000-only thin. Sparse nnz≤17k keeps chimera/chp/li05/ct, drops lee1_07/lee4/pooling crowns.

## Follow-ups
- FAIL timing → iter145 single ticket nnz≤16k n<6k budget 1.5M (saved plan).
- REJECT thin → different mechanism.
- PROMOTE < 0.848883 → LEAD RECLAIMED + extend.
