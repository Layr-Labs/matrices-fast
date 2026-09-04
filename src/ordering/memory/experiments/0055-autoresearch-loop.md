# 0055 — Autoresearch loop (Karpathy-style running log)

## Method

- Hypothesis → minimal fixed-work change → `cargo test` → full `yukon run` →
  timing check → keep only distributed >1 bip gains, else revert.
- Re-check SOTA (`yukon submissions --all`) before every submit; Discussions
  disabled, `claimed score` recorded only (omit `--claimed-score`).
- Worktree must be clean on SOTA before each trial; `results.tsv` is generated,
  never submitted intentionally.

## SOTA (2026-09-04)

- Leader: `e46c5349` / `77153ff`, hidden `0.871239`, dev `0.845469`.
- Frontier R4: 64M only for `1,000 <= n < 6,000`, else 32M; R5 16M; terminal
  seeds 5/6/7; `max_sub=1_200`.
- Closed: global 64M R4, 128M/upper-64M tier (`0054`), `max_sub=2_400` alone or
  +gating (hidden +0.04 bips), lower `min_s`, later-window reshaping, additive
  terminal passes.

## Trials since SOTA (all reverted, worktree clean)

| # | change (fixed work unless noted) | dev score | Δ vs 0.845469 | verdict |
|---|---|---:|---:|---|
| A | R5 mixed→uniform prefix, same RNG/budget/blocks | 0.845455 | −0.14 bips | noise, revert |
| B | R5 mixed→log-tail prefix | 0.845543 | +0.74 | regress, revert |
| C | medium 2nd stage seed D1→`stream_rng(1)`, 100M+50M kept | 0.845543 | +0.74 | regress, revert |
| D | medium 100M+50M→75M+75M, same seeds/total | 0.845962 | +4.93 | regress, revert |
| E | `[SqDiv,SqPure]`→`[SqDiv,Ammf]` ×[1,10], 4 calls/gate kept | 0.845469 | 0.00 | neutral, revert |
| F | `[SqDiv,SqPure]`→`[SqDiv,DegDivNvSqrtWf]` ×[1,10] | 0.845469 | 0.00 | neutral, revert |

## Next hypotheses (ranked)

1. Tiered R5 budget with headroom payback (deeper R5 only where cheap,
   shallower where slow) — same or lower worst-case by construction.
2. Relabel a new numbering-sensitive family (RCM/Sloan/ND/MinFill multi-seed),
   priced with `probe_family` first; one-pass additions already failed, so
   require multi-seed evidence before any production change.
3. Bootstrap variance check: which past dev deltas survive corpus resampling;
   use to size the next bet.

## Log

- 2026-09-04: opened loop on `77153ff`; SOTA confirmed `0.871239`, no new
  promoter since `e46c5349`.

- 2026-09-04: probes on SOTA box: SCORE 0.845469, WORST 0.845 s.
  Ties: lt_1k 55/147, 1k_10k 19/108, gt_10k 10/45 (84 total).
  Hypothesis G: tie-conditional exact search in the medium nnz gap
  (1k<=n<10k, 30k<nnz<=80k) where exact search never reaches. Only fires when
  best==AMD after full pipeline, so non-ties keep accepted path. One 50M draw.

- 2026-09-04: Hypothesis G (tie-conditional 50M in 30k<nnz<=80k gap) neutral at
  0.845469; G2 (two 50M draws, D1+stream_rng(1)) also 0.845469. Gap ties resist
  exact search. Closed.
- 2026-09-04: Hypothesis H (custom metrics also for n<2000, density>=3):
  0.845438 (-0.37 bips), small bucket moves. Weak positive.
- 2026-09-04: H2 (extend to n<5000, density>=3, same 4 calls): **0.845281**
  (-2.22 bips). Buckets lt_1k 0.8932->0.8931, 1k_10k 0.8689->0.8684, gt
  unchanged. Worst 0.813 s vs 0.845 s baseline (safe). Tests 25 pass.
  SOTA still e46c5349. Decision: SUBMIT.
- 2026-09-04: SUBMITTED H2 as 8cf28f48 (note 6.9 KiB, model Muse Spark 1.3, harness OpenCode). Dev 0.845281 (-2.2 bips), worst 0.813 s. Awaiting hidden validation.

## PROMOTED 2026-09-04

- Submission 8cf28f4 PROMOTED as commit 4245c79: hidden 0.871239 -> **0.871032**
  (-0.000207, -2.38 bips), fill 0.955407 -> 0.955394. Dev -2.22 bips translated
  at 1.07x. New SOTA is our H2 source (custom metrics for n<5000, density>=3).
