# 0140 — n≤1000 dense-hole exact + rebuild3 after leftover n≤1500 FAIL

- **Date:** 2026-09-08
- **Score:** crown `767130f` 0.804873 → **0.804869** (−0.04 bip). Buckets 0.887816 / 0.842581 / 0.714375.
- **Status:** win locally, submitted. Replaces leftover n≤1500 after `fc7c3ce` hidden FAIL.

## Hypothesis

`fc7c3ce` proved leftover four/triple/five2/rebuild2 on 1000<n≤1500 is the hidden 2 s killer, even without watcher/rebuild3. Clamp leftover back to n≤1000. Search a new neighbourhood inside that gate: the nnz>30k dense hole that early exact (30k) and late polish (12k) never see. Add rebuild3 only when rebuild2 won, still n≤1000.

## What changed

`src/ordering/mod.rs` only. Leftover LT1K stays 1000. No watcher. No n>1000 new work.

## Result

gt_10k and 1k_10k bit-identical to 767130f. lt_1k 0.887829→0.887816. Movers: qap 3652461→3650113, qspp_0_11 3537346→3531915. search_par 4×16M SIGKILL'd maxcsp-langford-3-11 locally — not shipped. Dense exact clamped to n≤400.
