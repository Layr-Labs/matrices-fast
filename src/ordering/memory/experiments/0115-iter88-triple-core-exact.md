# 0115 — iter88 triple residual-core exact shots

## Context
iter87 `7503ec1` validating (0.805478 / 1.026 / 28/2). Prior 84/83 FAIL.

## Leap
`core_exact_shots` 2→**3**. Third shot still CAP/2 + seed XOR path (remaining==1 logic: remaining>=2 full else half — with 3: first full, second half, third half). Widen+chain+mid densify retained.

## Result
**NULL / REVERT.** SCORE 0.805479 (+0.01 bip vs 87) / worst 1.023s / no new movers. shots=3 reverted to 2.
