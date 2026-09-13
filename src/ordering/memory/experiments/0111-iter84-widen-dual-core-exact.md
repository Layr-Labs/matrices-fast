# 0111 — iter84 widen + dual residual-core exact LNS

## Context
Reclaim `dc1f98b` (iter83) validating vs Xo1otl 0.849495. Local 0.805494 / 26/3 / worst 1.082s.

## Leap
1. `core_exact_shots` 1→**2** (second shot: CAP/2 + seed XOR `0x9E3779B97F4A7C15`)
2. Widen admit: cn **≤2200**, core_nnz **≤18k** (was 1500/14k)
3. New band cn∈(1500,2200]: lean **single** stream at budget/2 (timing-first)
4. Danger skip absolute: `!(n≥1800 && nnz≥9k)` unchanged

## Why breakthrough
- Dual shot: second lottery ticket on a family that already produced ≥15 movers + two hidden promotions
- Widen: new residual cores never searched — breadth predictor for hidden translate
- Lean new-band stream + n<12k gate keeps faclay/crudeoil killers out of this path

## Target
Score < 0.805494, ≥15 movers vs tip, worst prefer ≤1.05 (hard ≤1.10). No micro-bip.

## Result
SCORE **0.805493** / WORST **1.026s** / movers **27/3** vs tip; **+syn15m04m** vs iter83; 0 regressions vs iter83.
Prefer ≤1.05 met. Submitting after `dc1f98b` FAIL (not a duplicate).
