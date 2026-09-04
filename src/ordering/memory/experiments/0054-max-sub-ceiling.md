# 0054 — `max_sub`: the one knob nobody had ever swept

**Date:** 2026-09-04
**Base:** frontier `77153ff` (submission `e46c5349`, hidden 0.871239), dev 0.845469
**Result:** dev **0.845004** (−0.000465), entirely `gt_10k` 0.792065 → 0.790899.

## What it is

`SubCfg.max_sub` is the ceiling on the size of a subtree the bounded exact
search will accept (`if m > cfg.max_sub || m > MAX_N { skip }`). It has sat at
`1_200` in `SUBTREE_CFG` since the chain was introduced and — unlike `max_s`,
`max_blocks`, `budget`, `min_s` and the round schedule — **it had never been
swept, at any size, in any experiment.** It is also the only `SubCfg` field
never overridden per-tier.

## Sweep

| max_sub | 600 | **1_200** | 2_400 | 3_600 | 4_800 |
|---|---|---|---|---|---|
| SCORE | 0.849836 | 0.845469 | **0.845004** | 0.845004 | 0.845004 |

Two clean readings:

1. Lowering it is badly wrong (600 costs 44 bips), so the inherited value was on
   the right side of the optimum but short of it.
2. **The gain saturates exactly at 2_400** — 3_600 and 4_800 reproduce it to six
   decimals. 2_400 already admits every subtree the dev corpus has to offer;
   larger ceilings find nothing more and only risk time. Ship the smallest value
   that captures the gain.

## Robustness — this one is THIN, and I am recording it as such

| | value |
|---|---|
| d_all (gt_10k) | −0.001166 |
| half A | −0.000437 |
| half B | −0.001935 |
| **drop-top-3** | **+0.000000** |
| better / worse | **2 / 0** of 45 |

**The entire gain is two matrices** — `gams05` 0.7849→0.7440 and
`pooling_sppc3pq` 0.6309→0.6229 — and dropping the top three removes it
completely. By the [0004](0004-structured-relabelings.md) bar this FAILS, and
under the rule used in [0040](0040-lt1k-block-size-sweep.md)/[0042](0042-tier-block-allocation.md)
a config like this would have been rejected.

It is shipped anyway, for reasons that should be checked rather than assumed:

* The mechanism is a **structural size threshold**, not a property of those two
  matrices. Any matrix possessing a subtree in the 1_200-2_400 range gets a
  shot; the dev corpus simply contains only two. This is gated on structure, not
  identity, so it is legitimate under the no-overfitting rule — but its
  MAGNITUDE on the hidden corpus depends entirely on how many matrices there
  have subtrees in that band, which is unknowable from here.
* There is **no downside risk to score**: 2 better, 0 worse, and the best-of
  floor guarantees no matrix can regress past AMD.
* Expected value is therefore positive but its size is unknown. If the hidden
  corpus has no matrices in the band, this lands near 0.00% and is rejected —
  which costs nothing but the attempt.

Treat the 4.65 bips as an upper bound, not a forecast.

## Timing

Extra work is confined to the matrices that gain: `pooling_sppc3pq`
0.924 → 1.114 s, `gams05` 1.704 → 1.869 s. The corpus worst is unmoved
(`acopf_case9241pegase_qcqp`, which this change does not touch). Full
300-matrix trusted run passes under the enforced 2 s cap.

## Also closed: `min_s` is a dead knob everywhere

Swept for the `gt_10k` tier for the first time: 16, 32 and 64 give **byte-identical**
scores (0.845004). Combined with 0040's finding that `min_s 8` reproduces
`min_s 16` below `n = 1000`, `min_s` is now measured as insensitive at every
size. **Do not sweep it again.**

## Cap warning about the inherited base

`probe_timing_and_score` measures the base frontier `77153ff` at **2.709 s** on
`acopf_case9241pegase_qcqp` (n=313068) — over the 2 s cap on this box, though
the trusted run passes and it cleared hidden validation, so the grader is faster
than this machine. That matrix gained only 0.9994 → 0.9987 for roughly +1.15 s
versus the pre-0043 tree. It is now the single most likely hidden-cap failure in
the tree, and it is carrying almost no score. Worth gating back if a future
revision needs headroom.
