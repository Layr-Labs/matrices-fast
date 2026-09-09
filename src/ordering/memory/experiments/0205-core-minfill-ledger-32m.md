# 0205 — CORE_MINFILL_LEDGER 16M→32M on 647a tip

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / worst ~**1.138s**)
- **Promote bar:** hidden ≤**0.843073**; local well past ~0.792300 OR ≤ tip_local−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s multi-probe
- **Status:** **MISS / REVERTED** (no submit). Tree restored to tip `CORE_MINFILL_LEDGER=16_000_000`. 647a transplant stack left intact.
- **Model / harness:** Grok / Grok Bot

## Hypothesis

647a tip already ships residual-core exact MinFill (0091) gated on core size only
(`8 ≤ cn ≤ 4000 && core_nnz ≤ 30_000`) with a shared per-row deficiency-word ledger of
**16M**, strict exact-core accept. Jon’s 647a tip note flags **CORE_MINFILL ~32M (655 side ready)**
as the next orthogonal NEW BASE. Doubling the ledger may convert additional truncating cores
without touching TRANSPLANT_LEDGER / widths / admit / FINAL_FIVE_OPS=128M.

## Change (src/ordering/ only; reverted)

```rust
const CORE_MINFILL_LEDGER: i64 = 32_000_000; // was 16_000_000
```

647a stack kept intact throughout (TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8];
below-anchor OR sparse-large near-AMD; FINAL_FIVE_OPS=128M). No K4 / extrarelbl / KaHIP / Scotch.

## Results

Uncapped `probe_timing_and_score` (`SSI_ALLOW_UNSANDBOXED_WORKER=1`, REPEAT=1):
`/tmp/probe-timing-0205.log`

| metric | tip 647a claim | 0205 (32M) |
|---|---:|---:|
| SCORE | 0.792300 | **0.792300** (Δ 0) |
| WORST order() | ~1.138 s | **1.454 s** (lee4_09) |
| lt_1k / 1k_10k / gt_10k | — | 0.8874 / 0.8397 / 0.6854 |

Crown (0205): lee4_09 **1.454s**, lee4_10 **1.430s**, nuclear104 **1.384s**, lee4_06 **1.384s**.
Tip appendix crowns were lee4_10 1.138 / lee4_09 1.135 — ledger doubling blew lee4 prefer envelope.

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 | **FAIL** (identical 0.792300) |
| ≥10 better / 0 worse | **FAIL** (score-flat; no geomean movement) |
| uncapped worst ≤1.14s | **FAIL** (1.454s) |

`yukon run` skipped (timing + score gates already failed). No submit.

## Decision

**REVERT** `CORE_MINFILL_LEDGER` to **16_000_000**. Leave 647a tip intact.
Extra 16M words buy **no** public geomean movement on this corpus and push lee4 crowns
~0.3s past the 1.14s prefer line. Orthogonal minfill-ledger deepen on tip is closed at 32M
without a gate cut that also recovers score (none observed).

## Blockers / next

- 32M ledger: timing-closed on lee4 without score gain.
- Do **not** raise TRANSPLANT_LEDGER toward 2M (651a timing-failed).
- Next invent: other Jon orthogonal directions (INDEP_WORK ledger, completion n-gate) or
  a **conditioned** CORE_MINFILL fire (0091 standing caveat: spends on 193/300 for 6 winners)
  rather than a raw ledger double.
