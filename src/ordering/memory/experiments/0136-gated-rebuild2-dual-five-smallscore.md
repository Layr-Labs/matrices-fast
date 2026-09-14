# 0136 — Gated second rebuild + dual five n≤3000 + terminal SmallScore

- **Date:** 2026-09-08
- **Score:** crown `7257386` local 0.804919 → **0.804857** (−0.62 bip). Buckets 0.887865 / 0.842492 / 0.714375.
- **Status:** win locally, submitted. Replaces 0135 after `f606aae` hidden FAIL.

## Hypothesis

0135's mid-band 2M watcher (`n>4k`) failed hidden Benchmark. Darthweenies' ungated second chained rebuild (`5f4e82b`) also failed hidden timing. Jonathan's dual five 64M (`f86e225`) passed the cap then rejected thin (0.849082 vs 0.84913). His later unconditional cfg_agg 6M (`6a5d8a4`) failed like the 12M parent.

The remaining cap-safe remainder of "refine what ships":

1. Same gain-conditioned second rebuild as `5f4e82b`, but **`n < 10_000`** so crudeoil_lee4_09 / methanol / popdyn / faclay never enter.
2. Gain-conditioned **second** five-descent at 32M, only where the first 32M five strictly won and **`n ≤ 3_000`** (lee1_07 n=3670 stays out).
3. Terminal SmallScore on **all** `n ≤ 1_000` (nnz cap dropped; cost tracks n), then one restart from the new incumbent.

## What changed

`src/ordering/mod.rs` only. Watcher from 0135 removed. No gate changes to FINAL_REFINE, first rebuild, simp/pair, or first five.

## Result

| tree | score | lt_1k | 1k_10k | gt_10k |
|---|---:|---:|---:|---:|
| five-crown `7257386` (author) | 0.804919 | 0.8880 | 0.8426 | 0.7144 |
| 0135 with watcher (failed hidden) | 0.804898 | 0.887922 | 0.842572 | 0.714375 |
| ungated rebuild2 + SmallScore (not shipped) | 0.804842 | 0.887922 | 0.842539 | 0.714258 |
| **this package** | **0.804857** | **0.887865** | **0.842492** | **0.714375** |

gt_10k is a control (bit-identical 0.714375). Scoring claim is lt_1k (SmallScore + dual five n≤3000 overlap) plus 1k_10k (gated rebuild2 + dual five).

Isolated `probe_timing_and_score` worst: crudeoil_lee1_07 **1.021 s**, crudeoil_lee4_09 **1.005 s**. Prefer-≤1.00 missed; prefer-≤1.10 met. lee4_09 is outside both new gates (`n>10k` and `n>3000` and `n>1000`).

## Why it won

Same FINAL_REFINE observation: later stages replace the perm; local search that ran earlier never sees the replacement. Gating rebuild2 under 10k drops the gt_10k sliver that 5f4e82b paid the hidden cap for, and keeps the 1k_10k part.

## Follow-ups

- Do not ungate rebuild2 onto n≥10k (5f4e82b death).
- Do not retry the 2M mid-band watcher (f606aae death).
- Do not copy unconditional cfg_agg (b9c8d06 / 6a5d8a4 deaths).
- On REJECT <1 hidden bip: expected if translation is ~1.2× like dual-five; do not widen gates.
- On FAIL n/a: drop dual five first (the only new work that can sit on n>1k besides gated rebuild2), keep SmallScore.
