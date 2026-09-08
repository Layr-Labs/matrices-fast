# 0099 — Disagreement-gated residual MinFill (pricing)

- **Date:** 2026-09-07
- **Score:** no production change (probe-only pricing on `e88316d`, dev 0.806560)
- **Status:** PRICE-FAILS, nothing shipped. Probe-only; `mod.rs` non-test paths untouched.

## Hypothesis

Cores where the portfolio's own degree-family passes already agree are easy
and MinFill cannot beat them, so firing exact MinFill only where those passes
disagree buys the same winners while spending on a fraction of rows.

## What changed

Probe-only: new `#[ignore]` test `probe_disagreement_distribution` in
`src/ordering/probe.rs` (+285 lines). No production decision changed
(`mod.rs` diff remains the pre-existing test-only `observe_skip` hook under
`#[cfg(test)]`). Per admitted residual core the probe records the exact
5-pass spread (`rel = (max-min)/min`) versus the MinFill win/gain, plus
call-site search ms and charged words — timed at the call, never differenced
out of two 300-row `order()` timings.

## Result

300-row dev on `e88316d`: 573 captures → 419 admitted, 193 touched rows
(matches 0091), 200 core-level wins. Total MinFill spend is already tiny:
558 ms corpus, S1=0 / S2=0 / S3=10.5 ms against 20/8/100 ms allowed.

| fire iff | kept cores | win retention | call cut | charged cut | ms cut |
|---|---|---|---|---|---|
| rel > 0 (any) | 357/419 | 190/200 = 0.95 | 0.148 | 0.042 | 0.051 |
| rel ≥ 0.001 | 351/419 | 187/200 = 0.935 | 0.162 | 0.044 | 0.054 |
| rel ≥ 0.01 | 335/419 | 174/200 = 0.87 | 0.200 | 0.077 | 0.099 |
| rel ≥ 0.02 | 319/419 | 170/200 = 0.85 | 0.239 | 0.117 | 0.130 |
| rel ≥ 0.05 | 273/419 | 139/200 = 0.695 | 0.348 | 0.151 | 0.179 |
| rel ≥ 0.10 | 225/419 | 106/200 = 0.53 | 0.463 | 0.247 | 0.282 |

Lost at T=0.01 includes the 4th-largest winner (`waterund14`, rel=0.0099)
and a zero-spread S3-row win. Sorted-name halves: base wins 75 vs 125;
retention at >0 holds (0.947 vs 0.952) but diverges above 1%
(0.05: 0.813 vs 0.624; 0.10: 0.627 vs 0.472).

## Why it lost

The signal is free — admitted `cn ≤ 4000 ≤ 8000`, so `results[]` already
holds exact `flops_of` for all five passes at the MinFill decision, no extra
scoring needed — but it does not separate. 62 zero-spread cores hold 10 wins
(agreement ⇏ easy); win-rel median 0.105 vs loss-rel median 0.130 overlap;
no threshold retains winners at << spend.

## Follow-ups

- Disagreement-as-gate: closed on this evidence. Do not build.
- Option 1 middle-band / K=7+ design questions unchanged, for a bigger block.

## Links

- [0091](0091-residual-core-exact-minimum-fill.md) (shipped pass + call-site timing method)
- Research queue: [open-questions](../open-questions.md)
