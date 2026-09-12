# 0155 — Small alternate-PEO envelope and linear sorted permutation

## Model, comparison, and resulting policy

Implemented with GPT 6 through Codex at high reasoning effort.

Effort: high.

The promoted target remains `ab30c0e`, submission `a9905f2`, hidden flop score
**0.842857**, hidden fill **0.945102**. Its complete public score is
**0.792439173**. This experiment starts from local `b1f1328`, the flat
completion and exact linked MCS candidate of 0154, and makes a larger runtime
trade-off than another scalar-kernel optimization alone.

The production alternate-seed PEO dimension limit decreases from **50,000**
to **1,000**. The existing 4M-unit ledger, eight retained seeds, strict exact
acceptance, small-graph behavior, and terminal exchange envelope are retained.
Larger graphs still receive incumbent-completion PEO extraction, its existing
oversize/large ledgers, and donor-subtree transplantation. The already-paid
donor orderings are retained under the same rule. This change removes one
whole search phase on larger inputs instead of granting it another separate
allowance.

The second production change is an invocation-local sorted-permutation
workspace. It builds the same sorted CSC pattern as feral in O(n + nnz) time
per permutation, after one O(n + nnz) transpose setup. It preserves both
column pointers and every row-index entry exactly, including tie-relevant
neighbor order. It replaces the full-pattern permutation calls in
`leader_order`; residual-core helper functions retain their existing paths.

The sparse terminal schedule remains C24 + D64 + E32: width-10/24M,
width-12/stride-5/64M, width-8/stride-3/32M. It applies on the same structural
envelope as 0152: `1,000 <= n <= 12,000`, `nnz <= 200,000`, `nnz <= 16*n`,
maximum input degree at most `n/2`. Total allowance is **120M**, versus the
promoted parent's **136M**. No new window family or extra pass is added.

## Failure that motivated the larger work reduction

