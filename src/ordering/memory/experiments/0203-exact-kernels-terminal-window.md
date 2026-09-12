# 0203 — Exact runtime kernels and post-search window descent

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

## Target and baseline

The target is the promoted solver `7fd61df`, submission
`4e45ee63-2518-4f10-a949-3c7a28d8ca69`, at hidden flop score **0.842716**
and hidden fill ratio **0.945064**. This replaces the earlier `ab30c0e`
target at 0.842857. Recent submissions and both public notes were inspected
before choosing this implementation. The subsequent stratified-batch candidate
`7bba860` failed the hidden cap; its batch-selection change is not included.

The newly promoted source was checked out and run through sandboxed
`yukon run` before editing production behavior. All **300** public matrices
passed at **0.792226**, fill **0.924450**. Its exact direct score is
0.792225529081. Bucket counts are 147 / 108 / 45, with flop geomeans
0.887426 / 0.839125 / 0.685651. The scoring function is the weighted mean
of within-bucket geomeans, using weights 0.30 / 0.30 / 0.40; lower is better.
The grader recomputes the column counts and sum of squared column counts.

The local environment is an ARM macOS development host with the existing
gcc, cargo, cargo-deny, git-lfs and Yukon installation. The matrix corpus was
already cloned through Git LFS and setup was complete. All work continued in
the benchmark repository. No manifests, dependency selections, trusted harness
sources, or corpus files are changed by this candidate.

## What the promoted source taught us

Repeated earlier terminal candidates failed the hidden two-second cap.
Even a control with byte-identical promoted Rust source failed once, so the
opaque failure alone did not identify a particular new loop as its cause.
Our previous submission `f590dd38` also failed the cap and obtained no score.
There is no hidden improvement to claim from any of those failed runs.

The new promoted solver supplies a working structural runtime policy: when
its incumbent predicted flop count exceeds **20 billion**, a producer batch
is truncated to its first **64** entries. This limits expensive portfolios on
graphs whose factorization work is much larger than their input nnz suggests.
Its terminal exact greedy search uses a 50M allowance at `nnz < 3*n`, a 200M
allowance otherwise, and stops at `n <= 10,000`, `nnz <= 200,000`.

This candidate preserves that producer limit, the batch head and replay order,
the complete promoted portfolio, the terminal greedy stream, all original
window passes, donor transplantation, and the full **50k** alternate-PEO
envelope with its **4M** work allowance. The prior experiment that retired the
alternate phase above 1k is not carried forward. That retirement saved local
time but subsequent public submission notes showed a hidden score cost.
The new implementation obtains headroom from exact runtime kernels instead.

## Hypothesis and terminal placement

A terminal greedy search can move an ordering into a different elimination
basin after all earlier exact window passes have finished. The earlier windows
therefore do not prove that the final returned permutation is locally optimal.
Run another bounded exact window descent after the promoted terminal search,
starting from its final incumbent and accepting only a strict reduction in the
independently recomputed objective.

An eliminated set determines the remaining fill graph independently of the
order inside that set. Thus replacing a contiguous window can reduce its
column-count contribution without changing the suffix graph. The existing
component-factored subset DP solves each connected part of a window exactly,
retaining completed gains if its shared preparation/DP/replay ledger expires.
No new DP formula, floating approximation, or reported candidate score is used.

Production selects width **8**, **4** sweeps, offset step **3**, with one
**64M** allowance. Offsets are 0, 3, 6 and 1 modulo the window width.
Admission uses only the supplied pattern: `6 <= n <= 12,000`,
`nnz <= 200,000`, `nnz <= 16*n`, and maximum input column degree at most
`n/2`. This bounds bitset storage and replay work and excludes dense/hub
patterns. The output is checked as a bijection, then compared with the exact
score of the current returned permutation. This final comparison deliberately
does not depend on a legacy scalar incumbent ledger that earlier phases may
leave stale. Existing earlier adoption trajectories remain unchanged.

## Complete public terminal screen

The diagnostic screen evaluates four independent refinements from the same
promoted final ordering on every public matrix. Unadmitted matrices contribute
their unchanged incumbent score. Each emitted ordering is checked as a bijection
and independently scored; every admitted arm is asserted non-worsening.

