# Cross-candidate subtree transplant: donate the portfolio's discarded orderings into the elimination tree's blocks

## Baseline

Promoted tip `996e8d6` (submission `68d650ef`), which superseded `4d86414`
(`e5310ad6`, hidden 0.859573) while this experiment was being measured. Local
dev on this host, rebuilt from that tree and measured in the same pinned
interleaved run as the candidate: **0.826558**, 68 tests. All three of that
tip's changes are kept byte-for-byte, including `PEO_ALT_SEEDS = 8` and the
`ROBUST_MAX_N`/`ROBUST_MAX_NNZ` widening. The 32-set stage-6 change stays
withdrawn.

## Result

**dev 0.826558 -> 0.825648 (9.10 bips)**, fill 0.937028, 300 rows OK, 68 tests.
Per bucket lt_1k 0.889660 / 1k_10k 0.862148 / **gt_10k 0.750263**. On the
previous tip `4d86414` the identical change measured 0.826784 -> **0.825411**
(13.73 bips); the difference is this tip's own `PEO_ALT_SEEDS = 8`, see
Finding 4.

**And the candidate's corpus maximum is 59.2 ms BELOW this tip's** (1.4568 ->
1.3976 s, pooled single-trial sd 1.5 ms, 34.3 sigma). This tip's robust-envelope
widening admits `faclay75` (n = 272 878, nnz = 1 379 706) to the robust AMD pass
and takes it to 1.4572 s = **1.986 s on the slowest host on record, 99.3 % of
the 2 s cap**. The inherited pre-symbolic entry refusal returns 70-78 ms on
exactly `faclay75`, `gabriel10` and `acopf_case9241pegase_qcqp`, handing back
83 ms of that slack.

Files changed: `mod.rs` (the stage plus three constants), `scoring_ws.rs` (three
accessors that return arrays the scorer had already computed), `peo_extract.rs`
and the stage-6 call site (score-neutral hygiene inherited from `0085`'s
surviving half, hidden-validated as `f8941f7f`).

## The observation this starts from

`leader_order` has **36 `consider` call sites**. Each builds an ordering, scores
it exactly, and discards it unless it beats the incumbent. On a small row about
thirty orderings are built and thrown away. `0084` recovered a little of that by
retaining the four best *displaced* orderings and running each through the
terminal PEO chain — a whole-permutation use of a discarded candidate.

But a discarded ordering is not uniformly worse than the incumbent. It is worse
*on aggregate*, and on some part of the graph it is better. The question is how
to spend a donor's local opinion without paying for its global one.

## Why the elimination tree makes that exactly answerable

`rgreedy::subtree_refine` already rests on two standard facts:

1. For `v` in a subtree `S` of the elimination tree of `perm`, `c_v` depends
   only on `v`'s DESCENDANTS (Liu's reachable-set characterisation), and `S` is
   closed under descendants.
2. The fill graph after eliminating a SET does not depend on the order within
   the set, so every `c_w` for `w` outside `S` is unchanged by any internal
   reordering of `S`.

So `Σ c_j²` splits as `(fixed part) + Σ_{v∈S} c_v²`, and an improvement inside
`S` is a global improvement of exactly the same size. An etree postorder — which
leaves `Σ c_j²` unchanged — makes every subtree a contiguous range of positions,
which is what lets the split be read off the permutation.

The consequence this experiment uses, and which `subtree_refine` does not need:
**disjoint subtrees contribute independently**. So transplanting one donor into
EVERY block at once and taking ONE exact score yields the exact per-block delta
for every block simultaneously, and the blocks can then be accepted
independently on a strict decrease. `k` donors cost `k` scores, not `k × blocks`.
The identity `score(assembled) == inc_f - Σ gains` is asserted on every winning
row of every probe arm and has never failed.

## The stage

Appended as the last statement of `leader_order_pool`, after the alternate-seed
chains:

```
inc_f, counts, etree parent, etree postorder  <- ONE score of the incumbent
base = incumbent permuted by the postorder      (score-identical)
for width in [4096, 512, 128, 32]:              coarsest first
    blocks = maximal disjoint postorder subtrees with size in [4, width]
    for donor in runner_up:                     the pool 0084 already retains
        trial = base with every block re-sorted by the donor's rank
        ONE score of trial; per-block exact contribution vs base's
        accept each block independently on strict <
    assemble the per-block winners, ONE score, keep on strict <
```

Widths run **coarsest first** because the largest wins are whole-subtree
donations: `procurement1large` -8.69 %, `crudeoil_lee4_10` -4.68 %,
`crudeoil_lee4_09` -4.60 %, `nuclear10a` -4.19 % all land at width 4096. A
ledger that runs out has to run out on the fine end.

The move is **non-local**: a transplanted block order is not in any perturbation
neighbourhood of the incumbent. That is why it reaches rows the swap class of
`0085`, and the 3-cycle / segment-reversal / Or-opt classes probed alongside it,
cannot. Nothing in it is a budget, a cap, a seed count or a gate width, and the
whole precondition is a function of `(n, nnz)` and of exact integer scores.

## Finding 1: the mechanism is worth ~35 dev bips unmetered

Post-hoc from `leader_order`'s own output (terminal placement, so a post-hoc
probe returns the shipped score exactly rather than a lower bound), all four
widths, eight donors, no ledger, over the 281 rows with `n <= 60_000`:

