# 0168 — 0148 SqDiv↔DegP125 (n>16k) + gasprod_band 16k

- **Date:** 2026-09-09
- **Score:** 0.793849 (MISS)
- **Status:** miss — reverted
- **ID note:** Planned as Order's 0167 (0148 package), but concurrent agent claimed 0167 for subtree-chain-50k (tip-flat miss, reverted). Renumbered to 0168.

## Hypothesis

Two free/near-free tip-relative components that are **not** yet on `74b6ccd`:

1. **0148 substitute:** tip metric list in `indep_first.rs` is
   `[DegDivNvSqrtWf, DegPlusDegme, DegSqrt, SqDiv, DegDivNvDegme, DegP075]`.
   On `n > 16_000`, replace `SqDiv` with `DegP125` in the same slot (same count).
   On `n <= 16_000` keep the tip list bit-identical. Avoids inventing new heavy-metric
   dispatcher arms (0165/0166 rejected).

2. **gasprod_band:** tip still has `gasprod_band = n >= 20_000` in `mod.rs`.
   Lowering to `n >= 16_000` was a free ≥0.3–0.5 bip 0-regression component on older
   tip (0154b) and is not yet on `74b6ccd`.

Do **not** stack FINAL_FIVE 14k / REDUCE deep / PAIR 96M / midexpand from jonathan 488a
(rejected fa75759). Do **not** retread 0163–0166 / AmindNorm / mid-xset / second MINL /
extrarelbl / unconditional re-transplant.

## What changed

`src/ordering/indep_first.rs`:
- Metric loop branches on `n > 16_000` → DegP125 where SqDiv was; else tip list.

`src/ordering/mod.rs`:
- `gasprod_band = n >= 16_000` (was 20_000).

## Result

Yukon local **0.793849** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0168.log`, `score.json`).
Fill 0.925786 (tip 0.925772). Buckets: lt_1k/1k_10k unchanged; gt_10k 0.689129 → 0.689167.

**0 better / 2 worse** / 298 same versus `/tmp/yukon-run-tip-74b6ccd.log`:
- pinene200 (n=19995): 0.835 → 0.837 (+0.25% ours)
- gabriel09 (n=21688): ours +0.002% (ratio display 0.913)

Below submit bar. Not submitted. `mod.rs` + `indep_first.rs` restored to tip; follow-up 0169 isolates gasband-only.

## Why it won / lost

The DegP125↔SqDiv swap on n>16k regressed at least gabriel09 (already in tip gasprod_band ≥20k, so metric-only). pinene200 sits in the new 16k–20k gasband *and* the metric gate, so the combined package cannot attribute blame cleanly — hence gasband-only isolate next. No free bip; package is a local regression.

## Follow-ups

- If combined package misses because of the metric swap: isolate gasband-only.
- Else: 0164 follow-up with residual-core polish + explicit best-of floor (never additive).

## Links

- Tip `74b6ccd`; board best 0.84349; promote bar ≤ 0.843389; local rebaseline 0.793834 / fill 0.925772
- Rejected neighbors: [0163](0163-subtree-chain-past-45k.md), [0164](0164-late-phase-on-core.md), [0165](0165-heavy-metric-degsqrt.md), [0166](0166-heavy-metric-cm-sqdiv.md), [0167](0167-subtree-chain-50k.md)
