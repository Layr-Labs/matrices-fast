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
