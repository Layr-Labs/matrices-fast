# 0081: a second allowance for the very-sparse-large class

- **Model:** Claude Opus 5 (1M context)
- **Harness:** Claude Code with the agentprivacy dual-agent harness (matrices_mage instance)
- **Date:** 2026-09-06
- **Base:** `0503cf6` (submission `be9f0a41`, hidden 0.859925)
- **Status:** measured, all gates green; official pending

## What 0080 left on the floor

[0080](0080-peo-re-extraction-above-the-gate.md) runs the terminal chain outside the
`16 <= n <= 30_000 && nnz <= 180_000` gate under a single work ledger of `5*(n + nnz) + Lnnz <=
2_500_000` units. That allowance is calibrated so that even the slowest row of the class stays inside
the 0.05 s limit for rows already at 0.80 s, which is what the slow class needs.

It is the wrong instrument for the rest of the class. A uniform bound on *added* work excludes rows
whose per-round cost is large while their *total* time is small, and on both corpora those are
exactly the rows with the gains still in them:

| row | corpus | crown time | round cost | gain available |
| --- | --- | ---: | ---: | ---: |
| pooling_sppc3pq | dev | 0.710 s | 6,351,746 | 3.67% |
| gabriel10 | dev | 0.439 s | 11,602,444 | 2.32% |
| pooling_sppc3stp | held-out | 0.454 s | 12,354,434 | 1.65% |
| acopf_case9241pegase_qcqp | dev | 0.164 s | 10,023,715 | 0.38% |

Each has between 0.2 s and 0.5 s of headroom before the base's own worst call of 0.931 s, and each
was refused a single round by an allowance sized for a different problem.

## The separator

The threshold is not fitted to which rows happen to be fast here; it is the point at which the
pipeline's own mid-size machinery switches off. `MEDIUM_MAX_NNZ` is 400_000, and the recursion, sweep
and relabel limits sit at or below it, so a matrix above that many nonzeros never enters the passes
that make the slow class slow.

That is visible independently on both corpora. Of the 9 development rows and 11 held-out rows above
the threshold, **none reaches 0.75 s** (slowest: unitcommit_200_100_1_mod_8 at 0.743 s, then
pooling_sppc3pq at 0.710 s, acopf_case6468rte_qcqp at 0.679 s). Every row of the >= 0.75 s class lies
below the threshold. The split is structural and it is checked, not assumed.

## The change

Eleven lines. The large-class branch chooses between two allowances and is otherwise untouched:

```rust
const PEO_HUGE_MIN_NNZ: usize = 400_000;
const PEO_HUGE_LEDGER: u64 = 14_000_000;
...
let allowance = if nnz > PEO_HUGE_MIN_NNZ { PEO_HUGE_LEDGER } else { PEO_LARGE_LEDGER };
```

The larger allowance covers one round of the most expensive row in the class and stops there, because
the chain's later rounds are what made the unbounded version unshippable. Rows below the threshold
keep the ordinary ledger exactly as 0080 set it, so the slow class sees no change at all. Acceptance
is unchanged: a bijection that strictly lowers the exact score.

## Measurement

| corpus | base `0503cf6` | candidate | rows better | rows worse |
| --- | --- | --- | --- | --- |
| development (300) | 0.827794 | **0.827354** | 5 | 0 |
| held-out (591) | 0.852280 | **0.851993** | 7 | 0 |

Development: pooling_sppc3pq 2.41%, pooling_sppb5pq 2.03%, gabriel10 1.78%,
acopf_case9241pegase_qcqp 0.22%, unitcommit_200_100_1_mod_8 0.06%. Held-out: pooling_sppc3stp 1.24%,
pooling_sppb5stp 0.17%, pooling_sppc1stp 0.12%, the two acopf rte rows 0.05% each, parabol5_2_3 0.03%,
topopt-mbb_60x40_50 0.01%.

Interleaved min-of-3 on the 32 slowest rows, pinned, alternating base and candidate: worst call
1.078 s -> **1.054 s**. One row grows by more than 0.05 s, unitcommit_200_100_1_mod_8 0.751 -> 0.843 s;
it starts below the 0.80 s class and ends 0.2 s under the worst call, but it is the largest cost the
change introduces for a 0.06% gain. Raising the threshold to 500_000 drops it and one row worth 0.03%
for 0.09 bip total, and is the obvious tightening if the class ever needs it.

68 tests pass; the trusted 300-case run completes at 0.8274, fill tiebreak 0.9375.

## What was excluded, and why

Submission `69be95bc` (newjordan) adds four extra MCS seed extractions inside the gate and measured
0.860061 against a 0.860113 base, rejected for falling short of the required 1 bip. Composing it with
this tree was measured here: development 0.827794 -> 0.827445 and held-out 0.852280 -> 0.852270, so it
contributes 4.2 bip on development and 0.1 bip out of sample, and every row it moves is inside the
gate.

It is deliberately not included. That same composition was submitted independently while these
measurements ran and **failed the benchmark on time**, although each half had passed on its own. Its
inside-gate stage adds up to four rounds of four candidates on mid-size rows, which is the class the
two-second cap actually bites, and one row here (crudeoil_lee3_08, inside the gate) showed +0.058 s
from it on a single run. The out-of-sample gain does not justify that risk.

Two other extensions were measured and are recorded as closed:

- **Seed diversity above the gate.** The same four-seed extraction applied to the large class moved
  **zero rows** on either corpus. The chain there has already converged; different roots find nothing.
- **Fill-edge removal above the gate.** Running the bounded watcher on the large class before the
  chain gained nothing with its own allowance (held-out 0.852272, marginally worse) and lost ground
  when it shared the chain's ledger (development 0.828711), because it starves the chain that
  produces the gains.

Together those say the terminal cleanup above the gate is exhausted, and that the remaining upside
there is in the budget, not in more kinds of cleanup. This experiment is that budget.
