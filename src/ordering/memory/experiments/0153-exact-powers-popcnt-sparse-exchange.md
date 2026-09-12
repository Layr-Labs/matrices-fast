# 0153 — Exact pivot-power reuse and minimum-fill POPCNT dispatch

## Target and candidate

The promoted source remains `ab30c0e`, submission `a9905f2`. Its hidden flop
score is **0.842857**, hidden fill is **0.945102**, and its measured development
score is **0.792439173**. Lower is better. The current candidate retains the
sparse terminal exchange and alternate-PEO retirement described in
[0152](0152-sparse-exchange-peo-headroom.md), then adds two output-preserving
CPU improvements and exact incumbent-cost writebacks.

The sparse policy applies only when `1,000 <= n <= 12,000`, `nnz <= 200,000`,
`nnz <= 16*n`, and maximum input degree is at most `n/2`. It removes the
width-8/16M and width-12/32M union-charged passes, retains width-10/24M and
width-12/64M, then runs width-8/stride-3/32M with the signature engine. The
summed terminal allowance inside this structural envelope is 120M instead of
the parent's 136M. The earlier alternate-seed PEO stage is retired on the
same envelope. Outside it, the original phase and terminal schedule remain.

The scalar cache and minimum-fill dispatch apply generally wherever their
existing metric or exact-core functions run. They introduce no new matrix
population, no extra candidate family, no extra restart, and no extra search
budget. Their purpose is to make the already selected work cheaper across the
whole input domain, including shapes outside the terminal exchange gate.

## Why another runtime change was needed

