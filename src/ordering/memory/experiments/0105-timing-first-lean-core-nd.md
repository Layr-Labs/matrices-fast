# 0105 — timing-first lean residual-core ND (iter58)

## Context
iter56 stacked medium+2 (3k–8k) + fat once-per-row core-ND (n<8k, cn≤3k, core_nnz≤25k, METIS switches/imb/seeds/combined + Scotch×3 + KaHIP Fast/Eco/seeds) + full EXTRA densify n<3k → local **0.805843 (−4.00 bip), 11/0 movers**, but full timing **WORST 1.855 s** (crudeoil_lee1_07, arki0016, chimera/chp/nuclear/mpbp…). Submit bar requires ≤1.10 s.

iter57 once-per-row core quotient metrics (SqDiv/DegSqrt/…) under same n<8k gate → **NULL** (identical score/movers). Not a breadth lever.

## Leap (not a micro-bip)
- Drop medium+2 entirely (revert to tip 4-ticket exact-search floor).
- Keep residual-core ND mechanism; harden gates to `n<4500 && nnz≤18k && cn∈[80,2000] && core_nnz≤12k`.
- Cut tickets ~3×: METIS default + switches {50,150,300,800} + imb 0.15; Scotch default + one seeded trials; KaHIP Fast only.
- Remove iter57 metrics block.

## Hypothesis
Timing killers were (a) medium+2 on crudeoil-class and (b) fat ND ticket storms on 5–8k / high-cn cores. Lean gates keep nuclear/ringpack/digabel-class wins if they fit the envelope; breadth to ≥15 must come from a NEW cheap-band family afterward — not densify/ticket inches.

## Result
(pending iter58 yukon + full timing probe)
