# 0217 — Fill-greedy indep-first late best-of (on 0210b KEEP)

- **Date:** 2026-09-09
- **Base:** **0210b KEEP** tree (tip `62654a5` + conditioned terminal re-transplant + skip second refine n≥30k/nnz≥200k). Local 0210b SCORE **0.791911** / 22/0 / WORST **1.485s** (tip same-window 0.792300 / 1.560s).
- **Inventor path:** `/workspace/inventor-queue/0217-fill-greedy-indep-late-bestof/on-0210b/` (NOT tip-only top-level).
- **0210b submit:** `c401dd13` still validating at decision time.
- **Status:** **MISS / REVERTED** (0217 only; 0210b KEEP left intact)
- **Model / harness:** Grok / Grok Bot

## Hypothesis

Fill-greedy independent-set construction (exact missing-neighbourhood / fill rank) is a different basin from production degree-greedy `indep_first::run`. Same `INDEP_WORK_LEDGER=8M` (not a deepen). Late best-of on finished incumbent; AMD-only cores. Stacked Scoreboard 0210b → 0217.

## Change (`src/ordering/` only)

1. `indep_first.rs`: add `run_fill_greedy`.
2. `mod.rs`: late best-of before 0210b second refine; 0210b skip gates unchanged.

## Results

Probe: `/tmp/probe-timing-0217.log` vs `/tmp/probe-timing-0210b.log` / tip `/tmp/probe-timing-tip-647a-0210.log`.

| metric | tip | 0210b | **0217 on 0210b** |
|---|---:|---:|---:|
| SCORE | 0.792300 | 0.791911 | **0.791911** (Δ 0 vs 0210b) |
| WORST order() | 1.560 s | 1.485 s | **1.737 s** (> tip & 0210b) |
| movers vs 0210b | — | — | **0 better / 0 worse** (null) |

## Decision

**MISS / REVERTED.** Did not beat 0210b SCORE; worst regresses 1.737 > 1.485/1.560. Restored `/tmp/0210b-package/mod.rs` + tip `indep_first.rs`. 0210b package untouched. No submit. One yukon not used (miss before submit).
