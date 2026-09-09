# 0206 — Conditioned CORE_MINFILL fire gate (half envelope) on 647a tip

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / worst ~**1.138s**)
- **Promote bar:** hidden ≤**0.843073**; local well past ~0.792300 OR ≤ tip_local−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s multi-probe
- **Status:** **MISS / REVERTED** (no submit; no yukon). Tree restored to tip gate `cn≤4000 / core_nnz≤30_000`, ledger **16M**. 647a transplant stack left intact.
- **Model / harness:** Grok / Grok Bot

## Hypothesis

0205 closed raw `CORE_MINFILL_LEDGER` 16→32M (score-flat 0.792300, worst 1.454s).
Scoreboard locked 0206: keep ledger **16M**, tighten fire only to
`8≤cn≤2000 && core_nnz≤15_000` (half tip envelope) so lee4-class / large residual
cores skip exact minfill spend while small-core exact wins remain.

## Change (src/ordering/ only; reverted)

```rust
const CORE_MINFILL_MAX_CN: usize = 2_000;       // was 4_000
const CORE_MINFILL_MAX_CORE_NNZ: usize = 15_000; // was 30_000
// CORE_MINFILL_LEDGER remains 16_000_000
```

Strict exact-core accept unchanged. 647a stack intact throughout
(TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8]; sparse-large AMD-tie;
FINAL_FIVE_OPS=128M). No K4 / extrarelbl / 488a / TRANSPLANT→2M / ticket swaps.

## Results

Uncapped `probe_timing_and_score` (`SSI_ALLOW_UNSANDBOXED_WORKER=1`, REPEAT=1):
`/tmp/probe-timing-0206.log`

| metric | tip 647a claim | 0206 (half gate) |
|---|---:|---:|
| SCORE | 0.792300 | **0.793107** (Δ **+0.000807** worse) |
| WORST order() | ~1.138 s | **1.476 s** (lee4_09) |
| lt_1k / 1k_10k / gt_10k | — | 0.8874 / **0.8424** / 0.6854 |

Crown (0206): lee4_09 **1.476s**, lee4_10 **1.475s**, nuclear104 **1.440s**, lee4_06 **1.333s**.
1k_10k geomean rose vs 0205's tip-flat 0.8397 → **0.8424** — half-envelope skips
mid-band cores that were converting under the tip 4k/30k gate.

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 | **FAIL** (0.793107 worse) |
| ≥10 better / 0 worse | **FAIL** (net regress; no keep path) |
| uncapped worst ≤1.14s | **FAIL** (1.476s) |

`yukon run` **skipped** (timing + score gates already failed). No submit.

## Decision

**REVERT** fire gate to tip `CORE_MINFILL_MAX_CN=4_000` /
`CORE_MINFILL_MAX_CORE_NNZ=30_000`. Ledger stays **16M**. Leave 647a tip intact.

Conditioned half-gate **regresses** public geomean (loses mid-core exact wins) and
does **not** pull lee4 crowns under 1.14s. Abandon further CORE_MINFILL deepen /
condition on this tip.

## Blockers / next

- CORE_MINFILL deepen axis closed on 647a tip: raw 32M ledger (0205) score-flat +
  timing-fail; half fire gate (0206) score-regress + timing-fail.
- Do **not** raise TRANSPLANT_LEDGER→2M; do **not** retry raw 32M ledger.
- **Next (Scoreboard):** 0207 `INDEP_WORK_LEDGER` 8M→16M (parent may dispatch).
