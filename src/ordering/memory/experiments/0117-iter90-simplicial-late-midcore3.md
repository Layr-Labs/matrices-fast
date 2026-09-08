# 0117 — iter90 cheap simplicial late + mid-core 3rd stream

## Context
`7503ec1` still validating. iter89 faclay AMF cut bit-identical / no new movers / worst still ~1.03. Prefer ≤1.00 unmet. Need HARDER leap: more movers.

## Leap
1. Keep faclay AMF α-1 skip (nnz≥1.2M) — flops-neutral timing hope
2. Revert null cheap 8-stream
3. **Simplicial promotion** after cheap late polish (cost≤8M, 4M budget) — new move type
4. Mid-core exact streams cn∈(900,1500] 2→**3**

## Target
>28 movers vs tip; worst prefer ≤1.00 if possible else ≤1.05; no micro-bip.

## Result
SCORE **0.805473** / WORST **1.020–1.024s** / 28/2; +2 deepen vs 87. Prefer≤1.00 unmet.
