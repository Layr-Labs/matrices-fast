# 0079 — PEO alternate seeds 4 → 6 under the shared ledger

- **Date:** 2026-09-06
- **Score:** 0.826784 → 0.826736 (−0.58 bip dev; fill 0.937368 → 0.937367)
- **Status:** WIN locally (2 movers / 0 regressions / 298 ties); submitted.

## Hypothesis

e5310ad's own data: its 2-seed variant scored 1.74 of the 4-seed 4.1 bip —
seeds are the paying axis of the alternate-seed PEO mechanism. Six retained
seeds cost nothing structural: stalled seeds spend one round each and the
shared 4M ledger plus entry-fee skip cap total work exactly as before.

## What changed

`src/ordering/mod.rs`: `PEO_ALT_SEEDS` 4 → 6 (+ comment). One constant.

## Result

Full trusted 300-matrix run:

| bucket | before | after |
|---|---:|---:|
| `lt_1k` | 0.8897 | 0.8897 |
| `1k_10k` | 0.8631 | 0.8631 |
| `gt_10k` | 0.7522 | 0.7522 |

(bucket prints round to 4dp; exact weighted means 0.826784 → 0.826736).
Per-matrix diff over the 300-row tables: 2 strict movers, 0 regressions:

- `sonet21v6` 0.9840 → 0.9650
- `syn40hfsg` 0.9680 → 0.9670

## Robustness read (honest)

Two movers is thin, and 0004's lesson (single-matrix wins flip across halves)
applies in spirit — but this is not a new lottery policy, it is two more
deterministic seeds through the already-promoted chain under the same ledger.
No new gates, no new timing exposure (ledger-capped), strict `<` admission.
Submitted for hidden arbitration; a rejection is data on hidden conversion
of thin dev wins.

## Follow-ups

- If hidden confirms: try seed-source widening (the e5310ad author's stated
  next step — plumb direct-set stages into `runner_up`), NOT a larger ledger
  (their 8M variant scored the same and hit 1.82 s).
- If hidden rejects: keep 4 seeds (revert) — retention by official evidence.

## Links

- Research queue: [open-questions](../open-questions.md)
