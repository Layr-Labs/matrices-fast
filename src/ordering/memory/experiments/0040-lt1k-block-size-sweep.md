# 0040 — The `lt_1k` block-size cap is the knob, and it wanted to be SMALLER

**Date:** 2026-09-03
**Base:** frontier `1deddca` (submission `a43ed612`, hidden 0.874999), dev 0.849801
**Result:** dev **0.849487** (−0.000314), and the corpus worst `order()` DROPS
1.774 s → 1.610 s. A score win that is also a timing win.

## Where this came from

[0038](0038-subtree-chain-into-lt1k.md) opened the subtree chain to `lt_1k` and
shipped ONE small-graph configuration — `min_s 16, max_s 512, max_blocks 8,
budget 4M` — chosen by analogy with the terminal deep pass, not by measurement.
Its own "Next" section flagged that only that single point had been tested. This
is the sweep.

## Instrument

`probe_lt1k` (new, test-only): scores **only** the 147 `lt_1k` matrices and dumps
every per-matrix ratio. The bucket runs in ~26 s against ~145 s for the full corpus,
which is what made a 16-point sweep affordable. Because the bucket geomean enters
the score linearly at weight 0.30 with the other buckets held fixed,
`score_delta = 0.30 * bucket_delta` — verified exactly below.

## The sweep

All configurations hold the 32M requested-work ceiling (`blocks x budget x
streams`), so this is purely about how that fixed budget is SHAPED.

| min_s | max_s | blocks | budget | bucket geomean |
|---|---|---|---|---|
| 16 | 768 | 8 | 4M | 0.895501 |
| 16 | 768 | 4 | 8M | 0.895190 |
| 16 | 512 | 16 | 2M | 0.895171 |
| 16 | 512 | 8 | 4M | 0.894939 ← 0038's shipped config |
| 16 | 384 | 32 | 1M | 0.895010 |
| 16 | 384 | 24 | 1.33M | 0.894965 |
| 16 | 384 | 16 | 2M | 0.894619 |
| 16 | 320 | 32 | 1M | 0.894483 |
| 16 | 288 | 16 | 2M | 0.893955 |
| **16** | **256** | **16** | **2M** | **0.893893** |
| 16 | 224 | 16 | 2M | 0.893961 |
| 16 | 192 | 16 | 2M | 0.894074 |
| 16 | 128 | 16 | 2M | 0.894402 |
| 16 | 96 | 16 | 2M | 0.894335 |
| 8 | 512 | 8 | 4M | 0.894939 (identical to min_s 16) |

Two readings:

1. **`max_s` — the cap on how large a searched block may be — is the dominant
   knob, and 0038 had it more than 2x too high.** Sweeping it from 768 down to
   256 is worth more than any block-count or budget change.
2. **`min_s` below 16 does nothing.** `min_s 8` reproduces `min_s 16`
   byte-for-byte, so the block floor is already past the point of diminishing
   return. Do not sweep it again.

The optimum is a **plateau, not a spike**: 224 / 256 / 288 all land within
7e-5 of each other (0.893961 / 0.893893 / 0.893955). That is the main reason to
trust it — a single lucky point would not have two neighbours agreeing.

## Robustness (the 0004 bar)

[0004](0004-structured-relabelings.md) established that this family is a lottery
whose apparent wins often rest on one matrix and flip sign across disjoint corpus
halves. Every candidate here was therefore checked on two disjoint halves
(alternating in name-sorted order) and with its three biggest wins dropped:

| config | all | half A | half B | drop-top-3 | verdict |
|---|---|---|---|---|---|
| **max_s 256, 16x2M** | **−0.001045** | −0.001106 | −0.000983 | **−0.000269** | **ROBUST** |
| max_s 224, 16x2M | −0.000978 | −0.000642 | −0.001323 | −0.000185 | robust |
| max_s 288, 16x2M | −0.000984 | −0.001073 | −0.000891 | −0.000178 | robust |
| max_s 320, 32x1M | −0.000455 | −0.000261 | −0.000655 | **+0.000057** | FAILS |
| max_s 384, 16x2M | −0.000320 | −0.000182 | −0.000461 | **+0.000051** | FAILS |

The two configurations nearest 0038's shipped point **fail** the drop-top-3
test — their gains rest on three matrices. The 224-288 plateau passes on every
column. This is why the sweep result is 256 and not "whatever scored best".

18 matrices improve and 4 regress against 0038's config (all still at or below
the AMD floor, which the best-of anchor guarantees); the net is robust.

## Timing: the change is FREE, and then some

| | corpus worst | lt_1k worst |
|---|---|---|
| 0038 shipped (max_s 512, 8x4M) | 1.774 s | 0.784 s |
| **this (max_s 256, 16x2M)** | **1.610 s** | 0.748 s |

Capping block size lowers the work actually performed per block, so the revision
is **cheaper as well as better**. Given that 0025 failed the hidden 2 s cap three
times, a change that buys score while REDUCING the worst case is the best kind
available here.

## Verified arithmetic

Predicted `0.849801 + 0.30 * (0.893893 - 0.894939) = 0.849487`. Measured on the
full corpus: **0.849487**. The bucket model is exact, which is what makes
`probe_lt1k` a valid stand-in for the full run during a sweep.

## Next

* `max_s` was never swept for the `n >= 1000` chain, where `SUBTREE_CFG.max_s`
  is still 384. If the same "too high" finding holds there, `1k_10k` and
  `gt_10k` have the same free win — and they carry 0.70 of the weight.
  **This is the strongest open lead in the repo right now.**
* 55 `lt_1k` ties remain untouched by any of this.
