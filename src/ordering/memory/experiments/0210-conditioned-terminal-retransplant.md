# 0210 — Conditioned terminal re-transplant isolate (Scoreboard LOCKED)

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / same-machine worst ~**1.483s** from tip-647a-0209)
- **Promote bar:** hidden ≤**0.843073**; local well past 0.792300 OR ≤ tip−0.0001; ≥10 better / 0 worse; worst ≤ tip same-machine (prefer ≤1.14)
- **Status:** **FAILED hidden n/a (timing)** — submit `9b18a070`; see [0210b](0210b-conditioned-retransplant-large-skip.md)
- **Model / harness:** Grok / Grok Bot

## Hypothesis

Scoreboard LOCKED 0210 (abort MINL invent): after phase-14 freezes the donor pool, ~6 replacing stages (MINL, late polish, FINAL_REFINE + rebuilds, simplicial/pair, terminal five/four/triple, completion, subset-window) can replace the incumbent. A second ledger-bounded `refine_with_donors` on the **finished** incumbent — fired only when those stages strictly improved AND the phase-14 donor pool was non-empty — reaches assemblies the first pass never saw. Same tip ledger 1M / widths [4096,512,128,32,8] / sparse AMD-tie / FF 128M; `transplant_probe.rs` untouched.

## Change (`src/ordering/mod.rs` only)

1. After phase-14 `refine_with_donors`: `transplant_entry_flops = best_flops`; clone `donor_perms` from `runner_up` when non-empty.
2. Immediately before `return best_perm` (after subset_window stages): if `best_flops < transplant_entry_flops && !donor_perms.is_empty()`, call `refine_with_donors` again on finished incumbent.
3. Admit only on bijection + strict `score < best_flops`.
4. Also sync `best_flops` on the subset_window_descent_step accept (so the condition sees that stage's gains).

Never unconditional. Do **not** raise `TRANSPLANT_LEDGER`. Do **not** widen gates. Do **not** touch MINL/LNNZ/n-caps. OPS 128M; no ledger 2M; no K4.

647a stack kept intact: TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8]; sparse AMD-tie; FINAL_FIVE_OPS=128M.

## Results

Same-window tip baseline: `/tmp/probe-timing-tip-647a-0210.log` (SCORE **0.792300**, WORST **1.560s**).
Change probe: `/tmp/probe-timing-0210.log`.

| metric | tip same-window | 0210 |
|---|---:|---:|
| SCORE | 0.792300 | **0.791908** (Δ −0.000392) |
| WORST order() | 1.560 s | **1.518 s** (≤ tip) |
| lt_1k / 1k_10k / gt_10k | 0.8874 / 0.8397 / 0.6854 | 0.8874 / 0.8397 / **0.6844** |
| movers | — | **17 better / 0 worse** |

Top movers: lee4_06 −0.0097, chp_shorttermplan2d −0.0094, lee4_09 −0.0087, gabriel09 −0.0056, methanol200 −0.0017, sporttournament18 −0.0017 (+11 micro).

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 | **PASS** (−0.000392) |
| ≥10 better / 0 worse | **PASS** (17/0) |
| worst ≤ tip same-machine | **PASS** (1.518 ≤ 1.560); prefer ≤1.14 miss on hot box (tip also 1.560) |

## Decision

**FAILED hidden n/a (timing)** on `9b18a070`. Local KEEP metrics stand; do not stack further work on this package — pivot [0210b](0210b-conditioned-retransplant-large-skip.md).

## Decision

**KEEP / SUBMIT.**
