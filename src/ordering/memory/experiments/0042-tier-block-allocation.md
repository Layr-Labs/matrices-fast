# 0042 — Tier the block ALLOCATION too (and two confirmations)

**Date:** 2026-09-03
**Base:** our promoted frontier `223023b` (submission `0cb69525`, hidden 0.874307), dev 0.848955
**Result:** dev **0.847732** (−0.001223), from `1k_10k` 0.873403 → 0.872433 and
`gt_10k` 0.796916 → 0.794594. `lt_1k` byte-identical.

## Two confirmations first (both negative, both worth not re-deriving)

[0041](0041-size-tiered-block-cap.md) left two leads. Both are now closed:

**`gt_10k`'s `max_s` was already optimal.** Swept in isolation for the first time
(it had only ever been varied through the shared constant). The curve is cleanly
unimodal with the minimum exactly at the inherited value:

| LARGE_MAX_S | 288 | 320 | 352 | **384** | 416 | 448 |
|---|---|---|---|---|---|---|
| gt_10k | 0.7987 | 0.7983 | 0.7977 | **0.7969** | 0.7994 | 0.8004 |

**The later rounds' windows are already optimal.** `cfg3.max_s = 512` and
`cfg4/cfg5.max_s = 768` apply to every size and had never been swept. Both
directions lose:

| cfg3 / cfg4-5 | 256/384 | 384/512 | **512/768** | 512/1024 | 640/1024 | 768/1200 |
|---|---|---|---|---|---|---|
| SCORE | 0.849279 | 0.849111 | **0.848955** | 0.849015 | 0.849098 | 0.849249 |

The widening schedule across rounds (384 → 512 → 768) is deliberate and correct:
round 1 wants a small window, later rounds want larger ones. Do not re-sweep.

## The win: allocation is a per-tier knob, not just `max_s`

0040 found that for `lt_1k`, at a FIXED 32M ceiling, 16 blocks x 2M beats the
default 32 x 1M. That question was never asked of the other two tiers — they
still used 32 x 1M. Asking it:

**`1k_10k`** (`MID_BLOCKS` x `MID_BUDGET`, product fixed at 32M):

| allocation | 64x0.5M | 32x1M | 20x1.6M | **16x2M** | 12x2.67M | 8x4M |
|---|---|---|---|---|---|---|
| 1k_10k | 0.8750 | 0.873403 | 0.8729 | **0.872433** | 0.8733 | 0.872460 |

**`gt_10k`** (`LARGE_BLOCKS` x `LARGE_BUDGET`, product fixed at 32M):

| allocation | 32x1M | **16x2M** | 8x4M |
|---|---|---|---|
| gt_10k | 0.796916 | **0.794594** | 0.793966 |
| worst `order()` | 1.636 s | 1.653 s | **1.957 s** |

**`8x4M` scores best on `gt_10k` and is REJECTED anyway.** It measured 1.957 s on
the worst matrix — 98% of the 2 s SIGKILL. 0025 failed the hidden cap three times
from exactly this kind of margin, and this box is already ~2x slower than the one
the older pages used. 16x2M gets most of the gain and leaves the worst case
unchanged at 1.653 s. **Score is not worth a cap failure.**

## Robustness (the 0004 bar)

Disjoint halves + drop-top-3, against each tier's 32x1M base:

| tier | config | d_all | half A | half B | drop-top-3 | better/worse |
|---|---|---|---|---|---|---|
| 1k_10k | **16x2M** | −0.000977 | −0.000638 | −0.001303 | −0.000327 | 33/16 |
| 1k_10k | 8x4M | −0.000949 | −0.000547 | −0.001338 | −0.000153 | 40/14 |
| gt_10k | **16x2M** | −0.002320 | −0.002549 | −0.002077 | −0.000438 | **23/2** |
| gt_10k | 8x4M | −0.002948 | −0.003095 | −0.002792 | −0.001022 | 27/2 |

All robust. The `gt_10k` result is the cleanest in the repo so far: the two
halves agree closely (−0.00255 / −0.00208, not the ~4x spread 0041 had) and only
**2 of 45** matrices regress.

Top `gt_10k` movers: `pooling_sppc3pq` 0.6821→0.6382, `pooling_sppc1pq`
0.2048→0.1995, `mpbp_21` 0.8871→0.8729, `crudeoil_pooling_dt3` 0.9671→0.9615,
`gams05` 0.7920→0.7879.

## The shape of the whole finding, across 0040-0042

Every tier wants roughly **16 blocks of ~2M**, not 32 blocks of 1M, and wants a
`max_s` matched to its size class. The chain shipped one global allocation for
all three. Current state:

| tier | max_s | blocks x budget |
|---|---|---|
| `n < 1_000` | 256 | 16 x 2M |
| `1_000 <= n < 10_000` | 128 | 16 x 2M |
| `n >= 10_000` | 384 | 16 x 2M |

The requested-work ceiling is 32M for every tier — unchanged from what shipped
before 0040. Nothing here spends more; it spends the same budget differently.

## Next

* `min_s` is still 32 for both upper tiers and was never swept there (below
  `n = 1000` it is 16, and 0040 showed 8 is a no-op at that size).
* `max_sub` (1_200) has never been touched at any size.
* 55 `lt_1k` ties remain.
