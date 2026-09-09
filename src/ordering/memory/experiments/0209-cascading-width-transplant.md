# 0209 — Cascading width-ladder transplant (rebuild etree after commit)

- **Date:** 2026-09-09
- **Base tip:** `62654a5` / jonathan308 647a @ hidden **0.843173** (local tip ~**0.792300** / worst claim ~**1.138s**)
- **Promote bar:** hidden ≤**0.843073**; local well past ~0.792300 OR ≤ tip_local−0.0001; ≥10 better / 0 worse; uncapped worst ≤1.14s (or ≤ tip same-machine)
- **Status:** **MISS / REVERTED** (no submit; no yukon). `transplant_probe.rs` restored to tip. 647a stack left intact.
- **Model / harness:** Grok / Grok Bot

## Hypothesis

647a’s width ladder `[4096,512,128,32,8]` scores every width against the **frozen entry**
incumbent tree. After a coarse width commits an improving assembly, finer widths still
transplant onto the old postorder. **Cascade:** on each strict width-level commit, rebuild
`base`/`parent`/`base_counts` from the committed assembly (one ledger unit) and continue
finer widths on that tree. Same `TRANSPLANT_LEDGER=1M`, same widths, same below/sparse-AMD-tie
admit — pass structure only (not ledger climb).

## Change (`src/ordering/transplant_probe.rs` only; reverted)

In `transplant_pass`, after `f < best_f` assembly admit: charge one unit, recompute
postorder/parent/counts from assembled perm, set `cascade_f = f`, continue ladder so finer
widths do **not** score against the frozen entry etree.

647a stack kept intact throughout (TRANSPLANT_LEDGER=1M; widths [4096,512,128,32,8];
sparse AMD-tie; FINAL_FIVE_OPS=128M; CORE_MINFILL 16M@4k/30k; INDEP_WORK 8M). No K4 /
ledger 2M / 0205–0208 families.

## Results

Tip same-machine uncapped probe: `/tmp/probe-timing-tip-647a-0209.log`
SCORE **0.792300**, WORST **1.483 s**.

0209 cascade uncapped probe: `/tmp/probe-timing-0209.log`

| metric | tip same-machine | 0209 cascade |
|---|---:|---:|
| SCORE | 0.792300 | **0.792271** (Δ **−0.000029**) |
| WORST order() | 1.483 s | **1.494 s** (lee4_10; Δ +0.011) |
| flops-exact movers | — | **13 better / 6 worse / 281 same** |

Better (top): lee4_06 −24007, arki0002 −5494, chimera_mgw-c16-2031-01 −4809, lee2_06 −4284, …
Worse: lee1_07 +33367, chimera_rfr-02 +3019, multiplants_stg5 +1872, chp_partload +358, …

| criterion | result |
|---|---|
| SCORE well past 0.792300 OR ≤ tip−0.0001 (≤0.792200) | **FAIL** (0.792271) |
| ≥10 better / 0 worse | **FAIL** (13/6) |
| uncapped worst ≤1.14s OR ≤ tip same-machine | **FAIL** (1.494 > tip 1.483) |

`yukon run` **skipped**. No submit.

## Decision

**REVERT** cascade in `transplant_probe.rs`. Leave 647a tip intact.
Cascading width-ladder alone is thin + non-monotone on public flops and does not clear the
keep bar. STOP — do not invent another tip-appendix knob; next orthogonal is parent/Scoreboard
(MINL upper-n/LNNZ per CoS).
