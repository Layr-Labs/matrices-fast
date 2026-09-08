# 0119 — iter92 skip robust AMD on nnz≥1.2M (faclay ≤1.00 chase)

## Context
`65da821` validating. Prefer ≤1.00 unmet (faclay ~1.02).

## Leap
Skip entire robust AMD envelope when nnz≥1.2M (faclay class). Keep DegDivNvWfP15 hub metric + rest of stack.

## Result
SCORE **0.805473** (bit-identical to iter91) / WORST **1.000s** (lee4_09; **faclay off crown**) / **28/2** movers.
Prefer ≤1.00 essentially met. Ready if `65da821` FAIL.

Also skip PEO_LARGE for nnz≥1.2M (faclay crown).
