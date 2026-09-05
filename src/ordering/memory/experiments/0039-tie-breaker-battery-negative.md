# 0039 — The separator/min-fill family is exhausted on the surviving ties (NEGATIVE)

**Date:** 2026-09-03
**Base:** frontier `971649b`, dev 0.850594
**Result:** **NEGATIVE, nothing shipped.** Closes the "big tied matrices" open question.

## Hypothesis

87 of 300 dev matrices are still tied at exactly 1.000 (56 `lt_1k`, 21 `1k_10k`,
10 `gt_10k`). Every tie is pure upside under the best-of floor, and
`open-questions.md` carried a long-standing top lead that the biggest tied
matrices (`faclay75`, `acopf_case9241pegase_qcqp`, `gabriel10`,
`unitcommit_200_100_1_mod_8`) "are gated out of everything" and might have unused
budget. Test: throw the whole separator / min-fill family at every surviving tie
in the two leverage-rich buckets and see what breaks.

## Method

Added `probe_tie_breakers` (test-only). For each of the **31** tied matrices with
`n >= 1000`, it runs 16 candidates and reports each one's wall-clock **and** the
ratio it would achieve:

* METIS, more work: `niparts 16/fm 20`, `niparts 32/fm 30`
* METIS, different shape: `seed` 7/21, `nd_to_amd_switch` 100/400/1000,
  `max_imbalance` 0.05/0.30
* Scotch default, Scotch `n_sep_trials = 10`
* KaHIP default, KaHIP `Eco`
* AMF at `dense_alpha` 2.0 / −1.0 / 16.0

496 measurements in total.

## Result: zero wins

**Not one of the 16 candidates goes below ratio 1.0000 on any of the 31 ties.**
The minimum ratio achieved, per candidate, across all 31 matrices is exactly
1.0000 for every single one. These ties are not "unreached" — they are places
where AMD genuinely wins and more of the same family does not help.

## And the big matrices are worse than unreachable

`probe_large` on the `n >= 100_000` matrices measures both halves of the trade:

| matrix | n | nnz | cur ratio | METIS time | METIS ratio |
|---|---|---|---|---|---|
| `faclay75` | 272878 | 1379706 | 1.0000 | **14.7 s** | 2.2273 |
| `gabriel10` | 244056 | 1148210 | 1.0000 | 2.1 s | **4.4925** |
| `acopf_case9241pegase_qcqp` | 313068 | 1292408 | 0.9994 | 2.2 s | 2.9325 |
| `unitcommit_200_100_1_mod_8` | 146830 | 476332 | 0.9918 | 0.5 s | 1.8241 |
| `cont6-qq` | 120395 | 557994 | 0.8899 | 0.8 s | 1.3930 |

Nested dissection on these KKT graphs is not merely unaffordable, it is **badly
wrong** — 2.2× to 4.5× worse than AMD. Scotch on `faclay75` returns ratio 9519;
KaHIP takes 38–48 s. AMF at three `dense_alpha` values returns exactly 1.0000 on
`faclay75` and 1.0275 on `gabriel10`.

So the existing `METIS_MAX_N = 130_000` / `METIS_MAX_NNZ = 320_000` gates are not
leaving value on the table; they are protecting the run. The header comment on
`METIS_MAX_NNZ` ("measured: 6.2 s at nnz≈1.38M") understates it on slower
hardware — it is 14.7 s here.

## Conclusion

Do not spend further effort adding separator or min-fill variants to the tie set
in `1k_10k` / `gt_10k`, and do not widen the partitioner gates toward the large
matrices. The remaining headroom in this architecture is in **local search on the
incumbent** (the subtree chain), not in more global candidates — which is what
[0038](0038-subtree-chain-into-lt1k.md) then exploited, in the one bucket the
chain had never been allowed to touch.
