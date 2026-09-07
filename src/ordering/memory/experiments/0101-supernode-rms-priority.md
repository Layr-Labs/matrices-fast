# Fixed-pass supernode RMS priority from a block factorization cost

Effort: high

## Candidate status

This is an unmeasured remote-only candidate based on promoted commit
`e88316db49ddf27aa5b862c41386f9c1fe6aca0f`. The baseline official score at
the pre-experiment check was 0.850740. This candidate does not claim that
score as its own, and no claimed score is supplied. The operator prohibits
local candidate execution, so no setup, build, test suite, benchmark, timing
probe, or agent worker has run locally.

The production source for this experiment starts from the promoted baseline,
not the failed epoch-stamp or six-pivot experiments. The previous MINL changes
are preserved as historical documentation but are not compiled into this
candidate. An unchanged-production-source remote control was also submitted
as `da1af78d-5851-4d36-80ba-c6bf93375aa6` to assess current baseline execution.
Its result is separate evidence and must not be attributed to this new metric.
That unchanged-source control also timed out (workflow 34162012605), so the
base has demonstrated runtime instability in this run series. This candidate
is not claimed to resolve that instability; it tests a different priority
under the same ticket count and has no verified resource margin.

## Mathematical model

Consider an idealized clique supernode representing s consecutive pivots,
all sharing d external neighbors. Eliminating that block contributes column
widths d+s, d+s-1, ..., d+1 to the factor. Its predicted sum-of-squares work is:

```text
sum(i=1..s) (d+i)^2
  = s*d^2 + d*s*(s+1) + s*(s+1)*(2*s+1)/6.
```

Dividing by the number s of original variables represented in the block
gives the mean squared width per eliminated variable:

```text
M(d,s) = d^2 + d*(s+1) + (s+1)*(2*s+1)/6.
```

The ranking key is sqrt(M), a root-mean-square column width. The square root
is monotone, so it preserves rankings by mean predicted block work before
quantization. It puts the priority into degree-like units for the existing
bucket queue instead of introducing a cubic-range total block-work score.

For a singleton, M(d,1) = (d+1)^2 and the priority equals d+1. More generally,
the RMS lies between d+1 and d+s. This bounds its scale by the possible
column widths of the model block and avoids an unnecessarily broad dynamic
range. The formula uses fixed-count arithmetic and one square root; it does
not enumerate the s pivots or construct a dense block.

## What is exact and what is heuristic

The closed form is exact for the stated clique-supernode model. The quotient
ordering workspace supplies an AMD-style loose external-degree estimate and
a represented-variable count. Those are the values used as d and s. The
remaining arbitrary sparse graph is not necessarily identical to the model,
and its future fill is not known exactly from these two quantities alone.

Accordingly this is a heuristic pivot priority derived from a factorization
cost identity, not an exact global cost oracle and not a claim to solve the
NP-hard ordering problem. It may produce a worse final order than the metric
it replaces. The independent full-permutation scorer remains responsible
for comparing generated candidates with the incumbent.

The chosen normalization is per original variable rather than total block
work. That avoids penalizing a supernode simply for representing more pivots
without considering how many eliminations its cost purchases. Alternative
normalizations might have different behavior, but they are not included in
this one-dimensional experiment.

## Implementation

`custom_metrics::ScoreVariant` gains `SupernodeRms`. A small helper computes
M(d,s), and the re-insertion match applies its square root. The variant is
included in the degree-family list so that `ws.degree` continues to receive
the unmodified AMD loose-degree estimate, not the new ranking key. This
distinction is required for the subsequent quotient-graph bookkeeping.

The existing input validation, graph construction, pivot selection machinery,
supervariable merging, element creation, raw-degree cap, score quantization,
bucket queue, and permutation reconstruction remain unchanged. The formula
uses no random seed, graph identity, matrix name, environment value, timing
measurement, external file, or private evaluation information.

The represented count is positive at this re-insertion point by the existing
`nvi > 0` branch. The degree estimate is nonnegative and capped by the number
of remaining variables outside the supernode. Arithmetic is performed in
f64, consistent with the surrounding score formulas. For the benchmark's
index range the polynomial is finite; no integer exponentiation is used in
the production priority calculation.

## Fixed computation envelope

Within the existing post-cascade light-envelope block, this candidate replaces
the four unrelabelled `DegP125` tickets with `SupernodeRms` for n below 10000.
Their dense thresholds remain exactly 10, 5, 2.5, and 1. The existing outer
gate requiring the heavy arm and fewer than `METRIC_LIGHT_MAX_NNZ` nonzeros
is unchanged. The four old tickets are substituted, not supplemented.

For dimensions at least 10000, the prior tickets remain untouched. All
relabelled DegP125 tickets and other uses of that variant are also unchanged.
No new pass, restart, candidate score evaluation, subtree iteration, MINL
iteration, or operation allowance is added. The number of queued portfolio
jobs in the edited block is identical before and after the patch.

Equal pass counts do not prove equal runtime. A different pivot priority
can change quotient-graph evolution, the winning incumbent, retained donors,
and later refinement trajectories. The remote cap must still pass. The
sub-10000 gate limits the changed region but is not a certificate that all
affected inputs are fast. No claim is made from timing data on another host.

## Source-level tests, not executed locally

A closed-form test compares M(d,s) with an explicit sum of squared widths
for degrees zero through 64 and supernode sizes one through 32. It uses a
relative floating-point tolerance, verifies the degree-scale bounds, and
checks the exact singleton identity on these integer fixtures.

The new variant is also added to the existing synthetic bijection and
determinism tests for quotient metrics. Those tests exercise the same
ordering entry point as the other score variants rather than only the
arithmetic helper. They remain unexecuted locally under the operator's
constraint. Source inspection and scoped whitespace checking do not replace
them or establish a green build.

The official scoring workflow may run a different set of gates than every
repository unit test. Only checks actually reported by the workflow should
be described as executed. A successful upload establishes a queued candidate,
not a correctness result, timing result, score, or leaderboard improvement.

## Provenance, scope, and interpretation

This formula and slot substitution are developed from the block sum-of-squares
identity above. No unpromoted solver's source patch is incorporated. The
existing promoted quotient implementation and portfolio remain credited
through their repository history. The model and harness attribution identify
the agent implementing this specific experiment, not authorship of the base.

Production changes are restricted to `src/ordering/custom_metrics.rs` and
the ticket selection in `src/ordering/mod.rs`. Documentation stays under
`src/ordering/`. No scorer, harness, corpus, dependency, manifest, lockfile,
build configuration, or grader rule is edited. The development corpus remains
an LFS pointer in the source-only checkout.

If this candidate completes but loses, that is evidence against this exact
replacement under the evaluated distribution; it is not evidence that the
algebraic identity is false. If it times out, the equal-ticket envelope was
insufficient to preserve runtime safety. If it improves the official score,
the improvement must still exceed the promotion margin. No outcome is
inferred before the independent remote result is available.
