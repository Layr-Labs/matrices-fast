# 0169 — gasprod_band 16k isolate

- **Date:** 2026-09-09
- **Score:** 0.793834 (MISS, tip-flat)
- **Status:** miss — reverted

## Hypothesis

0168 combined 0148's SqDiv→DegP125 (n>16k) with `gasprod_band` 20k→16k and **regressed** locally (0.793834 → 0.793849, 0 better / 2 worse). gabriel09 (n=21688) is already inside tip's ≥20k gasband, so that loss is attributable to the metric swap. Isolate the historically free 0154b gasband component alone on tip `74b6ccd`.

## What changed

`src/ordering/mod.rs` only:
- `gasprod_band = n >= 16_000` (was 20_000)

`indep_first.rs` stays tip (SqDiv list unchanged).

## Result

Yukon local **0.793834** vs tip **0.793834** (`/tmp/yukon-run-0169.log`, `score.json`). Fill 0.925772. Buckets identical.

**0 better / 0 worse** / 300 same versus `/tmp/yukon-run-tip-74b6ccd.log`. Matrices in the newly opened `[16_000, 20_000)` gasband were bit-identical to tip.

Below submit bar. Not submitted. `mod.rs` restored to tip.

## Why it won / lost

On tip `74b6ccd`, lowering `gasprod_band` alone does not move the public table — every row that would newly take the immediate-accept path already lands on the same incumbent after deferred accept + later stages. The 0154b free bip does not reproduce here. Combined with 0168, the DegP125 swap is the component that actively hurts; gasband is currently a no-op on this tip.

## Follow-ups

- Do not restack DegP125 for SqDiv on this tip without a separate census.
- Residual-core polish with best-of floor remains a separate candidate (0164 follow-up).

## Links

- [0168](0168-0148-sqdiv-degp125-gasband16k.md)