Submission `aac9f0ec-8457-4119-aae7-cf79d9a3c63b`, local source `b1f1328`,
passed setup and sandbox checks, then failed the hidden two-second per-matrix
cap in workflow
[34685112302](https://github.com/Layr-Labs/matrices-fast/actions/runs/34685112302).
Benchmark started at 2026-09-12 09:13:26.643 UTC and reported the kill at
09:15:01.293 UTC, approximately **94.65 seconds** later. No hidden score or
matrix identity was exposed. Earlier 0153 also failed the cap.

The flat completion and linked MCS kernels have measured value: 49% less
reconstruction time and 37% less two-direction MCS time in their isolated
public paired probes. Their full public scores were unchanged. Nevertheless,
those kernel percentages did not establish sufficient whole-call headroom.
The next change therefore removes actual candidate-generation work.

An unchanged-parent control also failed previously: comparison
`ab30c0e...6afc1d2` changed only Markdown, and control submission `f7525b4b`
failed the same cap in workflow
[34665838580](https://github.com/Layr-Labs/matrices-fast/actions/runs/34665838580).
This limits causal attribution from opaque timeouts. It does not justify
calling another unscored upload successful or relying on identical retries.
This experiment changes the work done and will use the hidden grader as the
final check.

## Measuring the alternate-seed phase's current marginal value

The existing larger PEO comments described wins from an earlier frontier.
They are not sufficient evidence that those same seed chains still improve
the current portfolio after independent-set lifts, reduction cores,
incumbent-completion PEO, transplants, MINL, and terminal windows.

The phase trace on the current full public corpus showed substantial CPU
spend on many larger rows with little remaining score gain. To test the
trade-off, change only the structural dimension envelope and run the entire
pipeline. The resulting final counts are identical to 0154 on **299 of 300**
public patterns. There are no improved rows from the retirement and one
regression:

| public row | n | input nnz | 0154 flops | restricted-alt flops |
|---|---:|---:|---:|---:|
| maxcsp-ehi-85-297-71 | 2,372 | 207,996 | 1,110,971,341 | 1,112,710,800 |

This is a real loss, about 0.157% on that row, not an output-preserving
optimization. The full weighted public score moves from **0.792346128627**
to **0.792349776881**, an absolute cost of approximately **0.00000365**.
The resulting score still improves over the promoted source's public
**0.792439173** by approximately **0.00008940**. The hidden aggregate and
its promotion floor remain separate checks.

This dimension policy is general: small graphs retain a bounded alternative
search; larger graphs use the already retained incumbent cleanup and donor
transplant routes. Production does not identify maxcsp or any other matrix.
The public row is named only to account for the measured regression.

## Paired whole-pipeline timing of the retirement

A test-only dimension override restores the 50k envelope for the control
arm while keeping the same exact sorted-permutation and flat-PEO kernels in
both arms. Ten public medium/large cases have three old/new pairs each,
alternating arm order. Timing includes the complete `order()` invocation.
Both returned permutations are validated and independently scored after
timing. All **30 complete-permutation comparisons agree** on this sample.

| public case | 50k envelope | 1k envelope | final flops in both arms |
|---|---:|---:|---:|
| pooling_sppc3pq | 0.570900 s | 0.519992 s | 325,595,394 |
| mpbp_48 | 0.613744 s | 0.531363 s | 16,144,599 |
| crudeoil_pooling_dt3 | 0.572642 s | 0.491511 s | 33,883,414 |
| crudeoil_lee4_10 | 0.765754 s | 0.710832 s | 184,173,410 |
| chimera_selby-c16-02 | 0.462877 s | 0.463528 s | 2,299,702 |
| crudeoil_lee4_06 | 0.637429 s | 0.635967 s | 32,947,837 |
| nuclear104 | 0.769678 s | 0.695760 s | 78,332,024 |
| crudeoil_lee4_09 | 0.777077 s | 0.717590 s | 130,655,525 |
| arki0016 | 0.715716 s | 0.716517 s | 836,006 |
| gams05 | 0.736404 s | 0.710849 s | 3,259,322,396 |

Cases where the larger phase actually ran save approximately **4–14%** of
total ordering time in this measurement. Cases already excluded or refused
by the previous gates have neutral/noisy timings, as expected. This is more
substantial whole-call evidence than the previous isolated kernel gains.
It is still measured on the ARM development host, not a guarantee for hidden
x86 matrices or a universal speedup percentage.

The full direct probe completed **all 300** public patterns at the exact
score above, with a maximum `order()` time of **0.727 s**, taking **82.02 s**.
Whole-run comparisons across machine loads are not treated as causal; the
paired retirement probe provides the direct CPU evidence.

## Linear sorted-permutation construction

`SortedPermutation` borrows one immutable full pattern for its lifetime.
Count entries per original row, prefix-sum those counts, and build a flat
list of original columns for each original row. That is the stored transpose.
Setup happens once within this `order()` invocation, after the existing AMD
fill-free certificate can return early.

For a requested permutation, write its inverse, form new column pointers
from the corresponding original column lengths, and reset output cursors.
Visit new row indices in ascending order. The permutation identifies the
corresponding old row; the stored transpose identifies each old column that
contains that row. Its inverse gives the new column. Append the new row
index at that column's cursor.

Every original entry is mapped to the same new row and column as feral.
Each output column gets the same entry count. Ascending new-row visitation
makes each output column sorted without comparison sorting. The argument
holds for nonsymmetric input as well as symmetric patterns, and preserves
duplicate entries. This exact sorted representation matters because PEO
adjacency visitation influences tie order; an unsorted permutation that
preserves symbolic counts alone would not be equivalent here.

The workspace stores `u32` old-column and inverse indices, plus `usize`
row pointers and cursors. Dimension narrowing is within the already required
i32-indexed input domain. Additional storage is O(n + nnz), roughly
4*nnz + 20*n bytes, far below the 4GiB worker cap. Returned pattern buffers
are still owned, and are dropped by their existing callers.

## Exact comparisons and CPU evidence for permutation construction

The production helper was compared with feral across all 300 public patterns
and four permutations each: identity, reversal, and two fixed relabel tickets.
All **1,200** comparisons preserve column pointers and row indices exactly.
The active synthetic test adds nonsymmetric, unsorted, and duplicate-entry
patterns through eight vertices, including the zero-dimensional case, with
multiple deterministic permutations and repeated workspace reuse.

Three paired kernel repetitions alternate old/new order for each public
pattern and ticket. Aggregate minimum permutation time was **0.204489 s**
for feral and **0.113595 s** for the production helper, with **0.016807 s**
of transpose setup across the corpus. This is about **44%** less permutation
kernel time before setup, with setup amortized across repeated requests.

An earlier whole-pipeline paired screen kept the original 50k alternate-PEO
envelope and toggled only the permutation builder. Thirty complete outputs
matched on ten representative public cases. Whole-call savings were modest
and variable, ranging from neutral/slightly slower to about 6% faster. The
44% kernel result must not be presented as a 44% whole-pipeline gain.

## Other screens rejected

Four small-graph terminal windows, widths 8/10/12/14 with a 16M allowance,
found **zero** new improvements in the complete small public bucket under
the sparse structural gate. They were not added to production.

A previous-score reuse prototype compared a requested permutation exactly
against the scoring workspace's existing inverse mapping. It avoided a new
permutation copy, but found only zero to two hits on most larger sampled
rows and no consistent whole-call speedup in paired tests. It was reverted:
production has no new `previous_score` cell or `has_permutation` scoring
method. Exact score reuse is not assumed beneficial without measurement.

The six fixed-allowance medium terminal variants from 0154 also lost to the
retained width-8/stride-3 pass. This candidate therefore spends less earlier
work and keeps the locally best terminal policy instead of adding passes.

## Validation and remaining uncertainty

Only `src/ordering/` is edited. New production code uses Rust's standard
library and the supplied pattern. It has no clock/environment/filesystem/
network access, corpus identifiers, hard-coded permutations, external cache,
or output dependency on unordered iteration. Diagnostic clock and corpus
access, vendor bypass, and dimension overrides are confined to `cfg(test)`.

The complete release suite passed **121 active tests**, with 45 diagnostic
tests ignored. Official sandboxed `yukon run` passed **all 300 matrices** at
**0.792350**, fill ratio **0.924437**, including the purity/dependency gates
and repeated deterministic executions. The buckets are 147 at 0.887516,
108 at 0.839497, and 45 at 0.685615. The emitted score matches the direct probe.
The submitted patch passes `git diff --check`; generated `results.tsv` is
restored before committing. The hidden grader must both pass the two-second
cap and report a sufficient improvement over **0.842857**. Public counts,
public timing, and a successful build do not establish that result alone.
