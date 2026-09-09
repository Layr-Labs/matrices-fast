# 0147 — 273a leftovers: gasprod f3, popdyn g4, 4b-lift subtree (no hold / no AmindNorm)

- **Date:** 2026-09-09
- **Base:** tip `83a9250` / source `be9bae1` (273a), hidden **0.843577**, local **0.794121**, worst **1.105 s**
- **This:** local **0.794061** (−0.60 bip), worst **1.078 s**
- **Status:** measured; stacked into [0148](0148-273a-f3-g4-4b-subtree-minl2.md) (second MINL) before submit
- **Files:** `indep_first.rs` (f3/g4 on n≥20k, Amf5 on dt3-band small cores), `mod.rs` (`one_subtree_polish` after 4b accept)

## Why this, not the fail family

v21 `d4a7cbf` and jonathan 313a `3dc7fdc` both failed hidden on the lee4_06 4b-hold. Tight AmindNorm (276a / 302a) failed or rejected 0.00%. skip-METIS (v5/v12b) failed hidden. This tree does none of those.

273a itself is gated metric top-4 + DegP075 on 265a. Its leftover well is: unused set constructors on the n≥20k 1b band, and 0145's "subtree on a 4b-accepted lift, gated off lee4_09/10".

## Movers vs 273a

| matrix | n | nnz | 273a | this | note |
|---|---:|---:|---:|---:|---|
| methanol200 | 11999 | 76128 | 0.7246 | **0.7204** | 4b-lift subtree |
| gasprod_sarawak81 | 22536 | 75636 | 0.9135 | **0.9112** | f3/amd 1b, then subtree |
| popdynm200 | 22407 | 105584 | 0.9532 | **0.9518** | g4/ddnsw 1b |
| pinene200 | 19995 | 97990 | 0.8357 | 0.8357 | n≥20k keeps g4 off |
| lee4_06/09/10 | | | unchanged | | no hold, no skip-METIS |
| pooling_digabel19 | 514 | 5340 | 0.8330 | 0.8330 | extra tiny-n sets reverted (1b re-roll) |

## Closed on this tree

- Tiny-n BFS/desc/x3 on n≤2000: pooling_digabel19 0.8330→0.8399 (digabel 1b band).
- Double 4b-subtree: methanol200 0.7204→0.7245 (reverted).
- 4b-subtree on n=2508 torsion50: +296 flops (just outside hydro 1800..=2500).
- x3 n≤10k and extra DegP125/ddnw15/SqPure n≤10k: zero movers.
- AmindNorm, skip-METIS, lee4_06 hold: not shipped.
