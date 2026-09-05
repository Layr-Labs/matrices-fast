# 0038 — Extend the bounded subtree chain into `lt_1k`

**Date:** 2026-09-03
**Base:** frontier `bf39c18` (submission `c67fe6e`, hidden 0.875560), which
measures dev **0.850225** on this box.
**Result:** dev **0.849801** (−0.000424), fill → 0.947880. 17 better / **0 worse** / 283 identical.

> Developed on frontier `971649b` and rebased TWICE as the leaderboard moved
> mid-session (hybridnoise promoted three times in ~90 minutes). The delta is
> essentially invariant across all three bases — **−0.000427, −0.000427,
> −0.000424** — which is the expected result for a change confined to a bucket
> the other work barely touches, and is much stronger evidence than any single
> measurement.

> **The knowledge base lags the code — re-run the base, don't read it.** At
> commit `1417f26`, `index.md` reported the best local score as 0.850464 with
> buckets 0.896482 / 0.875665 / 0.797049, citing
> [0035](0035-chained-terminal-subtree-refinement.md). Re-measured, that checkout
> actually scores **0.850370** with 1k_10k **0.875531** and gt_10k **0.796916** —
> because `1417f26` is already
> [0036](0036-multiround-cascading-terminal-subtree-refinement.md)'s tree, whose
> own log line records exactly 0.850370, while the index block still described
> 0035. The page was stale relative to the `mod.rs` committed beside it. Since
> the score is a deterministic, hardware-independent function of
> `(pattern, permutation)`, the fix is mechanical: run the probe on the unmodified
> base before claiming any delta against it. I caught this only because `gt_10k`
> appeared to move when the change provably cannot touch `n > 10_000`; trusting
> the page would have had me claim −0.000521 instead of the true −0.000427.

## The observation

The bounded subtree-refinement chain — the technique behind *every* scoring win
from [0021](0021-exact-subtree-refinement.md) through
[0025](0025-adaptive-terminal-deep-subtree-search.md) — was gated at `n >= 1_000`
from the day it was introduced. Both its entry gate and the terminal deep pass
started at 1,000, so the whole `lt_1k` bucket never saw it.

The bucket table across those five experiments says exactly that:

| experiment | lt_1k | 1k_10k | gt_10k |
|---|---|---|---|
| 0021 | 0.8965 | 0.877695 | 0.798150 |
| 0022 | 0.8965 | 0.8803 | 0.7997 |
| 0023 | 0.8965 | 0.878657 | 0.7977 |
| 0024 | 0.8965 | 0.878037 | 0.797477 |
| 0025 | 0.896482 | 0.876098 | 0.797049 |

`lt_1k` is frozen at 0.8965 while the other two buckets move. It is also the
**worst** bucket and the second-largest by count (147 matrices, weight 0.30).

## Why it was safe to widen

`lt_1k` is the cheapest part of the corpus. Measured with
`probe_timing_and_score` on the frontier tree:

| | |
|---|---|
| corpus worst `order()` | 1.702 s (`arki0013`, n=44909) |
| **`lt_1k` worst `order()`** | **0.766 s** |
| `lt_1k` mean `order()` | 0.163 s |

So every `lt_1k` matrix carried ~0.9 s of unused headroom, and the matrices that
set the global worst case (`arki0013`, `crudeoil_lee4_10`, `gams05`,
`crudeoil_lee4_09`; all n ≥ 15,904) are untouched by an `lt_1k` gate change.

## What shipped

1. `SUBTREE_MIN_N = 64` replaces the literal `1_000` in both chain gates (the
   round-1 entry gate and the terminal deep pass).
2. `subtree_cfg_for(n)` gives graphs below `n = 1_000` a **reallocated** config:
   `min_s` 32 → 16, `max_s` 384 → 512, `max_blocks` 32 → 8, `budget` 1M → 4M.
   Blocks × budget × streams is `8 × 4M × 1 = 32M`, **identical** to the shipped
   `SUBTREE_CFG` ceiling. This is a reallocation, not an increase — the same
   discipline that brought 0022 back inside the cap after 0021 blew it.
3. The `n <= 1_000` exact randomized-greedy search gets a **second stream**
   (`50M` at seed `0xD1B5_4A32_D192_ED03`) beside the existing 100M one, mirroring
   what the medium branch already does. [0004](0004-structured-relabelings.md)
   settled that this family is a pure lottery with no exploitable local
   structure, so more tickets is the only lever that pays; the first entry is
   byte-identical to the accepted single stream, so this strictly adds a draw.

## Measured, step by step

Score is a pure function of the pattern, so these differences are exact, not noisy.

Development series on the `971649b` base:

| variant | lt_1k | SCORE | corpus worst | lt_1k worst |
|---|---|---|---|---|
| frontier (chain `n >= 1000`) | 0.8965 | 0.850594 | 1.702 s | 0.766 s |
| `SUBTREE_MIN_N = 200` | 0.8956 | 0.850332 | 1.745 s | 0.820 s |
| `SUBTREE_MIN_N = 64` | 0.8954 | 0.850259 | 1.800 s | 0.825 s |
| + second small-graph stream | 0.8953 | 0.850232 | 1.810 s | 0.802 s |
| **+ deeper small-graph blocks** | **0.8951** | **0.850167** | **1.721 s** | **0.824 s** |

Final, rebased onto the `bf39c18` base, both re-measured on this box:

| | lt_1k | 1k_10k | gt_10k | SCORE | corpus worst | lt_1k worst |
|---|---|---|---|---|---|---|
| base `bf39c18` | 0.896420 | 0.875176 | 0.796916 | 0.850225 | 1.734 s | 0.784 s |
| **this revision** | **0.894939** | 0.875176 | 0.796916 | **0.849801** | — | — |

`gt_10k` is byte-identical to the base and `1k_10k` moves only via
`clay0305hfsg` (n = 1000 exactly, which the `n <= 1_000` exact-search gate
includes but the `lt_1k` bucket excludes). All 17 movers have `n <= 1000`. That
control held on all three bases.

Biggest movers (all `lt_1k`, as designed):

| matrix | n | before | after |
|---|---|---|---|
| `multiplants_stg1a` | 726 | 0.8960 | 0.8645 |
| `ndcc12` | 974 | **1.0000** | 0.9791 |
| `waterund14` | 333 | 0.3855 | 0.3653 |
| `multiplants_stg1c` | 800 | 0.9796 | 0.9603 |
| `waterund11` | 160 | 0.7770 | 0.7611 |
| `multiplants_stg1b` | 814 | 0.6255 | 0.6099 |

## Timing

Corpus worst moved 1.702 s → 1.721 s, but the top four slowest matrices are the
same four in every run and none is in `lt_1k`; across seven runs of this box the
same code spans 1.69–1.81 s, consistent with the ~1.6× run-to-run variance the
index already records. The number this change actually controls is the `lt_1k`
worst, which moved **0.784 s → 0.821 s (+0.037 s)** — under half the corpus
worst case, on the cheapest matrices in the corpus.

## Caveat on this box

**This machine is ~2× slower than the one earlier pages were measured on.** The
frontier tree records "worst local call 0.829 s" in
[0025](0025-adaptive-terminal-deep-subtree-search.md); the same tree measures
**1.702 s** here. Absolute timings on this page are therefore not comparable to
pages 0002–0025, only to each other. The comparative rule still applies, and this
revision sits at the frontier's own worst case within noise.

## Next

The `lt_1k` mechanism is not exhausted — 55 ties remain there. Worth trying:
a third small-graph stream, and sweeping `max_blocks`/`budget` in the reallocation
(only 8 × 4M was tested).
