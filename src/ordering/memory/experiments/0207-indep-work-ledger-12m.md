# 0207 — INDEP_WORK_LEDGER 8M→12M on 647a tip

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / worst ~**1.138s**)
- **Promote bar:** hidden ≤**0.843073**; local well past ~0.792300 OR ≤ tip_local−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s multi-probe
- **Status:** **MISS / REVERTED** (no submit; no yukon). Tree restored to tip `INDEP_WORK_LEDGER=8_000_000`. 647a transplant stack left intact. **Abandon INDEP_WORK deepen** (no 16M / 0207b).
- **Model / harness:** Grok / Grok Bot

## Hypothesis

Scoreboard timing tweak: raise only `INDEP_WORK_LEDGER` **8M→12M** (not 16M) to buy more
indep-first work without touching the 647a transplant stack. Prior CORE_MINFILL deepen
(0205 raw 32M, 0206 half-gate) closed timing/score-negative. Intermediate 12M tests whether
INDEP_WORK deepen is timing-safe on lee4 crowns before any 16M follow-up.

## Change (src/ordering/ only; reverted)

```rust
const INDEP_WORK_LEDGER: u64 = 12_000_000; // was 8_000_000
```

647a stack kept intact throughout (TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8];
sparse AMD-tie; FINAL_FIVE_OPS=128M; CORE_MINFILL_LEDGER=16M with tip gate cn≤4k /
core_nnz≤30k). No mid_force / extrarelbl / chimera / Scotch / ticket / TRANSPLANT→2M /
other indep_first gate changes.

## Results

Uncapped `probe_timing_and_score` (`SSI_ALLOW_UNSANDBOXED_WORKER=1`, REPEAT=1):
`/tmp/probe-timing-0207.log`

| metric | tip 647a claim | 0207 (12M) |
|---|---:|---:|
| SCORE | 0.792300 | **0.792360** (Δ **+0.000060** worse) |
| WORST order() | ~1.138 s | **1.564 s** (nuclear104) |
| lt_1k / 1k_10k / gt_10k | — | 0.8874 / 0.8397 / 0.6855 |

Crown (0207): nuclear104 **1.564s**, lee4_09 **1.440s**, lee4_10 **1.435s**, lee4_06 **1.319s**.
12M blows lee4 prefer envelope (~1.14s) and slightly regresses public geomean.

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 | **FAIL** (0.792360 worse) |
| ≥10 better / 0 worse | **FAIL** (gabriel10 ratio 0.9285→**0.9377**; probe emitted 299/300 rows, missed slay06m; no keep path) |
| uncapped worst ≤1.14s | **FAIL** (1.564s) |

`yukon run` **skipped** (timing + score gates already failed). No submit.

## Decision

**REVERT** `INDEP_WORK_LEDGER` to **8_000_000**. Leave 647a tip intact.
**Abandon INDEP_WORK deepen** on this tip — do not chain 16M / 0207b.
Parent next invent: completion n-gate / MINL retune (structural family), not more ledger bumps.

## Blockers / next

- INDEP_WORK 12M: timing-closed on nuclear104/lee4 without score gain (slight regress).
- Do **not** raise INDEP_WORK→16M; do **not** raise TRANSPLANT_LEDGER→2M.
- Next (Scoreboard): completion n-gate / MINL retune on `62654a5` (parent dispatches 0208).