| Width / offset step / allowance | Weighted score | Wins by bucket | Extra screen CPU |
|---|---:|---:|---:|
| Promoted final output | 0.792225529081 | — | — |
| 8 / 3 / 32M | 0.792125197228 | 0 / 22 / 5 | 0.321235 s |
| **8 / 3 / 64M** | **0.792029985254** | **0 / 28 / 7** | **0.412416 s** |
| 7 / 2 / 32M | 0.792140037390 | 0 / 23 / 5 | 0.257315 s |
| 10 / 3 / 32M | 0.792172971552 | 0 / 24 / 5 | 0.522410 s |

The selected arm improves **35** matrices and worsens **zero**, for a public
absolute decrease of approximately 0.000196 (2.47 relative basis points).
The seven-vertex arm is cheaper but leaves value, and the ten-vertex arm is
both slower and worse here. The 64M eight-vertex arm is selected because its
additional 32M buys substantial extra reduction for only 0.091181 seconds
across the entire diagnostic corpus. Those are measured ARM screen times,
including scoring completed candidates, not a guaranteed hidden CPU price.
Final integrated scoring is recorded below after the trusted run completes.

## Exact sorted-permutation workspace

The immutable full scoring pattern is transposed once into flat row pointers
and original-column indices, owned by this `order()` invocation. For each
requested permutation, compute its inverse and new column lengths. Visit new
rows in ascending order and append them to the appropriate permuted columns
using the transpose. This constructs the same sorted CSC representation as
feral in **O(n + nnz)** time without comparison sorting each column.

Exact sorted rows matter: the completion builder and MCS preserve neighbor
visitation order, so preserving symbolic counts alone would not pin ties.
All **1,200** comparisons across 300 patterns and identity/reversal/two fixed
relabels preserve both column pointers and every row-index entry. The active
synthetic test adds nonsymmetric, unsorted and duplicate-entry input through
eight vertices, including dimension zero and repeated workspace reuse.

The current paired kernel run measures feral at **0.206036 s**, the new builder
at **0.111766 s**, and transpose setup at **0.017040 s** across the corpus.
The roughly 46% kernel reduction is not a whole-ordering speedup claim.
All 34 full-pattern permutation sites in the leader use this workspace;
residual-core helper permutations retain their existing paths.

## Exact completion arrays and MCS buckets

The promoted completion kernel pools nested vectors. This candidate instead
uses exact-size flat child, factor-column and filled-adjacency arrays. Original
input neighbors and child reach sets are visited in their former order. Each
symbolic count is checked, and filled-graph edges are replayed in the original
append order, including reciprocal neighbor appends. Every ordered adjacency
row therefore matches the original builder.

MCS keeps one linked bucket entry per vertex using head/next/previous arrays.
Updating a label unlinks its former entry and pushes the new entry at the
bucket head. This preserves the order of valid LIFO entries from the old
stale-entry implementation, while bounding bucket storage by O(n).
Both forward and reverse adjacency sweeps retain their complete tie order.

The active public comparison covers **535** AMD/relabel cases. It compares
ordered adjacency rows with the allocating reference and compares both complete
MCS outputs with the allocating and newly promoted pooled implementations.
The pooled reference is given fresh scratch per pattern, then may reuse it for
that pattern, matching an isolated worker. Exhaustive small-graph/order tests
also compare against an independent Boolean elimination graph and symbolic
score, including every graph and ordering through five vertices.

Flat adjacency, factor columns and transpose scratch use O(n + nnz + nnz(L))
space. The largest existing structural factor limits remain unchanged. Buffers
belong to the invocation or the current reconstruction; no production cache
persists across input patterns. Peak sizes remain far below the 4GiB cap.

## Other exact runtime kernels

Integer-degree power evaluations use a bounded invocation-owned table, with
the original `powf` on a miss. Floating fill values use a bounded direct-mapped
cache keyed by exact f64 bits, also calling the original operation on a miss or
collision. The exponent and result arithmetic remain unchanged. The active
regression checks original result bits over ten exponents and 8,192 keys in
multiple access orders, including cache eviction and values above table size.
This does not replace a metric with an approximation or change tie policy.

