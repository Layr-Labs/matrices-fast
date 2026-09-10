# 0149 — Skip indep-first when the portfolio ties AMD (KILLED: +0.42 bips)

- **Date:** 2026-09-10
- **Score:** tip dev **0.792300 → 0.792342 (+0.42 bips)** via cap-free probe
  (`probe_timing_and_score`, no 2 s gate) → REVERTED same session.
- **Status:** hypothesis false. Do not retry in this form.
- **Files:** `mod.rs` indep gate `+ best_flops < amd_flops` (reverted).

## Hypothesis

PHASES data showed nuclear104 spending 0.29 s in `1b.indep` for zero score
(portfolio and indep both leave it tied at 1.0; its wins come from reduce +
transplant). When dozens of diverse portfolio families tie AMD, one more
diversity ticket looked like the longest shot — skip the lift there and bank
0.1–0.3 s on hot rows to fund transplant depth (the substitutive loop).

## Result

+0.42 bips regression. The lift DOES win on some portfolio-tied rows
(presumably the force bands: digabel 400–1000, hydro 1800–2500, gasprod ≥20k,
mid_force 8–20k&nnz≥50k — 0143 put those bands in precisely because the lift
wins there). Best-of-floor logic was correct about mechanism but wrong about
scope: "would not have won" is false for band rows.

## Consequences

1. The cut program is abandoned, not refined: a band-carved variant (skip iff
   tied AND outside force bands) might be dev-null, but its savings land on
   rows whose depth budget is timing-blocked anyway — the substitutive loop
   does not close. Margin without a spend is not a product.
2. The force bands are load-bearing: do not touch indep gating without a
   full-corpus score to prove null.
3. Methodology win: `probe_timing_and_score` (no cap kill) measures exact dev
   deltas under arbitrarily bad box load. All future scoring under load uses
   probes, never `yukon run` pass/fail.

## Census sidebar (same session)

`probe_census` on the six juiciest headroom rows (powerflow0118p, faclay30,
syn30m03m, arki0002, nd_netgen, popdynm200): NO single producer beats the tip
on any of them — best singles sit at or above tip ratios (e.g. powerflow best
single 0.9928 vs tip 0.9666; the gap is composition + polish). Producer
coverage is exhausted there; remaining headroom is search/polish depth only.
Worst single candidates hit 13671x (Scotch on faclay30) — the 0039 verdict
holds.

[Index](../index.md) | [Log](../log.md)
