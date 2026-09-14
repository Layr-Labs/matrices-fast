# Reset-first exact-window construction: remove an overwritten full-bitset copy

## Summary

This submission keeps the current frontier ordering pipeline and its bounded, exact-score accepted
improvements, then removes a provably dead memory operation from every exact-window descent call.
The mutable fill graph used by the exact-window kernel was initialized from the immutable pristine
graph immediately before the first sweep called `Game::reset()`. That reset copies the same complete
bitset again before any consumer can read the mutable adjacency. The new reset-first constructor
leaves the recycled mutable buffer uninitialized at construction and retains the mandatory reset.

This is intentionally a cap-margin change rather than an added search dose. It does not enlarge any
structural gate, ledger, width, sweep count, component set, or candidate pool. It does not change the
strict exact-flop acceptance rule. On the full 300-row development corpus, old and new produced the
same returned flop record on every row. In a production-worker timing frame, the six selected large
exact-window rows returned byte-identical permutations while median worker wall fell by 4.2% to
12.9%.

The behavior stack also contains two earlier bounded value changes already present in the source:

* exact-window component admission skips an individually unfundable component and continues to later
  components, while retaining the same precharged ledger;
* the dense/hub twin uses shape `10/4/4` only for `1,000 <= n <= 5,205`, and the prior `8/4/3`
  shape elsewhere.

The shared-basin fork is off. The displaced-ordering upper bound remains 50,000. The exact-window
ledger remains 2 GiB. No identity gate, corpus lookup, clock, environment dependency, or nondeterministic
iteration source is introduced.

## Mechanism and safety argument

For a graph of dimension `n`, the exact-window `Game` stores its mutable adjacency as a dense bitset
with `w = ceil(n/64)` machine words per vertex. The immutable `Pristine` object already owns the
canonical `adj0` bitset and its corresponding degree vector. Before this change, obtaining a game did
the following:

1. take a recycled `Vec<u64>` of capacity at least `n*w` from the thread-local pool;
2. resize it to `n*w` words;
3. copy all `n*w` words from `adj0` into it;
4. enter `subset_window_descent_config`;
5. charge the first sweep's reset work;
6. call `Game::reset`, which copies all `n*w` words from `adj0` into it again.

The first full copy has no reader. The reset-first constructor now performs steps 1 and 2, skips
step 3, and leaves steps 4 through 6 unchanged. Its scope is deliberately narrow: general `Game`
constructors still eagerly initialize their adjacency. Only the window driver that promises reset as
its first adjacency operation uses `Pristine::game_reset_first`.

All early exits are safe:

* invalid dimensions, widths, steps, sweeps, budgets, seeds, and CSR input are rejected before a
  game is constructed;
* a zero-sweep request is rejected before construction;
* pristine construction validates and builds the canonical immutable adjacency;
* after construction, if the first reset charge cannot be paid, the function returns without reading
  the mutable adjacency;
* if the charge succeeds, `Game::reset` overwrites every word before prefix elimination, exact-window
  refinement, degree inspection, or any other adjacency read.

The logical charge remains unchanged. In particular, the submission does not turn the saved physical
copy into more search work. Every sweep still charges `2*n*w + 8*n`, retains its mandatory reset,
uses the same offset, visits the same windows, and observes the same component and DP ledgers. This
separates implementation efficiency from search-dose changes and preserves the candidate sequence.

## Full-development output receipt

The comparison used the graded-closest in-process frame with the extra scoring phase marks disabled.
Both arms were compiled from the same source and selected through a test-only eager/lazy seam; the
production build always selects reset-first construction. The corpus had 300 rows.

| arm | score | lt_1k | 1k_10k | gt_10k | identical flop records |
|---|---:|---:|---:|---:|---:|
| eager construction | 0.790230 | 0.8873 | 0.8372 | 0.6822 | reference |
| reset-first construction | 0.790230 | 0.8873 | 0.8372 | 0.6822 | 300 / 300 |

The `COUNTS` records include matrix name, dimension, nonzero count, AMD flops, and returned-ordering
flops. Their old/new diff is empty. Therefore every row has the same flop ratio and all three bucket
geometric means are identical, not merely equal after rounding.

## Production-worker wall and permutation receipt

Wall was measured with separately built production-worker binaries, one worker process per row.
The run order was reversed relative to the initial screen and used three processes for each arm and
row. The table reports medians. Raw output permutation files were compared byte-for-byte for every
pair and had matching SHA-256 digests.

