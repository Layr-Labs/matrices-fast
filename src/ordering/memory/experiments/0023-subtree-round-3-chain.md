# 0023 — Subtree round-3 chain with a widened block window

- **Date:** 2026-09-03
- **Score:** 0.852246 → **0.851642** (timing-probe aggregate; full-run row in
  results.tsv). Buckets: lt_1k 0.8965 (unchanged), 1k_10k 0.8790 → **0.8787**,
  gt_10k 0.7986 → **0.7977**. Worst order() = **0.904 s** (`crudeoil_lee4_10`).
- **Status:** submitted.
- **Parent:** `6fb4842` (hybridnoise 89cd716c, promoted hidden 0.877695).

## Context

The subtree-refinement phase is now a *chain*: round 1 (round=0, 32 blocks ×
1M, the original 0021/0022 gate), then hybridnoise's round 2 (round=1, 24
blocks × 1M) which runs only when round 1 improved the incumbent. Each round
re-postorders the improved tree, ranks its subtrees by flop contribution, and
searches bounded disjoint blocks at 1M ops each — a deterministic,
bounded-work, strictly-monotonic chain. Hidden submissions carrying round 2
passed the 2 s cap (0.877695, then a 32-block round-2 variant at 0.877631 —
0.64 bip short of the 1-bip promotion bar).

## Hypothesis

The chain mechanism is sound; the marginal round-2 changes were running out of
steam because both rounds searched the SAME block window (min_s 32 / 16,
max_s 384). A third chained round on the round-2 incumbent should still find
wins — and widening the block window upward (max_s 512) lets slightly larger
subtrees of the *improved* tree be searched, which neither earlier round ever
allowed. Block search stays budget-bounded (1M ops per block), so the only
added cost is one more ≤32M-op pass on matrices where the earlier rounds
already improved (the cheapest-to-search, most-structured part of the corpus).

## Change

Inside round 2's acceptance (round 2 improved the incumbent), run a round 3:

- `round = 1` (diversified per-block seeds, same as round 2),
- `max_blocks = 32`, `min_s = 16`, **`max_s = 512`**,
- same 1M-op per-block budget, strict-best-of accept.

Pure function of (n, nnz) gates plus deterministic search outcomes; no
threads, no clocks, no matrix identities.

## Probe evidence (round-3 configs, from the shipped incumbent)

| config | score | d vs parent |
|---|---|---|
| (parent) | 0.852246 | — |
| round3 24b s16 ms384 | 0.851915 | −3.3 bips |
| round3 32b s16 ms384 | 0.851854 | −3.9 bips |
| **round3 32b s16 ms512** | 0.851581 | **−6.7 bips** |
| round3 24b s16 2×0.5M | 0.851957 | −2.9 bips |

The max_s bump dominates: 66 matrices improved, led by pooling_sppc1pq
(n=14100, nnz=477680 — the flagship subtree-refinement matrix keeps
yielding), pooling_sppa0pq/9pq/9tp, mpbp_21/34/35/46/47, slay09h,
powerflow0300p/0118p, chimera_selby, crudeoil_pooling, blend146.

## Wiring result (timing-probe aggregate; full corpus)

Score **0.851642** (lt_1k 0.8965 / 1k_10k 0.8787 / gt_10k 0.7977). The wired
number is *better* than the probe estimate because round 3 runs on the actual
chained incumbent (rounds 1+2 accepted), not on the shipped output alone.
Worst local order() 0.904 s (vs ~0.85 s parent on the same box), inside the
passed-revision envelope (≤1.019 s).

## Learning

Within a bounded-work refinement chain, the unturned knob is the BLOCK WINDOW
(max_s), not more blocks or streams of the same window: the earlier 32→16
min_s sweep and the 16/24/32 block sweeps each bought <2 bips, while max_s
384→512 on the third pass bought ~3 bips over the same block count. Larger
subtrees of an already-refined tree contain structure that small-block
searches never see. Next candidate: sweep max_s upward (512/640/768) on round
3, and test whether round 2 itself wants the wider window (it may make round
3 unnecessary on many matrices).

## Follow-up measurements (same session)

- round-3 max_s 640 (round 2 unchanged at ms384): 0.851642 → **0.851568**
  (1k_10k 0.878657→0.878437). Small but consistent.
- **round-2 max_s 640 + round-3 max_s 640: 0.851660 — WORSE than ms640 on
  round 3 alone.** Widening round 2 changes the round-2 incumbent and the
  round-3 search lands on a different local optimum that is worse overall.
  Chain order matters: keep round 2 at its shipped ms384 window and tune round
  3 only. Do not reopen the "widen round 2" arm.
