# 0204 — the terminal ladder as a *live-winning* device (ratio gate at 90 %)

## Question

`cad51a0f` shipped the four-rung dense ladder alone and came back **failed on the
2.0 s cap** (Benchmark step 111.6 s, gh run 34708974859) while its single-rung
parent completed and promoted. Which per-row price made it unsurvivable, and can
its value be kept while that price is deleted?

## Instruments

* Probe mirroring production (`SSI_INDEP_FORCE=1 SSI_LADDER_STRIDE=1
  SSI_NO_SPARSE_LARGE_TIE=`), one binary, one host session.
* New test-only `FENCETRACE` line: every batch's `n`, `nnz`, incumbent fill and
  task count, plus both fence predicates.
* New `RATIOGATE` line: the row's live `best_flops / anchor_flops` and whether
  the rung list ran.
* Per-row wall-time A/B between two trees on the same host, with the host's own
  repeat floor (+-0.05 s on rows the change cannot touch) used as the noise bar.

## Findings

1. **The fence is blind to the heavy tail**: dev's giants (`acopf…` n=313 068
   fill 3.5e7; `gabriel10` fill 1.4e9; `faclay75` fill 3.2e9) are 3 orders of
   magnitude under the 2e10 fill bound and are 3 of the corpus's slowest rows.
2. **A cost key cannot replace it**: `FENCETRACE` shows those rows run 1-3
   candidate tasks per batch (restart budget `budget/nnz`); 527-task queues exist
   only on n = 30..120 rows. Implemented as `LADDER_COST_NNZ`, left `usize::MAX`.
3. **Per-row time is a plateau**: the 20 slowest rows span n 7 993..313 068 and
   all sit at 0.83-1.19 s; ~25 gates of 10-240 ms each. `faclay75` gains nothing
   after its portfolio (0.9405 -> 0.9405) yet spends 0.47 s more.
4. **Static near-baseline stops are not free**: cutting after `1b.indep` at
   ratio >= 0.98 costs +0.00297 dev (98 rows, 19.8 s saved); after `4.subtree`
   >= 0.98, +0.00191 for 8.4 s. Late stages rescue rows at ratio 1.0
   (`nuclear104` 1.0000 -> 0.7593).
5. **The ladder pays only where the row is already being won**: of 187 rows whose
   time moved by >0.005 s (+10.7 s), 10 changed value and all had ratio <= 0.86
   when the rungs ran; the dearest additions in the cap-relevant class
   (`arki0016` +0.099, `mpbp_15` +0.108, `mpbp_07` +0.073, `powerflow0300p`
   +0.056) were at ratio 0.91-0.96 with zero flop gain.

## Shipped

`SHIPPED_LADDER_EXTRA` restored, gated by `LADDER_RATIO_PCT = 90` (rungs run only
while `best_flops <= 0.90 x anchor_flops`, anchor = snapshot after the first
scored batch; empty list below the gate, so the shipped rung is removed there
too). Sweep: pct 100 = 0.792226 (wiring control, exactly the frontier), 95 =
0.792259, 90 = 85 = 0.792166.

| build | SCORE | Σ order() |
|---|---|---|
| frontier | 0.792226 | 125.0 s |
| four rungs, ungated (killed `cad51a0f`) | 0.792110 | 135.6 s |
| **four rungs + gate 90 %** | **0.792166** | **115.8 s** |

Official sandboxed harness (production path): 300/300 OK, **0.792166 / 0.924495**
(buckets 0.887462 / 0.838890 / 0.685651) vs the promoted 0.792226 / 0.924450.
Largest per-row increase vs the frontier: +0.110 s on a 0.44 s row; among rows
>= 0.5 s the max is +0.077 s and the class total is **-1.99 s**.

## Logs

`0204-r1-control-fencetrace.log`, `0204-full-phases.log`, `0204-phase-heavy12.log`,
`0204-ratio{85,90,95,100}.log`, `0204-L4-ratio{0,85,90}.log`,
`0204-harness-ratioladder.log`, `0204-submission-note.md`.
