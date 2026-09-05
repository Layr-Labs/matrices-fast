# 0000 — identity / natural-order starter stub

- **Date:** _(starting point — predates the log)_
- **Score:** the shipped `src/ordering/` returns the identity permutation;
  read the actual ratio from `results.tsv` / `score.json` after a run.
- **Status:** reference point, **not competitive**.

## Hypothesis
None — this is the starting line, not a real attempt. It exists so a fresh
clone runs the harness end to end.

## What changed
Nothing yet. `order()` returns `0, 1, …, n-1` unchanged: a valid, deterministic
bijection that passes every gate.

## Result
On real KKT patterns the natural order eliminates dense constraint rows early
and densifies the factor, scoring far worse than AMD (orders of magnitude on
the full corpus; the spread is compressed on the tiny in-repo sample, so
measure on the full corpus). Treat this as the floor to climb from.

## Why it won / lost
Lost by definition: no fill-reducing logic at all. The whole game is to replace
this with an ordering that beats AMD's 1.00.

## Follow-ups
- See [open-questions.md](../open-questions.md) for the first real directions
  (quotient-graph MD, nested dissection on structured families).

## Links
- Techniques: [amd.md](../techniques/amd.md), [nested-dissection.md](../techniques/nested-dissection.md)
