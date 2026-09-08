# 0099 — Max sub-10k EXTRA_METRICS densify, tip-strict MINL (no after-core)

**Date:** 2026-09-07 ~18:42 PT. **Base:** `c6b0311` / `2e6f2ee` (hidden **0.850463**; local **0.806243**).
**Result:** local **0.806143** (−1.00 bip). gt_10k 0.7150 (print-identical). Not submitted (thin; prior densify+MINL fails).

## Package
1. Reverted MINL to tip `!core_path_improved` only (both jonathan308 MINL-after-core submits failed).
2. Max unused `EXTRA_METRICS` under `n < 10k`: 14 specs × α∈{10,5,2.5,1}.
3. Kept +2 medium exact else-branch tickets (`n ≤ 6k`).
4. Reverted null PEO_OVERSIZE 4M → 2.5M.

## Context
- iter14 PEO ledger expand: null 0.806156.
- Best prior densify-ish local: iter10 0.806121.
- Standing: prefer ≤1.0–1.1s worst; gt_10k bit-identical densify.
