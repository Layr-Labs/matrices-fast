# 0095 — Terminal completion-lattice descent (MINL), ported from the SSI challenge

**Date:** 2026-09-07. **Base:** the 0094 tree (dev 0.810282; hidden −0.6 bips as
`2e3c00fb`, rejected under the 1-bip bar).
**Result (A/B of the stage alone, on `017a036` + 0094):** dev 0.810282 → **0.809866**
(−5.6 bips), 11 rows better / 0 worse; on the tip `aa5b471` (`f311d19` adds only the α grid this tree already carried) the whole tree lands at
**0.808268** vs the tip's 0.812247 (−49.0 bips);
worst same-box `order()` 1.523 s; the stage costs ≤ 0.1 s on any row, 3.8 s
over the corpus (2-vCPU pod).

## What it is

`minl.rs`: the finished incumbent's filled graph is minimalized by greedy
fill-edge deletion under the exact local chordality test (`N(u) ∩ N(v)` is a
clique), cheapest-first, every operation charged to one op budget (40M) with a
hard break; the minimal completion is realized by its MCS perfect elimination
order and by AMD on it; either is admitted only on a strict exact decrease.
Gates: nnz < 700k (above it the old challenge measured zero removals), fill
≤ 1.5M (checked before the filled graph is built), n ≥ 16. Placed LAST.

The crown's `minl_watch` (completion watcher, 0067) walks the same lattice
witness-driven with 8M credits; the systematic scan finds more on the mpbp /
rsyn / chimera / transswitch families.

## Movers (dev)

`mpbp_35` 0.3345 → 0.3229, `mpbp_34` 0.3158 → 0.3097, `rsyn0820m04m` 0.8342 → 0.8190, `chimera_selby-c16-01` 0.7012 → 0.6980, `transswitch2383wpr` 0.9810 → 0.9803;
then `mpbp_15`, `mpbp_21`, `lop97icx`, `kan_r3_h1_n4`, `chimera_mgw-c16-2031-01`, `rsyn0810m04m` by under 0.1 bip each.

## Placement experiment

An EARLY copy of the stage (before the exact search and the subtree chains,
where the old challenge ran it) was measured as a separate variant: +0.5 dev
bips (20 better / 23 worse: the exact search and subtree chains diverge from a
different incumbent), +2 s corpus time, +0.06 s on the slowest row. Dropped;
the stage is terminal only.

## Negative results this round

- Relabelled quotient-metric restarts (DegSqrt/SqDiv α10 × 4 seeds, hub-free
  light tier): −8.7 dev bips on three crudeoil_lee4/gabriel rows, worst row
  1.23 → 1.35 s. Not shipped: few-row pattern, cap cost.
- The 0094 pieces graded −0.6 hidden bips (`2e3c00fb`): dev wins on one to four
  giant rows translate at ~0.04.
