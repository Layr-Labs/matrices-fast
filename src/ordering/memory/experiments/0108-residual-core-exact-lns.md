# 0108 — residual-core exact LNS with work ledger (iter66)

## Dropped (thin cluster)
Lex-BFS / Sloan densify / tie-triggered extra relabel / lt_1k exact-stream densify.
That cluster maxed ~7 movers.

## Leap
Once-per-row **exact randomized-greedy LNS on the residual core** after reduce,
bounded by `CORE_EXACT_LEDGER` (80M) with per-call cap 60M. Gates:
`n<8k`, `cn∈[80,2500]`, `core_nnz≤18k`. Multiple streams on small cores;
adjacent-pair polish when LNS moves. Strict exact-core improve.

Orthogonal to: full-matrix exact search, core MinFill (deficiency greedy),
METIS densify, AMF/AMD portfolio.

Kept: METIS densify n<3k (nuclear), EXTRA densify, lt_1k local refine.

## Result
(pending)
