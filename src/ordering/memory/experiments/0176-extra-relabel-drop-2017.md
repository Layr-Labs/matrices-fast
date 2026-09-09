# 0176 — drop extra-relabel 20/17 conjunct

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (board best 0.84349; local tip 0.793834)
- **Score:** 0.793834 → **0.793837** (+0.03 bip)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

Extra relabel tickets currently fire only when the incumbent is well below AMD (`best_flops * 20 < amd_flops * 17`) or on the small-graph alternative (`n <= 1000 && nnz <= 30000`) that exists only to satisfy that conjunct. The public proxy has many `n < 6000`, `nnz <= 50000` rows with ratio >= 0.85 that never draw those tickets. Dropping only that conjunct admits them at the existing 8-ticket tier (16 still only when `best_flops * 5 < amd_flops * 4`). Caps stay `n < 6000` and `nnz <= 50000`.

Tip proxy (final flops, `/tmp/yukon-run-tip-74b6ccd.log`): 49 rows in the gate already below AMD fail the integer 20/17 test and are not covered by the small OR. Newly admitted set is not empty, so yukon was run. Checkpoint ratio is at least the final ratio, so those rows also fail 20/17 at the extra-relabel site.

## What changed

`src/ordering/mod.rs` extra_relabel predicate only. Removed:

`best_flops.saturating_mul(20) < amd_flops.saturating_mul(17) || (n <= 1_000 && nnz <= 30_000)`

After the edit: `amd_flops > 0 && best_flops < amd_flops && nnz > 0 && n < 6000 && nnz <= 50000`.

Restart count unchanged: 16 if `best_flops * 5 < amd_flops * 4`, else 8. `EXTRA_RELABEL_MAX_N` / `MAX_NNZ` untouched.

Predicate restored after the miss. HEAD remains `74b6ccd`.

## Result

Yukon local **0.793837** vs tip rebaseline **0.793834** (`/tmp/yukon-run-0176.log`). Fill 0.9258. Buckets unchanged: lt_1k 0.8875, 1k_10k 0.8398, gt_10k 0.6891.

Flops-exact vs tip: **0 better / 1 worse / 299 same**.

Inside `n<6000` and `nnz<=50000`: **0 better / 1 worse**.

Worse: `powerflow0118p` n=5065 nnz=18730, 117447 → 117596 (+149). Ratio 0.967 → 0.968. No outside-gate movers.

Short of score `< 0.793734` (got +0.000003). Gate has no improvement and one regression. Timing not probed. Not submitted. Predicate edit reverted.

## Why it won / lost

The newly admitted near-tie band did not pick up a better relabel ticket. The only movement is a regression on `powerflow0118p`, which sits in the admitted band (n=5065, nnz=18730, tip ratio 0.967). Extra 8-ticket relabel draws on a near-AMD incumbent can replace a later-stage path with a slightly worse portfolio winner. Do not drop the 20/17 conjunct again without a tighter accept or a block on this row.

## Follow-ups

- Do not resubmit this drop. Keep the 20/17 well-below gate.
- Do not raise `EXTRA_RELABEL_MAX_N` or `MAX_NNZ`.
- Do not raise the restart count to 12.

## Links

- Extra relabel well-below gate from [0061](0061-margin-scaled-leftover-search.md) / [0063](0063-time-margin-by-structure.md)
