# 0165 — DegSqrt into the heavy-metric 4-ticket prefix

- **Date:** 2026-09-09
- **Score:** not run (no edit)
- **Status:** aborted before yukon. No source change. Not submitted.

## Hypothesis

Same-count swap on `HEAVY_METRIC_ORDER`: drop the last ticket `extra_deg2_div_nv_wf002` / 10.0 and put DegSqrt at the same dense_alpha 10.0, if the heavy-metric sweep already accepts that variant name.

## What changed

Nothing. `src/ordering/mod.rs` was not edited. `SUBTREE_CHAIN_MAX_N` stays 45_000. HEAD remains `74b6ccd`.

Confirmed DegSqrt is **not** already in the 4-ticket prefix. Current list:

1. `extra_deg2_div_nv_wf05` / 10.0
2. `cm_sqpure` / 5.0
3. `extra_deg2_div_nv_wf05` / 2.5
4. `extra_deg2_div_nv_wf002` / 10.0

The heavy-metric dispatcher accepts only `cm_sqdiv`, `cm_sqpure`, and names present in `metric_sweep::EXTRA_METRICS`. There is no `cm_degsqrt` arm, and DegSqrt is not an `EXTRA_METRICS` spec. A string swap would be a silent no-op (unknown name falls through and is skipped). Adding a new match arm would invent a caller, which this experiment forbids.

## Result

No yukon run. Tip local stays 0.793834. No movers.

## Why it won / lost

Not tested. The sweep does not already name DegSqrt. DegSqrt already runs as a `ScoreVariant` on other tickets (light-tier α 1 / 5 / 2.5, core metric passes); that is not a legal `HEAVY_METRIC_ORDER` spec.

## Follow-ups

- Do not invent `cm_degsqrt` just to fill this slot.
- A later heavy-metric swap must use a name the existing match already dispatches (`cm_sqdiv`, `cm_sqpure`, or an `EXTRA_METRICS` name), or it is the same no-op.

## Links

- Code: `HEAVY_METRIC_ORDER` and the heavy-metric `match name` in `src/ordering/mod.rs`
- Related: [0092](0092-ported-heavy-tier-metrics-parallel-portfolio.md)
