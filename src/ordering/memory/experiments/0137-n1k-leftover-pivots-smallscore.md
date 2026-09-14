# 0137 — n≤1000-only leftover pivots + SmallScore on the shipped perm

- **Date:** 2026-09-08
- **Score:** crown `7257386` 0.804919 → **0.804873** (−0.46 bip). Buckets 0.887829 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Replaces 0136 after `c7c1a8a` hidden FAIL.

## Hypothesis

Every hidden FAIL this afternoon added work above n=1000 (watcher n>4k, ungated rebuild2, rebuild2 n<10k, five2 n≤3000, cfg_agg). Clamp every *new* pass to n≤1000. Re-introduce four/triple (dropped from the n≤4000 5/4/3 death) and an extra pair inside that gate. Keep rebuild2 and five2 only on n≤1000. SmallScore with four restarts, no nnz cap.

## What changed

`src/ordering/mod.rs` only. No new work on n>1000. Crown first-five n≤4000 unchanged.

## Result

gt_10k bit-identical 0.714375. 1k_10k 0.842581 ≈ crown. lt_1k 0.8880 → **0.887829**. multiplants_stg5 0.425→0.421. Worst isolated order(): lee4_09 **0.989 s**, lee1_07 **0.949 s** (neither enters a new gate).

## Follow-ups

- Do not ungate four/triple/rebuild2/five2 above n=1000.
- On REJECT thin: expected at ~1.2× translation; need a new generator, not a wider n gate.
- On FAIL: unexpected (no new n>1000 work); drop four/triple first.
