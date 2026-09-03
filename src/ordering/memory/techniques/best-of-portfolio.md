# Best-of portfolio (the current architecture)

## What it is
`order()` does not commit to one algorithm. It runs a *portfolio* of candidate
orderings on the matrix, scores each one with feral's own symbolic analysis
(`Σ cⱼ²`, the exact quantity the grader ranks), and returns the cheapest. The
portfolio is **anchored on `feral_amd::amd_order` with library-default options**,
which is bit-for-bit the grader's baseline ordering.

## Why the anchor is the whole trick
Because the baseline itself is in the candidate set, the returned permutation can
never be worse than AMD on any matrix: `ratio ≤ 1.0` always. Every additional
candidate is therefore **free upside** — it either wins and lowers the ratio, or
loses and is discarded. There is no risk/reward tradeoff to balance, no tuning
that can backfire on an unseen matrix.

This inverts the usual research question. It is not "which ordering is best on
this family?" but "**which candidates can I afford to run?**" The only cost of a
bad candidate is time.

## Consequence: the binding constraint is the 2 s cap, not ordering quality
Every design decision reduces to a time budget. Candidates are gated by
`(n, nnz)` — never by wall-clock, because the harness runs `order()` twice and
requires byte-identical output, so the candidate set must be a pure function of
the pattern's shape.

**Measured 2026-07-26 (see [experiments/0002](../experiments/0002-measured-gates-metis-kahip.md)):**

| | |
|---|---|
| worst local `order()` | **1.019 s** on `crudeoil_lee4_10` (n=17809, nnz=120632) |
| hard cap | 2.0 s, SIGKILL, one breach FAILs the entire run |
| second/third worst | 1.011 s `ringpack_30_2`, 0.869 s `nuclear104` |

That is only a ~2× margin. Anything added to the slow tier risks the run. The
[`probe`](../../probe.rs) module exists because the harness prints `(capped)`
instead of a time, so this number is otherwise invisible.

The cost driver is **nnz, not n** — `qapw` (n=705, nnz=87496) takes 0.539 s while
much larger sparse matrices take less. Gate by nnz first, n as a backstop.

## Where the remaining headroom is
122 of 300 matrices are tied at *exactly* 1.000 — AMD beats every separator-,
profile- and bandwidth-based candidate on them (60 in lt_1k, 40 in 1k_10k, 22 in
gt_10k). Pushing every tie to 0.95 would take the score from 0.888 to ≈0.869, so
that set bounds what this architecture can still deliver by adding more of the
same kind of candidate.

Per-matrix leverage is very uneven, and this decides where to spend effort:

| bucket | count | weight | leverage per matrix |
|---|---|---|---|
| lt_1k | 147 | 0.30 | 0.0020 |
| 1k_10k | 108 | 0.30 | 0.0028 |
| gt_10k | 45 | 0.40 | **0.0089** |

A single gt_10k matrix is worth ~4.4 small ones. Unfortunately gt_10k is also
where the time cap bites hardest — the largest tied matrices (`faclay75`
n=272878, `acopf_case9241pegase_qcqp` n=313068, `gabriel10` n=244056) are gated
out of every candidate except AMD and one AMF pass.

## Diminishing returns — measured
A blanket 12-variant partitioner sweep (METIS seeds/imbalance/switch/dense-
quotient, Scotch seeds, KaHIP seeds/modes) improved only **7 of 260** matrices in
the cheap region, worth −0.0042 on the score. The portfolio is close to the
ceiling of *this family of methods*; the next real gain probably needs a
qualitatively different candidate, not another partitioner setting. See
[open-questions.md](../open-questions.md).

## The prediction above was right, and here is what "qualitatively different" meant
The 122 ties are not evidence that the matrices are hard — they are evidence that
**AMD is the winner there**, and that every candidate added so far was drawn from
families that lose to AMD on that set. The fix was not a better competitor to AMD
but *another AMD*: because AMD's tie-breaking reads the vertex numbering,
`AMD(Q A Qᵀ)` composed back through `Q` is a different minimum-degree ordering
for the cost of one AMD pass. That single change improved **41 of 300** matrices,
worth −0.0070 — more than the entire partitioner sweep — and most of the wins are
former exact ties. See
[experiments/0003](../experiments/0003-relabelled-amd-multistart.md).

The transferable lesson: when a large set is tied at the anchor, look for
variation *within* the anchor's own family before adding another family.

It also changed the shape of a gate. Every other candidate here uses a hard
`(n, nnz)` envelope; the multi-start instead takes a per-matrix **time budget**
(`restarts = budget / nnz`), which bounds its cost everywhere at once and makes
the budget its own gate. That pattern is reusable for any candidate whose cost
scales with nnz and whose quality improves with repetition.

## The multi-start is a LOTTERY, so buy tickets rather than aim them

