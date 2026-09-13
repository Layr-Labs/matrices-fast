# 0144: repeated boundary signatures and final offset coverage

Base: `b5783a60348aa495bfbb6a32949e380d8cce46d8`, official hidden 0.844675.
The independent-set-first family is upstream work, retained in this candidate.

## Verified local results

- Remeasured base: **0.798477** on all 300 public patterns.
- Candidate: **0.798268**; exact aggregate **0.798268039736831**.
- Fill ratio: **0.927268**.
- **48 better / 0 worse / 252 unchanged** versus the remeasured base.
- Buckets: **0.887622 / 0.840735 / 0.699402**.
- **118 active tests pass**, 38 optional probes ignored by the ordinary run.
  Relevant timing, offset, budget, and CPU probes were run explicitly.
- Official local `yukon run` passed all gates and reproduced the score.
- Latest worst timing-probe call: **1.130 s**. Base measured 1.083 s.
  Single-run timing is noisy: earlier unchanged code measured 1.027 and 1.142 s.
- Remote evaluation pending at the time this entry was prepared.

## Findings that selected this direction

Eleven assessment streams covered score leverage, latency, resource use,
practitioner forums, solver communities, ordering maintainers, and CPU library
design. Numerical BLAS itself cannot change the score of a fixed permutation.
The useful common idea was retaining compact exact representations and
amortizing preparation.

New counters disproved the inference that short window runtime meant complete
search: out of 238 invoked calls per width, 68 width-8, 77 width-10, and
117 width-12 calls exhausted the inherited logical budget.

With original prices retained, the signature engine recorded **31,587 local
queries, 22,062 exact cache hits, and 386 trivial certificates**, reproducing
all 300 pre-substitution FLOP counts.

## Implementation

- `rgreedy/window_signatures.rs`: exact internal adjacency plus the histogram
  of outside-neighbor incidence masks. A subset-zeta transform supplies union
  boundary cardinalities. Outside vertex IDs and outside-to-outside edges are
  irrelevant while the boundary is frozen; multiplicities are not.
- Reusable scratch is resized only as needed. Exact local-order solutions are
  cached within a descent, mapped back to current vertex IDs, and verified by
  full key equality. No cross-matrix or persistent result cache exists.
- Memo insertion is capped at 2048 entries and a 4-MiB accounting allowance,
  plus normal container overhead. Complete movable cliques with universal
  outside neighbors have a closed-form cost and skip DP entirely.
- The union solver remains the fallback when its estimated work is cheaper.
  Existing search stages retain their old logical prices, including on hits.
- Only the added final stage uses extraction/incident/DP work pricing: cached
  or trivial problems do not pay for an unexecuted DP. All work choices use
  deterministic allowances, not clocks.
- `bit_kernels.rs` and `Game`: fused OR/new-bit count with runtime-guarded
  hardware POPCNT for at least eight words, otherwise portable Rust. Sparse
  and known-clique paths, logical charges, buckets, and random streams remain
  unchanged. AVX2-assisted merging lost to POPCNT in microbenchmarks and is
  test-only.
- `core_lift.rs`: omit live-live edge-membership queries whose answers follow
  from the maintained adjacency invariant. Exhaustive and synthetic tests
  preserve prefixes, costs, core CSCs, and budget behavior.

The final quality pass is **width 12, four sweeps, step 5, 64M work** at
`6 <= n <= 12000 && nnz <= 200000`. Its offsets are 0/5/10/3. It runs
**after deferred independent-set acceptance**, so it refines the actual output.
Every replacement still requires a strict full exact-score improvement.

## Controls and alternatives

On the earlier f7f60dc base, extra width-12 step-5 coverage projected 0.804480
with 36 wins and no regressions. After incorporating b5783a6, the same coverage
under old prices measured **0.798317**, 37 wins and no regressions. Signature
work pricing improved this to **0.798268**, 48 wins and no regressions.

Width-14 and step-1 controls were measured but did not beat step-5 width-12
coverage at comparable cost. Width-14 remains supported and tested, not
selected by the production schedule. Hardware POPCNT microbenchmarks over
16-188 words measured approximately 1.79-1.87 times portable throughput;
this is not a whole-ordering speedup claim.

## Correctness and measurement cautions

Cache keys must include current fill and boundary multiplicity. Same interior
topology alone is insufficient. Cross-invocation warm starts violate the
pattern-only contract. More funded work can change later search trajectories;
do not infer dominance over an older multi-pass result without measurement.

The 2-second cap is per matrix, not a cap on summed corpus time. Do not compare
corpus-total milliseconds with that limit. Cargo can place the first `COUNTS`
record after the test-name prefix; parsers must find the record marker and
assert exactly 300 rows.

Sources: [SuiteSparse #311](https://github.com/DrTimothyAldenDavis/SuiteSparse/issues/311),
[BLASFEO #118](https://github.com/giaf/blasfeo/issues/118#issuecomment-601610334),
[OpenBLAS #1092](https://github.com/OpenMathLib/OpenBLAS/issues/1092#issuecomment-279299879),
[subset transform](https://arxiv.org/html/cs/0611101#S2.SS2).
Forum/runtime evidence is distinguished from measured exact-FLOP gains.

[Index](../index.md) | [Log](../log.md)
