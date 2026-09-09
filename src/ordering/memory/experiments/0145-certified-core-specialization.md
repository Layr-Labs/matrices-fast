# 0145: certified easy-region peeling and residual-core solver dispatch

Base: promoted `691aad43e05fec0d4851d0a4943df33cc232d500`, hidden 0.844528.

## Verified local outcome

| Metric | Control | Candidate |
|---|---:|---:|
| Public-dev score | 0.798268039736831 | **0.798032511126144** |
| Official local rounded score | 0.798268 | **0.798033** |
| Candidate fill ratio | - | **0.927127** |
| Paired worst order call | 1.149818 s | **1.133252 s** |
| Paired total order time, 300 patterns | 133.713477 s | 135.457664 s |

**38 better / 0 worse / 262 unchanged**. The paired test alternated which path
ran first and checked every control count against the saved accepted baseline.
All **131 active tests passed**; 42 optional probes are ignored normally.
The relevant decomposition, solver, canonicalization, and paired probes ran
explicitly. Official local `yukon run` passed all gates and agreed.

Candidate bucket FLOP ratios: **0.887622 / 0.840284 / 0.699151**.
Total ordering time increased about 1.3%; no overall runtime speedup is claimed.
Remote evaluation is pending.

## Mathematical basis

For a simplicial vertex v, its live neighbors form a clique. Moving it ahead
of a non-neighbor commutes. Moving it ahead of adjacent w changes pair cost
from `(d(w)+1)^2+d(w)^2` to `(d(v)+1)^2+d(w)^2`, because `N[v]` is contained
in `N[w]` and therefore `d(v)<=d(w)`. The residual graph after the pair is
unchanged. Inductively:

`F(certified_prefix ++ incumbent restricted to core) <= F(incumbent)`.

The prefix adds no fill, so the residual is exactly induced and any core order
has the exact split `prefix_flops + core_flops`. This is stronger than merely
preserving a global optimum, but does not justify an arbitrary new core order.
The implementation seeds every core solver with the restricted incumbent.

Independent exhaustive checks through n=5 examined all 1,099 simple labeled
graphs and 124,469 permutations without a counterexample. Checked-in tests
cover dominance, the exact split, forest/chordal attachments, invalid inputs,
budgets, cycle cores, and serial/parallel result identity.

## Implementation and dispatch

`leaf_core.rs`:

1. Deterministically peel isolates/leaves and explicitly verified simplicial
   degree-two vertices. Check the neighbor edge in original sorted CSC; no
   fill is inserted. Recheck only after a neighbor disappears.
2. Require 20% removal, core n <= 12000, and core directed nnz <= 200000.
   Check core size/density before allocating/copying its CSR/CSC data.
3. Restrict the final incumbent to the induced core without reordering it.
4. If all core degrees equal two, every component is a cycle and every order
   has the same column-count multiset: retain the seed without search.
5. Otherwise evaluate width-8 and width-10 exact windows independently from
   that same seed: four sweeps, step 3, budgets 32M and 48M.
6. Reuse the existing bounded candidate executor and exact core scorer.
   Select in deterministic task order, lift, and require a strict full-pattern
   score improvement.

The late hook follows every existing incumbent mutation, including the
independent-set deferred candidate and prior final windows. The original
finished ordering remains the safety floor. This is a specialized additional
candidate, not an unsupported early replacement of the whole portfolio.

## Evidence-driven selection

Degree-one peeling removes 30.7% of public vertices but only 10.4% of nonzeros.
Twenty-one of 45 large patterns leave at most 12000 core vertices. A low
four-core is only a feature, not proof that elimination is easy.

Independent whole-graph BFS verified: 266 connected patterns, 32 disconnected
with one cyclic component plus trivial extras, and only two with multiple
cyclic components. A preliminary inconsistent bipartite census was withdrawn;
the corrected result has only star/edge components, not nontrivial complete
bipartite components. General disconnected-component portfolio assembly was
therefore not prioritized.

| Core control | Score |
|---|---:|
| Leaf peel + width12/step5/32M | 0.798212221 |
| Verified degree2 + same solver | 0.798211428 |
| Verified degree3 + same solver | 0.798211428 |
| Degree2 + width8/step3 | 0.798080776 |
| Degree2 + width10/step3 | 0.798069371 |
| Degree2 + width12/step5 | 0.798122832 |
| Degree2 + width14/step5 | 0.798195731 |
| Degree2 + serial greedy | 0.798238314 |
| Degree2 + four-stream greedy | 0.798190793 |
| Selected independent width8/10 minimum | **0.798032511** |

Degree three and wider/more-threaded solvers were not automatically better.
The chosen kernels keep the measured extra stage cost small.

## E-graph-inspired probe

Production e-graphs/canonicalization were not added. Test-only
`signature_canonical.rs` uses exact boundary-aware relabeling after color
refinement and bounded individualization. Full transformed-key equality is
required; interior or degree summaries alone are unsafe. Caps are width12,
64 boundary signatures, 64 candidate mappings, 100k work/attempt and
2M work/pattern.

On the pre-specialization baseline: 11070 raw misses, 4691 canonicalized,
350 additional matches, 137103087 diagnostic work units, and 6329 work-cap
refusals. This was insufficient evidence to add production complexity.
Any future cached canonical witness must preserve caller tie semantics or be
treated as a separately scored candidate.

## Engineering safeguards

An agent accidentally reset the accepted CPU code while handling mixed line
endings. The parent restored the exact promoted file and reran CPU-state tests
before these experiments. Only ordinary module declarations are used; a
prototype's forbidden path annotation was removed before gated validation.
All reported final tests/benchmarks ran in the required Ubuntu WSL sandbox.

[Index](../index.md) | [Log](../log.md)