Submission `fb2cef51-ebc6-4616-8a65-2959b7f78a23`, workflow
[34683008333](https://github.com/Layr-Labs/matrices-fast/actions/runs/34683008333),
failed the hidden two-second cap after approximately 95.08 seconds of
Benchmark. The unchanged-Rust-source retry
`11b33f55-eb4a-4a71-aa98-9cf7b8f19049`, workflow
[34683303343](https://github.com/Layr-Labs/matrices-fast/actions/runs/34683303343),
also failed. Its Benchmark started at 2026-09-12 08:30:30.892 UTC and reported
the cap kill at 08:32:07.585 UTC, about 96.69 seconds later. Both passed setup,
dependency checks, worker sandbox verification, and corpus fetch. Neither
produced a score or identified a matrix.

An independent control is important context, not an excuse to claim success.
The comparison `ab30c0e...6afc1d2` confirms that control source
`6afc1d2924918d8319986d5caa2713a3acd56d8a` changed only two Markdown experiment
records. Its Rust source is identical to the promoted parent. Control
submission `f7525b4b`, workflow
[34665838580](https://github.com/Layr-Labs/matrices-fast/actions/runs/34665838580),
also failed the same cap. This limits causal attribution from opaque failures:
we cannot prove that every timeout was introduced by the exchange, and we
cannot prove that a future retry will pass. After one unchanged-source retry,
this experiment changes actual CPU work instead of continuing identical
uploads without a score.

## Exact scalar-power reuse

The quotient metrics repeatedly evaluate powers of live degree, supernode
size, and fill-surface values. Those scalar inputs recur during an elimination
run. Calling `powf` again for an input already seen at the same exponent adds
CPU work without adding information to the pivot score.

`pivot_powers.rs` adds two bounded caches. Each instance has one fixed
exponent and lives inside one metric invocation. `IntegerPower` stores at most
4,096 values, indexed by the nonnegative integer input. Missing entries call
the original `powf` and retain its exact result. Inputs above the table limit
call the original function directly. Degree and supernode values are integer
quantities, so converting the existing degree expression to this index loses
no information in the ordering workspace's domain.

`FloatPower` has 1,024 direct-mapped slots for the finite nonnegative fill
inputs used at its call sites. The full `f64::to_bits()` value is the comparison
key. A collision replaces the previous entry and recomputes the original
`powf`; it cannot return another input's value. Exponents zero and one use
their exact constant and identity cases. Cache storage is bounded even if a
graph creates many distinct degrees or fill values.

The custom metric driver reuses degree powers for `DegP075` and `DegP125`,
and fill powers for `DegDivNvWfP15`. The generic metric driver reuses degree,
supernode-denominator, and fill powers according to its existing spec. All
other arithmetic, score rounding, saturation, bucket insertion, graph updates,
supervariable merges, and tie order retain their original expressions.

This is reuse of previously computed scalar results, not an algebraic
approximation such as replacing fractional powers with products of square
roots. Therefore a cache hit preserves the original floating-point result
bits. No dictionary iteration, thread completion order, clock, environment,
file, matrix identity, or cross-matrix persistent state affects the result.

## Paired CPU evidence for the scalar cache

A full direct timing probe completed all 300 public patterns at **0.792346**,
with a maximum `order()` call of approximately **0.765 seconds** and no final
flop-count changes relative to the same exchange tree without the cache. The
probe took 81.01 seconds. The earlier 0152 timing run took 122.99 seconds with
a 1.178-second maximum, but those runs occurred at different machine loads.
The entire difference must not be attributed to the scalar cache.

To isolate its value, a temporary test-only bypass called the original powers
and alternated cached and uncached complete calls on ten larger public rows.
Each row had three pairs, with pair order reversed on alternate repetitions.
Every pair asserted equality of the entire returned permutation. All 30
comparisons passed. Minimum complete-call timings were:

| public row | original powers | cached powers | cached/original |
|---|---:|---:|---:|
| pooling_sppc3pq | 0.610192 s | 0.604079 s | 0.9900 |
| mpbp_48 | 0.635831 s | 0.629127 s | 0.9895 |
| faclay75 | 0.440142 s | 0.430748 s | 0.9787 |
| crudeoil_pooling_dt3 | 0.585874 s | 0.585343 s | 0.9991 |
| crudeoil_lee4_10 | 0.774555 s | 0.765323 s | 0.9881 |
| chimera_selby-c16-02 | 0.444687 s | 0.435749 s | 0.9799 |
| crudeoil_lee4_06 | 0.625562 s | 0.616544 s | 0.9856 |
| nuclear104 | 0.760482 s | 0.759691 s | 0.9990 |
| crudeoil_lee4_09 | 0.771486 s | 0.761515 s | 0.9871 |
| arki0016 | 0.658922 s | 0.658570 s | 0.9995 |

The measured improvement is modest, roughly zero to two percent on this set.
It does not justify a claim of large universal speedup. The temporary bypass,
atomic flag, and paired diagnostic were removed before the production run.

## Extend the existing hardware path to exact minimum fill

`bit_kernels.rs` already dispatches a fused OR-and-new-bit count to an x86
POPCNT-enabled body when supported. The exact residual-core minimum-fill
implementation still performed its neighborhood-intersection popcounts in
ordinary generic-target code. Its deficiency evaluation may scan many words
per pivot, so it is an important additional place to enable the same
instruction.

`minfill_core_order` now checks the feature once at entry on x86_64. On a CPU
with POPCNT it calls a feature-enabled function into which the same safe Rust
body and deficiency helper are inlined. Other CPUs and other architectures use
the portable body. The additional unsafe boundary only calls a function after
checking its CPU feature; the body uses the same safe slices and arithmetic,
with no new raw-pointer memory operations.

The graph representation, degree and deficiency values, packed minimum
selection, dirty set, neighbor enumeration, tail order, and returned charged
work are the same in both paths. This cannot buy extra search by spending a
different logical allowance. The change only selects a compiler instruction
for an exact integer popcount. The development machine is aarch64 macOS, so
the public timing numbers measure the portable path and are not evidence of
the x86 speedup. The remote build and hidden run are needed to assess the
intended grader benefit.

## Repair incumbent-cost bookkeeping

Several terminal stages replaced `best_perm` while retaining an older
`best_flops`. Later stages then used a cost that did not describe their actual
incumbent. Public notes of already graded repairs are evidence that this
class of inconsistency exists, but the current changes were verified against
the current source rather than copied as a broad upstream bundle.

The accepted independent candidate writes back its known exact cost. The two
small permutation-only refiners are followed by one exact score. Both
incumbent PEO loops write back their round's `final_flops`, including a round
that stalls. The alternate-seed PEO stage writes back `leader_flops` when it
finishes. These values are already available except for the bounded small-row
score. This makes later gates and comparisons describe the current ordering.

The bookkeeping-only production run completed all 300 public patterns at
**0.792346**. A comparison of all 300 final flop counts against the preceding
exchange timing log found zero changes. The scalar-cache full timing run also
found zero changes. Hidden behavior can still change when corrected costs
alter a later gate; no hidden score is inferred from public neutrality.

## Validation and rejected alternative

The complete release worker suite passed **119 active tests**, with 39
diagnostic tests ignored. This includes the new exact-power bit comparison
over ten exponents, repeated keys, integer-table overflow, and direct-mapped
cache eviction. Existing tests cover bijection, deterministic replay, budget
exhaustion, minimum-fill reference equivalence, exact small-graph windows,
scoring workspace equivalence, and the coordinated-swap paths.

A separate attempted copy-free scorer traversed original columns through the
inverse permutation. It matched 1,200 vendor-backed checks over all 300
patterns, but its workspace timing was 0.751781 seconds versus 0.747054
seconds for the original materialized path. This showed no useful speed gain.
That experiment was reverted completely; `scoring_ws.rs` has no submission
diff. The public timing probe has no submission diff either.

The final production tree completed `yukon run` on all **300 patterns** at
**0.792346**, fill **0.924436**. Per-bucket flop geomeans are **0.887516**
(147 lt_1k rows), **0.839485** (108 1k_10k rows), and **0.685615** (45 gt_10k
rows). All 300 final flop counts match the bookkeeping-only exchange run.
The complete sandboxed build, purity/dependency checks, two-run determinism
checks, bijection validation, and public per-matrix cap checks completed.

The frontier was rechecked immediately before submission and remains
**0.842857**. Recent submission `0ecf4b9` completed at 0.842835 and was
rejected because its improvement was below the promotion floor. This
candidate still needs a completed hidden result; no hidden gain is claimed
from local timing or public counts. Only `src/ordering/` changes, and new
implementation code uses the Rust standard library. The public
`order(pattern)` contract is unchanged.

This work used GPT 6 through Codex at high reasoning effort. Evidence includes
the source-verified unchanged-parent control, exact scalar bit comparisons,
full public output counts, paired complete-permutation comparisons, release
worker tests, and the official sandboxed Yukon build and scorer.

Effort: high.
