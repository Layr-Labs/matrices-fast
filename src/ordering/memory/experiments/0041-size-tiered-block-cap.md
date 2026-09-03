# 0041 — The block-size cap must SCALE WITH GRAPH SIZE

**Date:** 2026-09-03
**Base:** our promoted frontier `344a5d2` (submission `26932eba`, hidden 0.874601), dev 0.849487
**Result:** dev **0.848955** (−0.000532), driven entirely by `1k_10k`
0.875176 → 0.873409. `lt_1k` and `gt_10k` are byte-identical.

## The question 0040 left open

[0040](0040-lt1k-block-size-sweep.md) found that `max_s` — the cap on how large
a searched subtree block may be — was more than 2x too high for `lt_1k`, and
noted the obvious follow-up: the `n >= 1000` chain still used the shared
`SUBTREE_CFG.max_s = 384`, chosen the same way (by analogy, not measurement),
and `1k_10k` + `gt_10k` carry **0.70** of the score weight against `lt_1k`'s 0.30.

## First result: the global knob is already right, and that is the finding

Sweeping the shared `SUBTREE_CFG.max_s` for all `n >= 1000`:

| SUBTREE_CFG.max_s | 1k_10k | gt_10k | SCORE |
|---|---|---|---|
| 256 | **0.8742** | 0.7991 | 0.850057 |
| **384 (shipped)** | 0.8752 | **0.7969** | **0.849487** |
| 512 | 0.8760 | 0.8006 | 0.851202 |

Both alternatives are worse *overall* — but look at the split. Lowering the cap
to 256 **improves `1k_10k`** (0.8752 → 0.8742) while **hurting `gt_10k`**
(0.7969 → 0.7991). The two buckets want opposite things, and a single global
value is a compromise that serves neither.

That is the real finding: **the block cap is not a constant, it is a function of
graph size.** It was already tiered without anyone noticing — 0040 gave small
graphs 256 while everything else kept 384 — and the middle bucket was simply
sharing the large tier's value.

## The change

One constant and one `else if` in `subtree_cfg_for(n)`:

```
n < 1_000        -> max_s 256   (0040)
1_000 <= n < 10k -> max_s MID_MAX_S = 128   (this experiment)
n >= 10_000      -> max_s 384   (unchanged; measured best for gt_10k)
```

## The mid-tier sweep

| MID_MAX_S | 1k_10k bucket | SCORE |
|---|---|---|
| 384 (no tier) | 0.875176 | 0.849487 |
| 320 | 0.8745 | 0.849283 |
| 256 | 0.8742 | 0.849194 |
| 192 | 0.873875 | 0.849097 |
| 160 | 0.873947 | 0.849117 |
| **128** | **0.873409** | **0.848955** |
| 112 | 0.873709 | 0.849046 |
| 96 | 0.873898 | 0.849104 |
| 64 | 0.8746 | 0.849307 |

A clean basin with a minimum at 128 and a turning point below it — the same
shape 0040 found for `lt_1k`, at a different scale. Note the optimum for
`1k_10k` (128) is *smaller* than for `lt_1k` (256), which is the opposite of
what "scale with n" naively predicts; the mid bucket's chain also runs more
rounds, so the per-round window interacts with the round schedule. Recorded as
an observation, not a theory.

## Robustness (the 0004 bar)

Every candidate checked on two disjoint halves (alternating in name-sorted
order) and with its three biggest movers dropped, against the `max_s 384` base:

| MID_MAX_S | d_all | half A | half B | drop-top-3 | better/worse | verdict |
|---|---|---|---|---|---|---|
| 192 | −0.001304 | −0.000376 | −0.002201 | −0.000519 | 33/17 | robust |
| 160 | −0.001232 | −0.000291 | −0.002143 | −0.000330 | 30/22 | robust |
| **128** | **−0.001770** | **−0.000718** | **−0.002787** | **−0.000596** | 35/18 | **ROBUST** |
| 112 | −0.001469 | −0.000297 | −0.002603 | −0.000270 | 37/17 | robust |
| 96 | −0.001281 | −0.000116 | −0.002407 | −0.000142 | 33/21 | robust |

The whole 96-192 basin passes every column and 128 is its deepest point, which
is the plateau-not-spike signature 0040 argued for. Caveat worth recording: the
two halves disagree in MAGNITUDE by about 4x (−0.0007 vs −0.0028) even though
they agree in sign, so the size of this win is less certain than its direction.

Top movers at 128: `gasprod_sarawak16` 0.9955→0.9351, `chimera_selby-c16-01`
0.6063→0.5769, `pooling_sppa0pq` 0.3664→0.3536, `crudeoil_pooling_ct1`
0.7759→0.7535, `lop97icx` 0.8704→0.8520.

## Timing

Corpus worst `order()` 1.594 s → 1.565 s across the sweep — unchanged within
noise, and no tier raises it. A smaller block cap does less work per block, so
like 0040 this is not a cost increase.

## Next

* `gt_10k` still uses 384 and was never swept in ISOLATION — only as part of the
  shared constant, where its best value was 384 of {256, 384, 512}. A dedicated
  `LARGE_MAX_S` sweep at finer resolution (320/352/416/448) is the obvious next
  step, and `gt_10k` carries 0.40 weight on its own.
* The later chain rounds hard-code their own windows (`cfg3.max_s = 512`,
  `cfg4/cfg5.max_s = 768`) for ALL sizes. Those were never swept either, and by
  the logic above they should probably be tiered too.
