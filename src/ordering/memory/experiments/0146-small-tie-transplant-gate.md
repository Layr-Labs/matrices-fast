# 0146 — Small-tie transplant gate (v1 hidden fail, v2 hidden-identical)

- **Date:** 2026-09-09
- **Score:** tip `62654a5` (hidden 0.843173) dev **0.792300 → 0.792300**
  (bit-identical `score.json`: buckets 0.887390 / 0.839747 / 0.685397, fill
  0.924373); hidden v2 **0.843173, diff 0 (0.00%), REJECTED**.
- **Status:** hypothesis closed with data (both corpora).
- **Files:** `transplant_probe.rs` (+13/-1): `small_tie` gate in
  `refine_with_donors` (stage 14, terminal).

## Hypothesis

647a won by opening the terminal transplant to sparse-large near-AMD ties.
The dev corpus still holds 54 `lt_1k` + 17 `1k_10k` ties at exactly 1.000 that
the below-anchor + sparse-large gates exclude. Same 1%-tie test, smaller box,
density-guarded — pure coverage extension under the best-of floor.

## What happened

- v1 box (`n` 64..15k, `nnz` 2k..40k): dev bit-identical, but **FAILED hidden
  validation at the Benchmark step** (`actions/runs/34425938729`). No purity /
  bijection / determinism issue possible locally (300x2 runs pass); attributed
  to hidden timing — an unconditional constant stacked onto an already-hot
  hidden row (the 0088 failure mode).
- v2 box (`n` 64..8k, `nnz` 2k..12k, unit ≤ ~20k, sub-ms/row): dev
  bit-identical, hidden **validates cleanly but scores bit-identical
  (0.843173)**. The gate fires nowhere productively on either corpus.

## Conclusion

Small ties do not want transplant: on small rows the donor pool is the same
AMD-level quality as the incumbent, so there is nothing to transplant. Do not
retry transplant on rows below the sparse-large box. The v1→v2 shrink confirms
the timing attribution is at least plausible (v2 passes where v1 died), but
with zero score on both sides the family is closed here regardless.

## Negative controls this session (all reverted, all dev-null except #2)

1. Relabel-AMF alpha rotation `[5,2,-1,1,16]` → `[5,2,-1,2.5,0.5]`: dev
   identical. Lottery saturated; reverted.
2. Extra AMD tickets +4 (n≤1k) then +6 (n≤4k/nnz≤32k): full-package dev
   **0.792305 (+0.05 bips)** — early-phase extra draws re-roll downstream
   lotteries (0143 pathology). Reverted.
3. Dense-`lt_1k` extra SmallScore round: null. Reverted.
4. Mid-band hand-rolled envelope (RCM/Sloan/ND/GGGP for n in [1k,4k],
   nnz≤40k): `knp5-43`, `squfl010-040persp`, `hydroenergy1`,
   `pooling_foulds5pq`, `squfl025-025persp` all remain 1.000. Reverted.

## Next

The substitutive path is the only one with a pulse: cut the cap-infinity
METIS core pass in `indep_first` (0143: never wins except `pooling_sppc3pq`,
where cap-9 also wins), spend the ~0.1 s savings on `TRANSPLANT_LEDGER`
1M → 1.5M. Ledger 2M was -4.2 dev bips but died hidden; 1.5M funded by a real
cut is the safe interpolation. Never raise a ledger additively.

[Index](../index.md) | [Log](../log.md)
