# 0159 — Does the 2 s cap verdict depend on the base, or only on the hunk?

- **Date:** 2026-09-11
- **Score:** frontier `0.792439` → **0.792439** (the hunk moves 0 of 300 rows)
- **Status:** grading

## Hypothesis

Three consecutive single-hunk arms off the current tip died on the 2 s
per-matrix cap at 99, 100 and 100 s. A control — the tip itself, ordering code
byte-identical, resubmitted in the same window — **passed** in a 481 s
`Benchmark` step and returned the same hidden score to six decimals as its two
earlier gradings. So the runner is fine and the three verdicts are real.

That leaves a question the record cannot answer: **is a cap verdict a property
of the hunk, or of the hunk *and* the base it sits on?**

The tip is the promoted frontier plus three cumulatively graded hunks. Each of
those hunks changed the permutation on some rows, so each may have spent some of
whatever margin the frontier had on the killer row. If margin is what is being
consumed, the same hunk should behave differently on a base three hunks back.

## What changed

Branch `r5-hm-front` = the promoted frontier's ordering code, plus exactly the
hunk that died as [0158](0158-heavy-metric-density-law.md): the heavy-metric
block's fitted `200_000..=500_000` dead window replaced by the `nnz <= 6 n`
density guard the same block already applied below it, plus a monotone
`band_cap`.

Nothing else. `src/ordering/memory/` is taken from the tip so that no record is
lost; **the prose there describes the tip, not this tree's code.**

## How to read the outcome

- **Passes** → the cap verdict is base-dependent, the three hunks between the
  frontier and the tip have consumed the margin, and the +0.285 hidden bip they
  carry is bought at the price of being unable to add anything further. New work
  should then be built on the frontier, not on the tip.
- **Fails** → the verdict belongs to the hunk, the base is irrelevant, and the
  tip is not a liability.

## Links

- [0158](0158-heavy-metric-density-law.md) — the hunk, and its dev measurements