| row | skipped copy per call | eager median | reset-first median | delta |
|---|---:|---:|---:|---:|
| `arki0013` | 240.5 MiB | 4.5853 s | 4.3941 s | -4.2% |
| `crudeoil_lee4_09` | 30.2 MiB | 4.7092 s | 4.2643 s | -9.4% |
| `crudeoil_pooling_dt3` | 112.3 MiB | 3.6635 s | 3.4930 s | -4.7% |
| `methanol400` | 68.7 MiB | 2.8530 s | 2.7008 s | -5.3% |
| `nd_netgen-3000-1-1-b-b-ns_7` | 131.3 MiB | 3.3001 s | 2.8760 s | -12.9% |
| `ringpack_30_2` | 38.7 MiB | 4.5018 s | 3.9397 s | -12.5% |

These worker values are comparative measurements of complete worker executions in the same frame;
they are not claimed as predictions of the grader's absolute two-second clock. The relevant result is
that all six outputs are identical and the candidate removes work on every selected large class row.
At `n = 44,909`, a single avoided initialization copy is 240.5 MiB. A row can invoke the exact-window
machinery repeatedly, making this a material reduction in memory traffic without a new algorithmic
branch.

## Existing bounded value changes in this tree

### Component admission within a fixed ledger

The exact window is decomposed into live connected components of size 2 through the maximum width.
Before solving a component, the implementation charges `2^k * (16k + 6w + 24)` against the call's
deterministic ledger. The older policy abandoned the remainder of the window at the first component
whose precharge could not be funded. The current policy skips that component and continues walking
later components in deterministic position order.

The budget itself does not change, and an unfunded component is never allocated or solved. Every
returned candidate is still rescored exactly by the caller and adopted only on a strict reduction.
On 300 development rows this changed one row, improved that row, left 299 rows identical, and produced
no regression: score `0.790236 -> 0.790230`; `crudeoil_lee1_07` moved from `0.745866999` to
`0.744049962`. A smallest-first variant was separately tested and was identical to the shipped policy
on all 300 rows, so it is not included.

### Census-bounded dense/hub twin

The wider `10/4/4` twin shape is confined to dimensions 1,000 through 5,205. Outside this interval the
frontier's `8/4/3` shape remains in force. The bound removes a measured small-graph regression from the
unbounded variant while preserving the two independently observed mid-band movers. It does not admit
larger hidden hub rows to the wider schedule.

## Cap and generalization controls

The submission retains the following conservative production limits:

* the exchange ledger is exactly 2,147,483,648 logical work units;
* the exact class dimension ceiling is 45,000;
* class entry remains density- and maximum-degree-gated;
* the alternate PEO window remains at the frontier's 50,000 ceiling;
* the shared-basin fork remains disabled;
* no extra exact pass, sparse-span pass, PEO round, twin, restart, or candidate is added;
* every adoption remains a strict exact-flop decrease;
* all gates are functions of structural quantities such as `n`, `nnz`, and degree.

Memory for the exact mutable graph scales as `n*ceil(n/64)*8` bytes, and the immutable pristine graph
has the same leading term. Reset-first construction reduces transient bandwidth but does not change
the live-memory bound. The 45,000 ceiling and 2 GiB logical ledger remain unchanged, avoiding a new
large-matrix class.

## Verification

The trusted parent and candidate worker were rebuilt from the locked offline dependency set after the
change. The full release worker suite passed: 126 active tests, 0 failed, with 56 benchmark/probe tests
ignored by default. The suite covers bijection validity, determinism, exact-window oracle agreement,
budget exhaustion, reset semantics, component handling, signature-engine agreement, scoring-workspace
agreement, and synthetic exhaustive cases.

The reset-first change also has a test-only eager seam so future comparisons can restore the prior
copy without maintaining divergent source branches. That seam is compiled out of production behavior;
the graded worker does not consult the environment.

## Expected trade

The score contribution comes from the already bounded strict-decrease component-admission and dense
twin changes. The new change is deliberately score-neutral and targets failure risk: it gives back
memory bandwidth in the terminal exact-kernel class without consuming any search budget. The main
risk is that a hidden corpus contains no row moved by the bounded value devices, in which case the
submission can complete yet miss the promotion threshold. The change does not create a new cap class
or worsen an accepted ordering, so that score risk is preferable to adding a speculative pass near a
tight per-matrix limit.
