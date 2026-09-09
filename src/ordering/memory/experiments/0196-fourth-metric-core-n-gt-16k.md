# 0196 — fourth metric core on n>16k under (11,10); drop SqDiv

- **Date:** 2026-09-09
- **Base tip:** `74b6ccd` (local tip 0.793834; official best 0.843490, bar ≤0.843406)
- **Score:** 0.793834 → **0.793849** (+0.000015)
- **Status:** miss / reverted. Not submitted.

## Hypothesis

On `n>16k` the tip admits quotient metrics on the top-3 AMD-ranked cores
only (`METRIC_TOP_CORES = 3`), with the full six-variant list including
`SqDiv`. A near-tied 4th AMD core (within 10% of best: `amd_total * 10 ≤
best_amd * 11`) may still be worth a metric walk, especially once `SqDiv`
is dropped on that band (0168-style SqDiv swaps were noisy; tip six stays
on `n≤16k`).

Pre-check census (phase-1 AMD totals, metric envelope): public 4th-core
class under (11,10) is **non-empty** — `supplychainr1_053050`, `gams05`.
Proceeded to yukon.

## What changed

`src/ordering/indep_first.rs` only. Reverted after the miss. HEAD remains
`74b6ccd`. `COMPETITIVE_MARGIN` stayed `(3, 2)`. `REDUCE_EXTRA_DEPTHS`
stayed `[5,4,2,6]`. No set admit, no DegP125, no DegDivNvWfP15 swap.
`probe.rs` / `mod.rs` untouched.

1. `n≤16k`: tip bit-identical — metric top-4, six variants including `SqDiv`.
2. `n>16k`: metrics on top-3 AMD cores as tip, **plus** the 4th AMD-ranked
   core only when `amd_total * 10 ≤ best_amd * 11`.
3. On every `n>16k` metric core (including the first three), drop `SqDiv` —
   variants `DegDivNvSqrtWf, DegPlusDegme, DegSqrt, DegDivNvDegme, DegP075`.

## Result

Yukon local **0.793849** vs tip **0.793834** (`/tmp/yukon-run-0196.log`).
Fill 0.9258→0.9258. Buckets: lt_1k 0.8875 / 0.9599, 1k_10k 0.8398 / 0.9468,
gt_10k 0.6892 / 0.8844.

Flops-exact vs tip (`/tmp/yukon-run-tip-74b6ccd.log`): **0 better / 1 worse /
299 same**.

n>16k movers: **0 better / 1 worse** (`pinene200` 1134819→1137637).

4th-core class hits (`supplychainr1_053050`, `gams05`): **tip-flat** (no flop
change). `pinene200` precheck had only 3 AMD cores, so the regression is from
dropping `SqDiv` on the top-3 n>16k metric cores, not from admitting a 4th.

Keep bar unmet (score did not beat tip; not >1 n>16k movers; 1 worse).
Uncapped worst not probed. Not submitted. Metric admit + variant list restored
to tip.

## Why it won / lost

The (11,10) 4th-core gate fires on two public rows but does not change their
best-of (those 4th cores are not the ranking pass). Paying for the SqDiv drop
on every n>16k metric core costs `pinene200` a few thousand flops and lifts
the geomean. Do not resubmit this package; do not drop SqDiv on n>16k without
a compensating win.

## Follow-ups

- Leave tip `metric_k=3` / six-variant list alone on n>16k.
- A 4th-core-only package (keep SqDiv on top-3) is still open but the two
  class hits were flat here, so expect null unless a new gate finds a
  different class.

## Links

- Prior: [0179](0179-ndivwfp15-on-n-gt-16k.md), [0168](0168-0148-sqdiv-degp125-gasband16k.md)
