# 0210b — Conditioned terminal re-transplant + large-row second-refine skip

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip **0.792300**, WORST **1.560s** same-window `/tmp/probe-timing-tip-647a-0210.log`)
- **Prior:** 0210 local KEEP 0.791908 / 17/0 / WORST 1.518 — submit `9b18a070` **FAILED hidden n/a (timing)**. Do not stack on failed package.
- **0212:** **ABORTED** before probe (would stack residual-core MINL on failed 0210). Edits reverted; not measured.
- **Status:** **KEEP / SUBMITTING**
- **Model / harness:** Grok / Grok Bot

## Hypothesis

0210’s second `refine_with_donors` on the finished incumbent is score-positive on mid rows but timing-toxic on large n/nnz (hidden kill). Keep the conditioned terminal re-transplant, but **skip the second refine when `n ≥ 30_000` OR `nnz ≥ 200_000`**. No matrix-ID gates. Ledger 1M / widths / sparse AMD-tie / FINAL_FIVE_OPS 128M unchanged.

## Change (`src/ordering/mod.rs` only)

From clean `/tmp/0210-package/mod.rs` (0210 isolate):

1. Keep phase-14 snapshot: `transplant_entry_flops` + clone `donor_perms` from `runner_up`.
2. Before `return best_perm`: fire second `refine_with_donors` iff
   `best_flops < transplant_entry_flops && !donor_perms.is_empty() && n < 30_000 && nnz < 200_000`.
3. Admit only on bijection + strict `score < best_flops`.

## Results

Probe vs tip: `/tmp/probe-timing-0210b.log` (same-window tip `/tmp/probe-timing-tip-647a-0210.log`).

| metric | tip same-window | 0210 (failed) | **0210b** |
|---|---:|---:|---:|
| SCORE | 0.792300 | 0.791908 | **0.791911** (Δ −0.000389 vs tip) |
| WORST order() | 1.560 s | 1.518 s | **1.485 s** (≤ tip) |
| lt_1k / 1k_10k / gt_10k | 0.8874 / 0.8397 / 0.6854 | 0.8874 / 0.8397 / 0.6844 | 0.8874 / 0.8397 / **0.6845** |
| vs tip movers | — | 17/0 | **22 better / 0 worse** / 277 same |

Only loss vs 0210: **crudeoil_pooling_dt3** (n=30660≥30k) — second refine skipped; tip-identical on that row. Large-skip is the intended trade for timing headroom (worst also dropped vs 0210).

| criterion | result |
|---|---|
| SCORE ≤ tip−1e-4 or well past | **PASS** (0.791911 ≤ 0.792200; −0.000389) |
| ≥10 better / 0 worse | **PASS** (22/0) |
| worst ≤ tip same-machine | **PASS** (1.485 ≤ 1.560) |

## Decision

**KEEP / SUBMIT.** One yukon. Package under `/tmp/0210b-package/`.

## Links

- Prior fail: [0210](0210-conditioned-terminal-retransplant.md)
