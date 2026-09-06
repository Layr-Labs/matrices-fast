# 0082: the sparse-large band of the extra-depth reductions

- **Model:** Claude Opus 5 (1M context)
- **Harness:** Claude Code with the agentprivacy dual-agent harness (matrices_mage instance)
- **Date:** 2026-09-06
- **Base:** `a2614af` (submission `1a4183b7`, hidden 0.859834)
- **Status:** measured, all gates green; official pending

## The gap

The extra-depth reductions run behind two bands: small graphs, and dense mid-to-large graphs at
`nnz >= 6 * n`. Between four and six times the dimension sits a class that qualifies on nonzeros and
fails on density, receiving no extra depth at all: transswitch2736spr (4.75), transswitch2383wpr
(4.64), powerflow2736spr (4.16), powerflow2383wpr (4.06).

## Why the earlier null result did not survive

Relaxing the same floor was tried before and read as a null: the rows were admitted and nothing moved.
That was measured before the terminal chains existed. An extra depth leaves a different residual core,
and the terminal machinery now on this tree works on that different core. The reduction alone does not
pay; the reduction feeding the terminal stages does.

## The change

One constant, one clause, and **no budget raised**. A sweep confirmed it: at the existing pair budget
(1,000,000) and core ledger (300,000) held-out is 0.850365, and at 2,000,000 / 750,000 it is also
0.850365. The gain is in admitting the class, not funding it.

## Measurement

| corpus | base `a2614af` | candidate | better | worse |
| --- | --- | --- | --- | --- |
| development (300) | 0.827195 | **0.826709** | 2 | 0 |
| held-out (591) | 0.852139 | **0.850365** | 1 | 0 |

Gains: transswitch2736spr 6.71% and transswitch2383wpr 0.32% on development, powerflow2736spr 9.86%
out of sample. 68 tests pass; the trusted 300-case run completes at 0.8267, fill tiebreak 0.9374.

Interleaved minimum-of-ten, alternating base and candidate on exactly the fourteen rows the predicate
admits: **no row slower by more than 0.05 s**. powerflow2736spr +0.032, powerflow2383wpr +0.043,
transswitch2383wpr +0.038, transswitch2736spr +0.031. The corpus worst call is on rows this change
never reaches; an interleaved run over the 32 slowest rows put the candidate at 1.278 s against the
base's 1.311 s.

## The measurement that nearly went both ways

A first pass compared against a base table captured 50 minutes earlier and reported the gain row at
+0.092 s, a clear rejection. It was contamination: rows outside the admitted class had moved as much,
and the host had picked up load. Re-measured interleaved, the true cost is +0.032 s. Three repeats
put the borderline row at +0.042 and +0.058 on either side of the limit; ten repeats settled it at
+0.043. Single-run timings on this host are hints, never verdicts, in both directions.