**87 rows move, 43.8 points of summed ratio spread, ~35.6 dev bips**, the
largest single rows being `procurement1large` -8.69 %, `methanol200` -4.85 %,
`crudeoil_lee4_10` -4.68 %, `crudeoil_lee4_09` -4.60 %, `nuclear10a` -4.19 %,
`methanol400` -3.75 %, `pooling_sppc1pq` -3.04 %, `crudeoil_pooling_dt3`
-2.95 %. Note that `crudeoil_lee4_10` is the corpus's *slowest* row and it is
also one of the biggest winners.

That ceiling is not shippable, which is the whole content of Finding 3.

## Finding 2: the cost is one call to the scorer the tree already runs 30 times

The added work is `k + 2` calls to the **same** `ScoreWorkspace::flops` the
36 `consider` sites already use. Three new accessors (`counts`, `etree_parent`,
`etree_postorder`) expose arrays that call had already computed and reduced —
`nnz_l` is the sum of `counts` and the returned flop count is the sum of its
squares — so the per-position information costs literally nothing extra.

That scorer is `permute_pattern` + `EliminationTree::from_pattern` +
`column_counts_gnp`: `O(n + nnz·α)`, and it **never materialises the factor**.
So `fill` and `Lnnz` do not appear, and a pattern that fills in completely costs
exactly what a sparse one costs.

This is precisely the property `SmallScore` lacks. `0085` learned the hard way
that stage 6's `n <= 300 && nnz <= 3000` gate is not a work bound because a
`SmallScore` pass costs `words × (n + fill)` and `fill` is an OUTPUT. Here the
cost driver is an INPUT, so the gate really is a bound — and the ledger below
makes the added work an absolute constant on top of that.

## Finding 3: the ledger, in time, and the shape of the trade

Every score is charged `n + nnz` units against one `TRANSPLANT_LEDGER`, and a
row is refused outright unless the cheapest sequence that can produce anything
(setup + one donor + one verification = four units) fits. Measured rate over the
262 in-gate rows: **20.6-21.1 ns per unit** in the probe's fresh-allocation
form - but that measurement is **unpinned and optimistic by ~5x**. Pinned to two
cores under bubblewrap the implied rate on the A/B's flagged rows is
**40-106 ns/unit**, so the honest bound is 600k x 106 ns = **63.6 ms** local and
**86.7 ms** slow-host. Roughly 30 ms of every flagged row's delta is not even
attributable to the change: three rows the gate REFUSES outright
(`arki0013`, `transswitch2383wpr`, `crudeoil_pooling_dt3`) read +27 to +38 ms in
the same run, and a five-trial escalation on the previous tip put the largest
replicating increase at **+28.3 ms**. Fit any future ledger to pinned deltas.

| ledger | winners | dev bips | local worst added row | rows > +25 ms | rows > +50 ms |
|---:|---:|---:|---:|---:|---:|
| 600_000 | 55 | **11.29** | 20.8 ms | **0** | **0** |
| 800_000 | 63 | 14.87 | 23.6 ms | 0 | 0 |
| 1_000_000 | 70 | 22.91 | 31.8 ms | 11 | 0 |
| 1_200_000 | 70 | 23.94 | 41.6 ms | 16 | 0 |

**600_000 is shipped.** At the pinned worst-case rate that is 63.6 ms local and
86.7 ms slow-host, and it fits because the candidate's own corpus maximum is
1.3962 s -> **1.903 s** slow-host, leaving **97 ms** of slack where the tip it
builds on leaves **14**. At `PEO_ALT_SEEDS = 8` the interesting ledger is
1M-1.2M rather than 600k (22.9-23.9 dev bips); that waits on this draw. `4(n + nnz) <= 600_000` means `n + nnz <= 150_000`, so every large row —
`acopf_case9241pegase_qcqp` at n = 313_068, `faclay75` at n = 272_878, every row
above nnz = 150_000 — receives **zero** added work. The rows the cap actually
threatens are refused by arithmetic rather than by a dev-corpus statistic.

The stage is the **last statement** of `leader_order_pool` and `best_perm` is
returned on the next line, so no downstream conditional reads what it changes.
The realized-chain question that `0082`'s post-mortem raised is discharged by
position, not by a subset argument.

## Finding 4: donor count and ledger trade against each other

At `PEO_ALT_SEEDS = 4` the 600k ledger yields **11.29** dev bips; at **8** it
yields **3.03**, because eight donors at the coarsest block width exhaust the
ledger before the finer widths are reached. That interaction is the whole of the
13.73 -> 9.10 difference between this change on `4d86414` and on `996e8d6`.

`PEO_ALT_SEEDS = 8` is kept anyway - it is promoted, and no submission should
revert a promoted constant to buy score for its own mechanism. But the two knobs
have to be tuned together from now on: the widths are a better spend of a fixed
ledger than the donors are.

## What this does not settle

- **The ledger.** 22.9 dev bips sit at 1M units behind 11 rows crossing +25 ms.
  That distribution is well inside the only one ever measured cap-safe on this
  benchmark (61 rows > 25 ms / 26 > 50 ms / 138.8 ms worst, from `f8941f7f`,
  which was `rejected` and therefore ran to completion), but it is outside a
  cautious default, so it waits on this draw's verdict instead of being argued.
- **Placement.** Terminal placement prices exactly. Wired before the terminal
  PEO chain the same change would re-enter it, where `0085` measured a factor of
  3 between the post-hoc and shipped value of a change — but it would forfeit
  the by-position discharge above. That is a second experiment.
- **Donor source.** The pool retains the best *displaced* orderings. The stages
  that assign `best_perm` directly are still unrepresented in it, which is
  `0084`'s own declared next step.
- **Iteration.** One pass only. Re-postordering the accepted result and
  transplanting again is untested and would double the ledger.
