# 0112 — iter85 chained residual-core LNS from improved plateaus

## Context
iter84 dual+widen submitted as `71b8941` validating (0.805493 / 1.026s / 27/3).
`dc1f98b` FAIL. Chase 0.849495.

## Leap
After core-exact streams + pair descent + simplicial improve a core, run **2 lean chained LNS streams** (budget/4 each) seeded from the new plateau — not from the AMF floor. Distinct seeds `0x600DCAFE…` / `0xBADC0FFE…` XOR shot rot.

## Why breakthrough
Dual-shot alone was +1 syn mover (micro). Chained search explores the **basin around a proven improvement**, a different lottery than another random start. Still off-danger only; only paid when a stream already won.

## Target
New movers beyond iter84; worst prefer ≤1.05; ≥15 vs tip.

## Result
SCORE **0.805491** / WORST **1.018s** / movers **27/3** vs tip.
vs iter84: **+syn15m04m only** (−0.07% further) — micro extend. Prefer ≤1.05 met.
Hold submit until `71b8941` resolves; invent broader leap if needed.
