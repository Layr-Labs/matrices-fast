# 0211 — Residual-core late polish + best-of floor (0164 follow-up)

- **Date:** 2026-09-09
- **Base:** 0210 KEEP tree on tip `62654a5` / 647a (conditioned terminal re-transplant present; package `/tmp/0210-package/`)
- **0210 local baseline:** SCORE **0.791908**, 17/0 vs tip, WORST **1.518s** ≤ tip same-window 1.560s; submit `9b18a070` validating
- **Status:** **MISS / REVERTED** (0211 only; 0210 KEEP left intact). No submit. No yukon.
- **Model / harness:** Grok / Grok Bot

## Hypothesis

Same late exact-polish streams (`rgreedy::search` + adjacent-pair) on the deg≤3 residual core when `core_lift::reduce` shrinks. Keep the **full-graph ticket as floor**; admit splice only on trusted full `score` strict `<`. Prevents 0164’s 3-worse replacement regressions. Non-transplant; not ledger/gate/budget nudge. Re-probe on 647a+0210 — prior 0170 on `74b6ccd` was −0.49 bip / 6/1 (chimera_lga-01).

## Apply method

`cp /workspace/inventor-queue/0211-residual-core-late-polish-bestof/mod.rs src/ordering/mod.rs` (Inventor-rebased onto 0210 KEEP; verified `cmp` with inventor queue file). Edit only `src/ordering/`.

## Change (`src/ordering/mod.rs` only)

Inside existing iter74 window (`n∈[16,3000) && nnz≤12k && cost≤20M`), after full-graph streams+pair:

1. `core_lift::reduce(scoring_pat, REDUCE_ROW_DEG, REDUCE_MAX_CORE_N, REDUCE_MAX_CORE_EDGES)` (panic-catch).
2. Real shrink only: nonempty prefix, `cn < n`, `cn ≥ 2`, `prefix_flops < best_flops`.
3. Core seed = relative order of `core_ids` in post-floor `best_perm`.
4. Same `late_streams` budgets/seeds + same pair `(d_rounds,d_budget)` on core CSC; internal accepts via `flops_of`.
5. `splice` → bijection → full `score`; admit iff `f < best_flops`.

0210 conditioned terminal re-transplant left intact during apply. No MINL upper-n, no ledger 2M, no K4, OPS 128M.

## Results

Probe: `SSI_ALLOW_UNSANDBOXED_WORKER=1 cargo test -p ssi-candidate-worker --release probe_timing_and_score -- --ignored --nocapture --test-threads=1` → `/tmp/probe-timing-0211.log`.

| metric | 0210 KEEP | 0211 |
|---|---:|---:|
| SCORE | 0.791908 | **0.791859** (Δ −0.000049) |
| WORST order() | 1.518 s | **1.469 s** (≤ 0210 / tip 1.560) |
| lt_1k / 1k_10k / gt_10k | 0.8874 / 0.8397 / 0.6844 | 0.8873 / 0.8396 / 0.6844 |
| vs 0210 COUNTS | — | **5 better / 1 worse** / 293 same |
| vs tip 647a COUNTS | 17/0 | **28 better / 1 worse** / 270 same |

Worse (same as 0170): **chimera_lga-01** 497542 → **498683** (+1141). Better vs 0210: multiplants_stg1 −1135, netmod_kar1 −351, syn15m04m −143, pooling_adhya4pq −83, chimera_mgw-c8-439-onc8-002 −26.

| criterion | result |
|---|---|
| SCORE ≤ 0.791908−0.0001 or clear leap | **FAIL** (−0.000049 only; need ≤0.791808) |
| ≥10 better / 0 worse | **FAIL** (5/1 vs 0210; chimera_lga-01 regress) |
| worst ≤ tip/0210 same-machine | PASS (1.469 ≤ 1.518 / 1.560) |

## Decision

**MISS / REVERTED.** Restored `src/ordering/mod.rs` from `/tmp/0210-package/mod.rs` (7031 lines; 0210 KEEP). No yukon; no submit. Same chimera_lga-01 late-stage interaction as 0170 — floor does not eliminate the tip-visible regression. Residual-core late-polish best-of on 647a+0210 is exhausted in this form.

## Links

- Related: [0170](0170-iter74-core-polish-bestof-floor.md), [0164](0164-late-phase-on-core.md), [0210](0210-conditioned-terminal-retransplant.md)
