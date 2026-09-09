# 0210c — Conditioned terminal re-transplant + tighter large-row skip (n<12k, nnz<80k)

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip **0.792300**, WORST **1.560s** same-window `/tmp/probe-timing-tip-647a-0210.log`)
- **Prior:** 0210 submit `9b18a070` **FAILED hidden n/a (timing)**; 0210b submit `c401dd13` **FAILED hidden n/a (timing)**. Do not stack on either.
- **0218:** **ABORTED** stack-on-0210b (0210b failed; no submit). Tree restored to tip before 0210c.
- **Status:** PROBE PENDING
- **Model / harness:** Grok / Grok Bot

## Hypothesis

0210/0210b second `refine_with_donors` is score-positive on mid rows but still timing-toxic on large n/nnz under the hidden 2s cap (both failed n/a timing). Keep the conditioned terminal re-transplant, but **fire only when `n < 12_000` AND `nnz < 80_000`** — tighter than 0210b's 30k/200k. Ledger/widths/AMD-tie/FF128M unchanged. No matrix-ID gates.

## Change (`src/ordering/mod.rs` only)

From tip `62654a5` + 0210 isolate shape:

1. After phase-14: snapshot `transplant_entry_flops` + clone `donor_perms`.
2. Sync `best_flops` on `subset_window_descent_step` accept.
3. Before return: second `refine_with_donors` iff
   `best_flops < transplant_entry_flops && !donor_perms.is_empty() && n < 12_000 && nnz < 80_000`.
4. Admit only on bijection + strict `score < best_flops`.

## Results

Probe pending: `/tmp/probe-timing-0210c.log` vs tip `/tmp/probe-timing-tip-647a-0210.log`.

| criterion | bar |
|---|---|
| SCORE vs tip | beat tip (target ≤ tip−1e-4) |
| movers | ≥10 better / 0 worse |
| worst | ≤ tip same-machine 1.560s |

## Decision

Pending probe.
