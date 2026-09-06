# 0068: Small-core relabel portfolio and exact pruning

2026-09-05. Base: official 5734aba / 72aaadad, remeasured locally.

Four structurally gated AMF tickets on the degree-three residual core: reverse
labels and ascending (degree, index) labels, each at dense_alpha 2.5 and 10.
Eligibility: 8 <= core n <= 4000 and core nnz <= 30000. Evaluate these after
the existing recursive refinement and accept only a strict exact improvement.
The fixed prefix plus exact core objective equals the full objective.

Also rank the existing extra-depth passes with exact core scores and avoid
redundant full-graph symbolic scoring. Retain earlier local experiments' small
paired-swap/neutral-walk refinement with bitset scoring, bounded to n 12..300
and nnz <=3000, and forest certificate. Strengthen early score rejection with
the unavoidable neighbor-clique suffix bound: sum(k^2, k=1..d) + remaining-d.
These inherited refinements supply two small wins; the new core relabels supply
two medium wins. No gates depend on input identity.

Full trusted sandboxed development run: 300 matrices, score
0.833148 -> 0.832826; fill 0.939266 -> 0.939212; 4 wins, 0 losses, 296 ties.
Buckets: 0.889767 -> 0.889734; 0.866830 -> 0.865792; 0.765421 unchanged.

- rsyn0830m04m: 193427 -> 181793 flops (-6.015%).
- rsyn0820m04m: 179351 -> 167666 flops (-6.515%).
- sporttournament18: 13723 -> 13695.
- korcns: 6751 -> 6728.

Tests: 62 passed, 18 ignored diagnostic probes. Includes all 1024 simple
five-vertex graphs and all 120 permutations (122880 cases), checking cutoff
classification at exact score -1, exact score, and exact score +1 against the
trusted symbolic scorer; 5888 perturbed corpus scores and identical refinement
outputs on all 92 eligible corpus patterns. The exact-core-only intermediate
retained the earlier score 0.833138; no separate flop improvement claimed.

Local scores are not hidden scores. Official evaluation is pending. Prior
related additive refinements failed the hidden time cap, so local correctness
and a local pass do not establish hidden runtime safety.

Generated stress validation: 10 cycle/chorded-core-with-leaves patterns, n 32..8000, including residual-core gate boundaries. Two sandboxed runs per binary per pattern, 4 GiB address-space cap and 2 s subprocess timeout; all 40 calls pass bijection and repeated-permutation equality. Maximum observed whole-process wall time: control 0.349664 s, candidate 0.334726 s. This is a local generated sample, not a hidden-cap guarantee.
