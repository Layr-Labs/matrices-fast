# Per-matrix work ledgers (how to gate an expensive path)

## What it is

A **work ledger** replaces a flat structural limit (`if nnz <= K`) with a per-matrix budget in
abstract work units. Each iteration of an expensive path computes its own cost from the pattern,
checks it against the remaining budget, and either pays and runs or ends the loop. The budget is a
compile-time constant; the cost is a pure function of `(n, nnz, Lnnz, ...)`, so the result stays
deterministic and the two-runs-must-agree gate is unaffected.

This is the primitive that survived hidden validation in this repository. Flat limits did not.

## Why flat limits keep failing here

Three hidden validation failures share one shape: *a limit was widened so larger inputs entered an
expensive path, with nothing bounding what the largest such input could cost.* The clearest case is
`peo_extract::MAX_LNNZ` raised 300_000 -> 650_000 (submission `9efba180`): dev improved, two movers,
no regressions, and the graded run breached the 2 s cap.

A flat cap asserts *"anything under this size is free."* That is only true if the dev corpus
contains the worst case at that size, and it does not — the dev corpus reaches n ~ 340_000 and the
hidden corpus is rotated and unseen. A ledger asserts something weaker and checkable: *"whatever
this matrix is, it will not spend more than B units here."*

## How to build one (enough to implement)

1. **Instrument the loop.** Print, per iteration, every structural term you might charge plus the
   measured wall time. Set the budget effectively unbounded and run one corpus sweep. Keep it
   test-only; the shipped constants are compiled in.
2. **Fit the cost law by least squares** on those times. Report the residuals. Do not reuse a
   weighting fitted on a different loop or a different corpus slice.
3. **Pick the currency by its worst over-run**, `max(measured / predicted)`, not by residual
   spread — a budget is a bound on the tail. The two are not the same choice: on the terminal-PEO
   chain the unweighted control `n + nnz + Lnnz` has the *best* spread (sd(log) 0.429) and the
   *worst* tail (4.43x), while the fitted `40 n + 2 nnz + Lnnz` has 0.538 and **1.74x**.
4. **Check what the budget implies term by term.** `40 n + 2 nnz + Lnnz <= 4_500_000` implies
   `n <= 112_500` **and** `nnz <= 2_250_000` **and** `Lnnz <= 4_500_000`. If any term is unbounded,
   the ledger is a flat limit wearing a disguise. This check also identifies which of the older flat
   constants have become unreachable and are now only allocation guards.
5. **Refuse before you pay.** Split the cost into the part knowable before the expensive prefix runs
   (usually `n` and `nnz`, before any symbolic factorization) and the part that needs it. Gate the
   loop entry on the cheap part alone, so a matrix that cannot afford one iteration stops *before*
   paying for the measurement that would have told you so. On the dev corpus this alone returned
   216 ms with no score change whatsoever, all of it on the five largest rows.
6. **Charge cumulatively, per iteration.** Fixing an iteration count from the first iteration's size
   under-counts whenever the work per iteration is non-increasing, and over-counts whenever it is
   not. A running total is both simpler and tighter.

## Cost profile vs the cap

The budget converts directly to time once the currency is fitted: at 18.04 ms per million units, a
4_500_000-unit ledger is ~81 ms nominal and ~141 ms at the worst over-run observed — two orders
below the 2 s cap, and bounded for inputs that appear in no local corpus as much as for those that
do. Compare the same chain with no ledger: dev 0.825734 at a **2.907 s** worst call, i.e. failing.

## The trap to know about

A ledger bounds *time*, so it can only cost *score* through its entry test. Within the loop the
ledger just stops early. But an entry refusal turns a whole matrix away, and if the cost law weights
a term heavily then rows that are large in that term lose whatever they would have gained. Under
`40 n + 2 nnz + Lnnz` at 4.5M nothing above n = 112_500 enters at all. That is the intended safety
property — the fitted law says `n` is the expensive term, so large-`n` refusal is correct on time
grounds — but it is also the one way the design can give score back on a corpus you cannot see.
State it, and keep the entry test in the same currency as the ledger so the two cannot disagree.

## Where it is used in `src/ordering/`

| site | currency | budget |
|---|---|---|
| terminal PEO, both branches (`peo_round_cost`) | `40 n + 2 nnz + Lnnz` | `PEO_LEDGER = 4_500_000` |
| reduce extra cores (`REDUCE_EXTRA_CORE_LEDGER`) | core nnz, cumulative | 300_000 |
| reduce clique pairs (`REDUCE_PAIR_BUDGET`) | pair checks | 1_000_000 |
| `minfill_order` | pair checks, fails closed to degree order | 40_000_000 |
| `completion::refine_limited` | credits | 2M / 6M / 8M by stage |

Note the pattern in the last two: an exhausted budget must leave a **valid** result, not an
abandoned one. `minfill_order` appends the remaining vertices in ascending degree order; the PEO
chain simply keeps the incumbent permutation.

## Links

- Experiments: [0080](../experiments/0080-peo-re-extraction-above-the-gate.md) introduced the
  terminal-PEO ledger, [0081](../experiments/0081-oversize-in-gate-peo-ledger.md) extended it to
  in-gate oversize rounds and recorded the `9efba180` flat-limit failure,
  [0082](../experiments/0082-peo-ledger-measured-cost-law.md) fitted the cost law and added the
  entry refusal.
- Technique: [best-of portfolio](best-of-portfolio.md) — why a budgeted candidate can never lower
  the score, only cost time.
