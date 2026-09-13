# 0227 — deferred-lift parity: polish the held stage-1b lift, compare like with like

- **Date:** 2026-09-12 (iter48)
- **Score:** production frame, one binary, one session, 4-vCPU (`taskset -c 0-3`), 300 dev rows:
  parity OFF **0.791586** (worst 1.397 s) -> parity ON **0.791480** (worst 1.333 s);
  exact `COUNTS` diff **6 rows better / 1 worse / 293 identical**, delta **-1.06e-4 (-1.34 bips)**.
- **Status:** shipped (`indep_parity_off() -> false`; the deferred-lift polish is production), official
  local harness 300/300 at **0.791480 / 0.924039** (`results.tsv:1789261328`), submitted
  **41baf1ce-12f8-4d2b-8aa3-0233a62215a0** (validating). Supersedes `e07fe7ae` (9 windows) —
  same tree plus the parity device.
- **Page:** [0227-deferred-lift-parity.md](0227-deferred-lift-parity.md)

## Hypothesis

The 4b acceptance rule compares a **raw** candidate (the lift's score as taken at stage 1b)
against a **polished** one (the portfolio incumbent, after 2.descent + 3.search + the 4.subtree
chain). The forced arm (`n >= 20_000`) hands the *lift* that polish instead. If that asymmetry is
what the two frames disagree about (0218 measured the pair at 0.791635 / 0.791851 on the same
binary), then giving the held lift the identical polish and keeping the better of the two
*polished* candidates should recover the forced frame's value on the deferred rows — and it can
never be worse than the raw rule, because the polish is monotone and the polished lift is <= the
raw lift.

## What changed

`src/ordering/mod.rs`: the pre-terminal polish (2.descent + 3.search + the 4.subtree chain,
~530 lines) became one unit, `macro_rules! pre_terminal_polish`, expanded in the ordinary
position (incumbent) and again inside the 4b block when the raw lift wins. The three
`SIMPLICIAL_PROMOTION_*` consts and the `pair_descent_*` bindings that the region declared but
later stages also use were hoisted out of the macro; everything else is unchanged text, so the
second pass is byte-identical to the first. `SSI_INDEP_PARITY=0` (test-only) restores the raw
rule; the graded build compiles `indep_parity_off() -> false`.

## Result (all arms production frame `SSI_INDEP_FORCE=1`, same binary/session/4 vCPU)

| arm | SCORE | worst `order()` | buckets |
|---|---:|---:|---|
| parity OFF (shipped before this change) | 0.791586 | 1.397 s | 0.8873 / 0.8377 / 0.6852 |
| parity ON (**shipped now**) | **0.791480** | 1.333 s | 0.8874 / 0.8377 / 0.6849 |
| gate removed, parity ON ("parity everywhere") | 0.791480 | **2.162 s** | identical |

Exact per-row movers: methanol200 -0.894 %, crudeoil_lee4_10 -0.760 %, torsion50 -0.197 %,
graphpart_3g-0244-0244 -0.185 %, crudeoil_lee4_09 -0.061 %, glider400 -0.019 %,
**wastewater05m1 +0.39 %**. `yukon run` reproduces the probe to 4 decimals; suite 123/0/55.

## Two structural results that come with it

1. **Parity makes the force gate value-neutral on dev, row for row.** Removing
   `INDEP_FORCE_MIN_N` (gate off everywhere) with parity on is **0 rows different** from the
   gated arm (`probe-diff`: `improved 0 / regressed 0`, score identical to 4 decimals). The gate
   is therefore no longer doing any *value* work — but it is still load-bearing for wall clock:
   without it the worst rows become `crudeoil_lee4_09` **2.162 s**, `gabriel10` 1.859 s,
   `arki0016` 1.822 s, `acopf_case9241pegase_qcqp` 1.760 s (all > the 1.33 s worst of the gated
   arm, two of them past the 2 s cap). Keep the gate; it is a cost gate, not a value gate.
2. **The one loss is a *path-dependence* receipt, not a metric artifact.** On
   `wastewater05m1` (n=98, nnz=536) the trace reads
   `PARITY n=98 raw=8771 inc=8828 plift=8111 margin_ppm=81218`: the polish makes the lift 8.1 %
   better than the polished incumbent, parity adopts it, and the row still ends worse
   (8033 vs 8002 flops). The phase dump shows why: the terminal ladder took the raw rule's
   incumbent 0.6874 -> 0.6845 (-0.42 %) but the parity winner 0.6872 -> 0.6872 (-0.00 %) — the
   window-descent ladder is a path-dependent local search, and the "better" start is the worse
   basin. Consequence: a "run the terminal tail from both candidates" variant would recover only
   this one row (~7e-6), so it is NOT worth its cost — the open lead that pointed at it is
   closed by this receipt rather than by a build.
   Instrument: `SSI_PARITY_TRACE=1` (test-only) prints `PARITY n=... raw=... inc=... plift=...
   margin_ppm=...`.

## Why it won / lost

It won because it removes the pipeline's only *asymmetric* comparison: from here on, whichever
candidate stage 1b holds gets the same stages in the same order before the acceptance decision,
so the 1b gate choice can only matter through the search's path, not through an unequal budget.
It lost nothing measurable: 293/300 rows are bit-identical, and the single regression is bounded
(0.39 % on one small row) and understood.