Residual-core minimum-fill scans have an x86 POPCNT feature-enabled entry.
Runtime feature detection guards that entry; its body and deficiency helper
are the same safe slice/integer operations and logical charges as the portable
path. ARM and other CPUs retain the portable implementation. Previous remote
builds compiled this x86 path successfully, but the present ARM measurements
cannot quantify its native x86 speedup.

Two inherited `Instant::now()` bindings that were described as diagnostic but
were unconditionally present are now explicitly `cfg(test)`. Every clock,
environment switch, corpus loader, frozen reference and timing probe added by
this candidate is compiled out of the production worker. The submitted ordering
uses only its supplied pattern, deterministic ordered arrays and fixed seeds.

## Whole-call paired evidence and generated stress

The paired whole-call screen holds unrelated score-writeback repairs common
to both arms, disables the new terminal window, and toggles only pooled/feral
versus flat/linear representation kernels. Thirty complete permutations match
on ten representative public rows. Minimum times from three alternating pairs:

| Public pattern | Pooled/feral | Flat/linear |
|---|---:|---:|
| pooling_sppc3pq | 0.609742 | 0.568112 |
| mpbp_48 | 0.616145 | 0.599661 |
| faclay75 | 0.430848 | 0.436872 |
| crudeoil_pooling_dt3 | 0.587002 | 0.563021 |
| crudeoil_lee4_10 | 0.768618 | 0.736985 |
| chimera_selby-c16-02 | 0.513775 | 0.497159 |
| crudeoil_lee4_06 | 0.690956 | 0.666152 |
| nuclear104 | 0.755718 | 0.730503 |
| crudeoil_lee4_09 | 0.760500 | 0.727321 |
| arki0016 | 0.691232 | 0.688116 |

These are generally modest whole-call reductions, with one slight regression;
they do not support a universal percentage gain. The common bookkeeping repair
experiment was subsequently removed from production so that the final candidate
keeps the promoted adoption trajectory and changes quality only at its last
strictly accepted window. The exact array/state comparisons remain applicable.

The generated stress set includes random sparse graphs at n=2,048 / 8,000 /
12,000 with several degree scales, a 64x64 grid, and a 2,048-vertex hub. These
are deterministic structural generators confined to the diagnostic module,
not shipped patterns or corpus-recognition cases. Both representation paths
produce bijections; the new terminal output never worsens the exact score.
All **eight** fixtures pass. Maximum observed new ordering time is **1.010291 s**
on the random 8k graph with 127,850 nonzeros and over 46 billion predicted flops.
The 12k fixture has over 58 billion predicted flops and takes **0.755427 s**.
Those one-pair stress measurements held the same common bookkeeping repairs as
the paired kernel screen. They demonstrate relevant larger-fill structure;
they do not establish the hidden cap on an unknown x86 runner.

## Integrated validation and remaining result

The full release regression suite passed **121 active tests**, with 53
diagnostic tests ignored, in **79.59 s** using one test thread. The final
sandboxed `yukon run` passed **all 300 matrices** at **0.792030**, fill
**0.924364**, including the purity/dependency, bijection, repeated-output
determinism and watchdog checks. All **300** emitted flop counts exactly match
the selected terminal screen: **35 better / 0 worse / 265 unchanged** versus
the promoted source. The exact integrated score is **0.792029985254**.
Buckets are 147 at 0.887426, 108 at 0.838650, and 45 at 0.685518.
The complete current submission list was checked again immediately before
submission; `7fd61df` remains the promoted target at **0.842716**.
Generated `results.tsv` is restored, and `git diff --check` passes before
the candidate is committed. Only files under `src/ordering/` are submitted,
with no new dependency or manifest changes. New production code uses the Rust
standard library alongside the already allowed ordering implementation.

A successful local run is insufficient to claim a win. The hidden grader must
pass its enforced two-second cap and report enough reduction from **0.842716**
to clear the promotion threshold. The submission identifier and actual hidden
outcome will be recorded after grading. No promotion is inferred from the
public screen or these development-host timing measurements.
