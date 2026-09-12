# 0154 — Flat completion reconstruction and exact linked MCS buckets

## Model and target

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

The current promoted source is `ab30c0e`, submission `a9905f2`. Its hidden
flop score is **0.842857** and hidden fill ratio is **0.945102**. Its complete
public development score is **0.792439173**. Lower is better. No hidden win
is claimed until the grader completes and Yukon reports an improvement.

This candidate starts from experiment 0153, local source `e9da6ea`, and keeps
the sparse terminal exchange, exact incumbent-score bookkeeping, invocation-
local scalar-power reuse, and x86 minimum-fill POPCNT dispatch. The new
production change is confined to `peo_extract.rs`: store the incumbent's
chordal completion in flat arrays and run maximum cardinality search with one
live linked bucket entry per vertex. Both changes preserve the exact candidate
permutations, including adjacency visitation and LIFO tie order.

The terminal ordering policy remains width-10/24M, width-12/stride-5/64M,
then width-8/stride-3/32M on the same sparse envelope. Its total allowance is
120M, compared with the promoted parent's 136M. The envelope is
`1,000 <= n <= 12,000`, `nnz <= 200,000`, `nnz <= 16*n`, and maximum input
degree at most `n/2`. The alternate-seed PEO phase remains retired on that
envelope. All other terminal gates and schedules remain as in 0153.

## Why this is a runtime experiment