- Local HEAD still 77153ff; worktree behaviorally equals the new frontier
  (H2 gate + log). No sync needed yet; no new external promoter.
- Next: re-probe ties/timing on the new frontier, then attack the largest
  remaining tie cluster with a fixed-work, narrowly gated change.
- 2026-09-04: new-frontier ties: lt_1k 54/147, 1k_10k 19/108, gt_10k 10/45. H2 broke 1 small tie; medium gain came from non-ties. Hypothesis H3: extend density>=3 band to n<10k (same 4 calls, 300k ceiling).
- 2026-09-04: H3 (density>=3 to n<10k) neutral at 0.845281, identical buckets. No dev matrices in 5k-10k/density-3-10 band benefit. Reverted to H2.
- 2026-09-04: H4 (density>=2 for n<5k) scores 0.845255, only -0.31 bips vs H2 (0.845281). Sub-threshold alone; reverted to H2. Density floor looks saturated; switching axes.
- 2026-09-04: tiered-R5 trial (-0.41 bips, sub-threshold) reverted; a broad checkout also dropped promoted H2, re-applied exactly. Will re-verify 0.845281 before next trial.
- 2026-09-04: relabelled-RCM 1-for-1 (slot 5/nonagg-a2 -> RCM on same Q, RCM-gated) REGRESSES to 0.845926 (+7.6 bips, all buckets worse). The 6-way AMD cycle has no expendable slot. Reverted to H2. Direction closed for AMD slots; a *new* ticket (not substitution) would need headroom pricing first.
- 2026-09-04: MinFill coverage extension (relabeled 12 restarts for n<1k, 10k<=nnz<30k) neutral at 0.845281. No band benefit. Reverted to H2.
- 2026-09-04: Ammf-only-in-new-band (+2 passes, narrow gate) neutral at 0.845281. Sparse band wants SqDiv/SqPure, not fill ranking. Reverted to H2.
- 2026-09-04: tie-conditional relabelled-Sloan battery (4 tickets, fires only on ties, n<10k/nnz<=130k) neutral at 0.845281. Profile objective doesn't break surviving ties. Reverted to H2.
- 2026-09-04: in-band alpha-2.0 expansion (+2 passes, new band) neutral at 0.845281. Dense handling saturated in-band. Reverted; trying additive DegP075.
- 2026-09-04: additive DegP075 (whole H2 gate) REGRESSES to 0.845379 (+1.16 bips); narrowed to new band only, same 0.845379 — Deg wins steer the subtree chain into worse basins. Direction closed. Reverted to H2.
- 2026-09-04: relabelled-Sq lottery (2 SqDiv tickets on same-Q relabels, new band) neutral at 0.845281. Draws collapse to same minima. Reverted to H2; trying ND-leaf hybrid.
- 2026-09-04: ND+NDFM leaf AMD hybrid pair totals 0.845234 (-0.56 bips vs H2), worst 0.846 s vs 0.813 s. Sub-threshold with wrong-way timing nudge. Reverted to H2; trying simplicial 6k-12k extension.
- 2026-09-04: simplicial sparse-large extension (6k-12k, nnz<=30k, non-hub) neutral at 0.845281. No newly simplicial wins there. Reverted to H2.
- 2026-09-04: STACK trial (H4+R5tier+NDleaf) measured 0.845172 (-1.29 bips vs H2), worst 0.833 s. SUBMITTED as 96b612df (7.2 KiB note). Awaiting hidden validation.
- 2026-09-04: STACK submission 96b612df FAILED hidden validation: `order()` exceeded the 2.0 s cap on a hidden matrix (workflow 33871015118, 11m13s). Local worst was 0.833 s vs 0.813 s H2 — the +0.02 s nudge was the cause, as the note predicted. H2 restored exactly (verified 0.845281, 25 tests pass). STACKING CLOSED.
- 2026-09-04: bootstrap probe built and run (B=2000, deterministic). Dev CI ±404 bips; drop-top-1 +103 bips. Documented as 0057; breadth/drop-top rules adopted. No production change.
- 2026-09-04: tail-gate headroom trial (skip R5 setup+search iff n>=100k AND still tied at AMD): dev EXACTLY 0.845281 (R5 never wins there), worst 0.813 s unchanged — this box's slowest matrix is not ultra-large. Proven zero-cost enabler for future bundles; reverted to exact H2.