A follow-up sweep tested the obvious next step — spend part of the same restart
budget hill-climbing on the relabeling instead of resampling it uniformly — across
17 explore/exploit policies at identical cost. **Nothing beat i.i.d. uniform
robustly.** Bigger perturbations beat smaller ones monotonically, and removing the
chaining (so it is no longer a hill climb at all) changed nothing: AMD's
tie-breaking is a global cascade, so two relabelings a few transpositions apart do
not produce two similar orderings. There is no gradient to climb. See
[experiments/0004](../experiments/0004-structured-relabelings.md).

The transferable lesson, and it generalizes past this family: when a candidate's
quality-per-try is essentially i.i.d., the only lever is **the number of tries**, so
its design question is a *timing* question, not a search question. Which puts it
back where everything in this portfolio ends up — against the 2 s cap.

## …but "more tickets" has two readings, and the second one is much cheaper

Buying more tickets in the SAME lottery saturates: the budget table on
`RELABEL_BUDGET` shows that past 300000, each further 0.05 s of worst case buys under
0.0002 of score. Buying tickets in a **different** lottery does not, because the
prizes are somewhere else. `AMF(Q A Qᵀ)` composed back through `Q` costs one AMF pass
— exactly what one more AMD restart costs — but min-fill and min-degree *disagree
about which vertex to eliminate*, so its draws are not redundant. That single
observation was worth −0.0051, more than any tuning of the AMD multi-start could
reach: [experiments/0005](../experiments/0005-relabelled-amf-multistart.md).

The general form is the reusable part:

> **Any ordering routine whose output depends on the input vertex numbering becomes a
> randomized-restart algorithm under `relabel`, for free.** Run it on `Q A Qᵀ`,
> compose the result back through `Q`, and offer it to `consider`.

Cost is one pass per seed and score risk is *structurally* zero under the best-of
floor, so the only question per family is whether its pass fits the budget. Several
routines in this file have never been relabelled — RCM, Sloan, the ND separator
choices, MinFill — and each is a candidate second lottery. Prefer the ones whose
objective differs most from min-degree, since that difference is the whole mechanism.

That experiment also added a discipline this corpus needs. The score is a
size-bucketed geomean over 300 matrices with a **heavy tail** — `gt_10k` is 45
matrices at weight 0.40, so one 20% win there is worth ≈0.002 of score on its own,
which is larger than most changes ever measure. A 0.001 "improvement" from a single
instance is therefore the default outcome of any sweep, not a signal.
`probe_relabel_search` reports, alongside every score: the advantage measured on two
**disjoint halves** of the corpus, and the advantage with the **single biggest
contributing matrix dropped**. Anything that flips sign between halves or dies under
drop-1 is one lucky matrix. Run those columns before believing a delta — and before
shipping something that would be corpus-overfitting by accident.

## Exact search above the small tier

The exact elimination-game search is also useful above its original `n <= 1,000`
gate when work is limited by an operation count rather than elapsed time.
[Experiment 0020](../experiments/0020-medium-exact-search.md) uses two serial
stages with nominal budgets of 100M then 50M word operations on
`1,000 < n <= 6,000 && nnz <= 30,000`. It lowered the synced 300-matrix score
from 0.860780 to 0.859116, entirely in `1k_10k`. A 10,000-vertex extension
added cost but no wins. The inherited guard permits 25% extra work and checks
only at pivot boundaries. The useful rule is therefore to gate where a complete
exact trajectory fits this deterministic bounded budget; a larger dimension
limit alone does not add search depth.

## Allocate late subtree work to depth

After several subtree-refinement rounds, equal-work sweeps show that trajectory
depth matters more than broad block coverage. [Experiment 0025](../experiments/0025-adaptive-terminal-deep-subtree-search.md)
first reduced the frontier-source score from 0.851168 to 0.849622 with a 32M
deep phase, but its broad gate failed the hidden 2 s limit. The corrected design
uses four 4M searches for medium matrices and eight 2M searches for large
matrices, all inside `n<=80k/nnz<=250k`. It scores 0.850518 from the newer
0.851055 frontier source. Keep ranked selection and run the pass after the
complete chain, but treat the matrix-wide work limit and structural gate as part
of the algorithm. A fast local corpus result does not prove hidden safety.

## Links
- [amd.md](amd.md) — the anchor, and why it is hard to beat here.
- [nested-dissection.md](nested-dissection.md) — the separator family in the portfolio.
- [experiments/0002](../experiments/0002-measured-gates-metis-kahip.md) — the measurements above.
- [experiments/0003](../experiments/0003-relabelled-amd-multistart.md) — the relabelled-AMD multi-start and the budget-as-gate pattern.
- [experiments/0004](../experiments/0004-structured-relabelings.md) — why its search policy is settled, and the two-half / drop-1 robustness columns.
- [experiments/0005](../experiments/0005-relabelled-amf-multistart.md) — the second lottery (relabelled AMF), and the general "relabel anything numbering-sensitive" recipe.
- [experiments/0020](../experiments/0020-medium-exact-search.md) — bounded exact search in the medium sparse tier.
- [experiments/0025](../experiments/0025-adaptive-terminal-deep-subtree-search.md) — adaptive deep allocation for a terminal subtree pass.
