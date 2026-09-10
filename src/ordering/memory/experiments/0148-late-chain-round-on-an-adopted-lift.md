# 0148 — Subtree chain on a lift adopted at 4b: closed, and it is not a timing story

## The lead (0145's own follow-up)
> "with acceptance after subtree the lift misses the subtree pass. Running
> subtree on an accepted lift would cost 0.13-0.16 s on the slowest rows — not
> affordable there; could be gated to rows under ~0.6 s."

Two things to establish: which rows actually reach that branch, and whether this
box can pay for them.

## Step 1 — census (`probe_indep_4b_census`, all 153 rows with n >= 1000)
Test-only counters at the 4b adoption site (`probe_counters::INDEP_4B_WINS` /
`_LOSSES`) plus per-row `order()` timing.

| row | n | nnz | order() | headroom to cap |
|---|---:|---:|---:|---:|
| `pooling_sppa9pq` | 5030 | 120730 | 0.419 s | 1.58 s |
| `glider400` | 10017 | 49624 | 0.421 s | 1.58 s |
| `torsion50` | 2508 | 29808 | 0.537 s | 1.46 s |
| `procurement1large` | 14416 | 41068 | 0.850 s | 1.15 s |

`FOURB_WIN_ROWS = 4`. Deferred lifts that *lose* at 4b: `waste`, `popdynm25`,
`space25`, `chimera_lga-01`, `mpbp_35` — untouched by this change. Corpus worst
row is `crudeoil_lee4_09` at 1.39-1.47 s, and it is not in the set.

So the gate the note worries about is unnecessary: every row that reaches this
branch has at least 1.15 s of headroom even on this slow box. The affordability
question was the wrong question.

## Step 2 — A/B on exactly those four rows
One stage-4-style ranked round on the adopted lift (`subtree_cfg_for`, plus the
chain's diversified `round = 1` retry when the first finds nothing), strict-
decrease acceptance, behind a test-only kill switch so both arms run in one
binary.

| row | base ratio | with late round | delta | time cost |
|---|---:|---:|---:|---:|
| `pooling_sppa9pq` | 0.631198 | 0.631198 | 0.0000% | +0.038 s |
| `glider400` | 0.878110 | 0.878110 | 0.0000% | -0.029 s |
| `procurement1large` | 0.532310 | 0.532310 | 0.0000% | +0.054 s |
| `torsion50` | 0.661830 | 0.663428 | **+0.2414%** | -0.002 s |

Three rows: the chain round finds nothing on the lift, or nothing that survives
to the shipped ordering. One row: it fires and the ordering ends **worse**.

## Why the regression is not a bug
Acceptance inside the round is strictly monotone — it cannot return a worse
ordering than it was given. But everything after 4b is a *search seeded by the
incumbent*, and those stages are not monotone in the final result. A strictly
better input at 4b lands `torsion50` in a different basin, and that basin ends
0.24% worse. This is the same effect the 1b/4b design already encodes in the
other direction (a lift that leads the raw portfolio by 8% ending 22% behind),
and it is the reason the tip defers lifts instead of adopting them early.

REJECTED. Production code is byte-identical to the tip again; the census
counters and `probe_indep_4b_census` are kept because the four-row set is worth
knowing for any future work on this branch.

## What this closes
The 4b branch is not an unpolished ordering waiting for a pass. It is four rows,
three of which the chain has nothing to say about, and one where more polish at
that point is actively harmful. Do not revisit under a different config or a
wider gate: the cost side was never the binding constraint, so tuning the gate
cannot change the outcome.
