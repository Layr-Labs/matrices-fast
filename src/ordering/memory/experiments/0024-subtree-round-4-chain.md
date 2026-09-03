# 0024 — Subtree round-4 chain: keep widening the window

- **Date:** 2026-09-03
- **Score:** 0.851642 → **0.851347** (−2.95 bips; fill 0.948894). Buckets:
  lt_1k 0.896482 (unchanged) · 1k_10k 0.878657 → **0.878037** · gt_10k
  0.797749 → **0.797477**. Worst order() 0.901–0.918 s across probe runs.
- **Status:** submitted.
- **Parent:** `567d605` (our 0023 round-3 chain, promoted hidden 0.877373).

## Hypothesis

The four-round chain — round 1 (32×1M round=0), round 2 (24×1M round=1,
ms384), round 3 (32×1M round=1, ms512) — still yields on each fresh
incumbent. Round 3 found −6.0 bips over the two-round chain (probe) and
−3.2 bips on the hidden corpus after promotion, so the mechanism is not
saturated. A fourth chained pass over the round-3 incumbent, with the window
widened again (max_s 768), should keep paying.

## Sweep (measured on the shipped round-3 chain)

| round-4 config | dev score | d vs shipped | worst order() |
|---|---|---|---|
| (no round 4) | 0.851642 | — | 0.904 s |
| 24 blocks, ms768 | 0.851374 | −2.68 bips | 0.918 s |
| **32 blocks, ms768** | **0.851347** | **−2.95 bips** | 0.901 s |
| 24 blocks, ms1024 | 0.851497 | −1.45 bips | 0.908 s |
| 24 blocks, ms640* | 0.851660 | +0.2 bips | — |

\* = round-2 widened to ms640 as well — WORSE; widening round 2 changes the
round-2 incumbent and the later chain lands on a worse local optimum. Do not
reopen.

max_s 1024 overshoots (blocks too large spend the 1M budget shallowly);
max_s 768 is the sweet spot so far. 32 blocks ≈ 24 blocks (+0.27 bip) at the
same window.

## Change

Chain a fourth `rgreedy::subtree_refine` inside round 3's acceptance
(round=1, 32 blocks × 1M, min_s 16, max_s 768), strict best-of. Same
deterministic bounded-work chain; accepted path for every other gate is
byte-identical.

## Result

Full-corpus run: **0.851347** (results.tsv row 1788414337) — matches the
probe aggregate exactly. Movers again concentrate in the subtree-chain
families (pooling/mpbp/powerflow/slay/transswitch/unitcommit-class).

## Learning

Chain gains compound because each round re-ranks fresh subtrees of a better
tree; per-round yield is holding (r3 −6.0 bips, r4 −2.95 bips) as the window
widens. Round-count × max_s is the tuning surface: (2, 384) shipped by
hybridnoise, (3, 512) shipped here as 0023, (4, 768) ships here. Each round
adds ≤32M requested work only on matrices whose previous rounds improved, so
worst-case time has stayed ~0.90 s through four rounds.

## Round-5 extension (same session, on the round-4 chain)

A fifth pass (round=1 first, then round=3) was measured on the round-4 code:

| round-5 config | dev score | d vs round-4 |
|---|---|---|
| (round-4 only, shipped as a09a467d) | 0.851347 | — |
| 16 blocks, ms768, round=1 | 0.851294 | −0.53 bips |
| 24 blocks, ms768, round=1 | 0.851279 | −0.68 bips |
| 24 blocks, ms768, round=3 | **0.851247** | **−1.00 bips** |
| r4 round=2 + r5 round=3 | 0.851259 | — (worse; round-2 on r4 hurts) |

round=3 on the fifth pass helps (diversified later-round seeds incl. n≥10k —
see rgreedy seed rules). Full run: **0.851247** (results.tsv row, 2026-09-03).
Yields are flattening (r3 −6.0, r4 −2.95, r5 −1.0 bips); the chain is near
its end as a promotion vehicle.

## Post-chain polish + round-5 combo (same session)

The subtree chain runs AFTER order()'s only adjacent-pair descent, so
chain-improved incumbents were never locally polished across block
boundaries. Rerunning the shipped descent (same gates, plus n>=1000) on the
final incumbent improved 21 matrices (+0.4 s corpus-total; batchs121208m
-0.41%, chimera_selby, mpbp_46/47, sfacloc, crudeoil_li05, nuclear25a,
pooling_sppa0pq, crudeoil_pooling_ct1...).

Full run of round-5 (round=3, 24b, ms768) + post-chain polish on the round-4
code: **0.851181** (results.tsv, 2026-09-03; buckets 0.8965/0.8777/0.7973) —
-1.66 bips vs the shipped round-4 code (0.851347). Worst order() ~0.92 s.

## POST-CHAIN PIPELINE POLISH (the win after the chain topped out)

The chain cannot go past round 4: round 5 (round=3, 24b, ms768) FAILED the
hidden eval at the 2.0 s per-matrix cap ("order() exceeded the 2.0s
per-matrix cap and was killed"). Cap on chain depth = 4 rounds.

Pipeline-ordering gap: adjacent-pair descent, simplicial promotion and the
medium exact search all run BEFORE the subtree chain, so the chain's
improvements were never re-polished. Rerunning all three once on the final
chain-band incumbent (n>=1000, unchanged gates/budgets, strict best-of):

- pair descent rerun: −0.65 bips (21 matrices)
- simplicial rerun: ~−0.5 bips (13 matrices)
- medium exact rerun: ~−1.7 bips
- **all three together: 0.851347 → 0.851015 (−3.32 bips)**, full run
  2026-09-03; worst order() 0.921 s UNCHANGED on the heaviest matrix
  (crudeoil_lee4_10 n=17,809 is not in the pass bands). 1k_10k bucket
  0.8780 → 0.8769. Band-safe: added work only on n<=12k matrices.