The 0153 submission `ed253b8a-b262-493b-b7a8-93915d391a49` passed dependency
checks, sandbox verification, and compilation on the x86 grader, then failed
the hidden two-second per-matrix cap in workflow
[34684270583](https://github.com/Layr-Labs/matrices-fast/actions/runs/34684270583).
Benchmark began at 2026-09-12 08:53:21.351 UTC and reported the cap kill at
08:54:55.661 UTC, approximately 94.31 seconds later. It exposed neither a
hidden score nor a matrix identity.

Several preceding terminal-exchange submissions also failed that cap. An
unchanged-parent control matters: source `6afc1d2`, submission `f7525b4b`,
changed only two Markdown records relative to `ab30c0e`, yet workflow
[34665838580](https://github.com/Layr-Labs/matrices-fast/actions/runs/34665838580)
also failed the cap. This prevents assigning every opaque timeout exclusively
to a new terminal pass. It does not establish that another identical retry
would succeed. One unchanged-source retry was already used and failed.

The next useful experiment must reduce actual work. Reconstruction and MCS
have clear allocation costs independent of matrix identity. The prior
representation allocates separate child, factor-column, adjacency, and MCS
bucket vectors. Repeated PEO rounds rebuild them. Eliminating allocations and
stale MCS entries makes that same general ordering procedure cheaper without
adding a new family or buying extra search with a larger allowance.

## Flat reconstruction, with append order preserved

The reconstruction still validates dimension, input column pointers, row
indices, incumbent bijection, parent ordering, factor-column counts, and the
factor-nonzero limit before extracting candidates. Invalid or over-limit
inputs return `None` under the same structural contract.

First build elimination-tree children as a counting-sort CSR. Parent counts
determine the offsets. Inserting children in ascending original column order
reproduces the old per-parent child-list order exactly.

Factor-column lengths are already known from `counts[j] - 1`. Their prefix
sums provide flat offsets. One `Vec<u32>` stores the reconstructed factor
neighbors in the original reach order: input neighbors first, then the
previously reconstructed child columns, with the same per-column marker
deduplication. Each completed column must have its expected length before
construction proceeds. Child columns precede their parent, so their flat
ranges are complete and remain valid while later values are appended.

Count undirected completion degrees in original vertex coordinates. Prefix
sums allocate the exact adjacency capacity. Replay every old append event in
the same column and neighbor order, writing `v -> w` followed by `w -> v`
through per-row cursors. Consequently each vertex's adjacency slice contains
the same sequence as the former `Vec<Vec<u32>>`, rather than merely the same
edge set. This is necessary because reverse and forward MCS use that sequence
to resolve ties.

The factor columns are retained until replay instead of freed individually
at their parent. This trades a bounded flat factor buffer for fewer heap
objects and reallocations. Space remains O(n + nnz(L)). The existing largest
PEO factor limit is 20M entries; an 80MB factor-column buffer plus about 160MB
of symmetric completion neighbors is well below the 4GiB worker limit.
No new limit, matrix identity test, dependency, or external input is introduced.

## One current MCS entry per vertex

The former MCS puts a new copy of a vertex into a LIFO bucket every time its
weight increases. Old copies stay in lower buckets until popped and rejected
as stale. This is correct and linear in the completion edge count, but its
bucket vectors retain O(n + |E|) entries and allocate as they grow.

The replacement has arrays for bucket heads, each vertex's next and previous
entry, weight, and visited flag. Initially bucket zero follows the incumbent
order, matching the former reverse initialization followed by stack pops.
When an unvisited neighbor gains weight, unlink it from its former bucket and
insert it at the head of its new bucket. A selected vertex leaves its bucket
and is marked visited. Empty highest buckets are skipped as before.

This is the same LIFO ordering over valid entries. Removing a stale entry
early cannot change the relative order of entries that remain valid. Every
promotion places the vertex ahead of the current entries in its new bucket,
exactly as the former push did. The forward and reverse adjacency scans retain
their existing sequences. Reverse the resulting visit sequence to produce
the PEO, as before.

Bucket storage is now O(n), with no stale copies or growable per-weight
vectors. The graph and candidate semantics are unchanged. Extra linked-list
writes could have outweighed the allocation savings, so this was measured
before being selected for production.

## Differential and CPU evidence

The active public reconstruction test checks 535 accepted cases, using AMD
and a fixed relabel ticket across the development corpus. It compares every
ordered neighbor slice with a frozen former reconstruction. For both MCS
directions it also compares the entire returned permutation with the frozen
stale-bucket implementation. The existing exhaustive test covers every graph
and elimination ordering through five vertices, checks the independently
filled graph and exact symbolic flop scorer, and now asserts old/new MCS
permutation equality on each case. Bad-input tests remain active.

A paired reconstruction diagnostic uses the complete shipped incumbent on
146 medium and large public rows, under `n <= 50,000`, `nnz <= 1.2M`, and
factor nonzeros at most 4M. Three pairs per row alternate old/new execution
order and retain each variant's minimum. Aggregate old reconstruction time
was **0.128621 s**, versus **0.066160 s** for flat reconstruction, a reduction
of about **49%** in this measured kernel. The frozen old helper omits some
validation performed by the new path, so that detail favors the old helper.

A separate paired MCS diagnostic runs both forward and reverse extractions
inside each timed call on 144 accepted medium/large AMD completions. Three
pairs per row alternate old/new order. Both complete permutations agree
before timing. Aggregate minimum time was **0.190000 s** for stale buckets
and **0.120430 s** for linked buckets, about **37%** less measured MCS time.
These are kernel measurements on the ARM development host, not claims of a
37–49% reduction in whole-pipeline time or hidden x86 runtime.

The full direct public probe completed all 300 patterns at **0.792346128627**
and preserved every final flop count versus 0153. Its maximum call was
**0.936 s** and the probe took **91.81 s**. A terminal-screen process ran
concurrently for part of this probe; whole-run times must not be compared
causally with earlier differently loaded runs. The paired diagnostics provide
the evidence for selecting these kernels. The hidden cap remains an external
verification requirement.

## Terminal variants screened and rejected

The test-only schedule screen captures the exact pre-terminal incumbent,
replays width-10/24M and width-12/stride-5/64M once, then compares six final
passes with the same 32M allowance and four sweeps. The control reproduces
the entire shipped permutation on every in-gate row; aggregation includes
all 300 public patterns with unchanged out-of-gate results.

| final width / stride | full public score | wins / losses versus control |
|---|---:|---:|
| 8 / 3, retained control | 0.792346128627 | 0 / 0 |
| 8 / 2 | 0.792361894455 | 4 / 9 |
| 9 / 4 | 0.792383532852 | 11 / 18 |
| 10 / 3 | 0.792408511164 | 8 / 17 |
| 13 / 5 | 0.792461241968 | 4 / 26 |
| 14 / 5 | 0.792463528797 | 1 / 27 |

No screened replacement improves the control. None is wired into production.
Keeping the control avoids increasing either the nominal allowance or the
measured final-pass work to chase a few isolated wins.

## Submission checks and remaining uncertainty

Only `src/ordering/` is changed. Production reads only its pattern input and
uses Rust standard-library arrays and vectors for the new kernels. It has no
clock, environment, filesystem, network, global cross-matrix cache, corpus
fingerprinting, or output dependency on unordered iteration. Test diagnostics
use clocks and public corpus names only under `cfg(test)`.

The complete release test suite passed **120 active tests**, with 42 diagnostic
tests ignored. Official sandboxed `yukon run` passed all **300 matrices** at
**0.792346**, fill ratio **0.924436**, including purity/dependency gates and
repeated deterministic executions. The local score and fill match 0153.
`git diff --check` passed. The remote outcome will be appended below.
The intended hidden comparison is against **0.842857**, rather than the public
0.792439 baseline, and a passed build alone is not an improvement.
