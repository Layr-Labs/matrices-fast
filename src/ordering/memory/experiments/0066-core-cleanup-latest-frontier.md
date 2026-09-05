# 0066 — Core cleanup after the next frontier advance

2026-09-06. The first six-return study and its native controls are preserved in [0065](0065-returned-pro-and-core-cleanup.md). Before submission, another solver's promoted source `08a993534b7b2a3b7bb4ebc8a27b3a0b9f946a3d` advanced the official score to **0.867023**. This experiment reapplies the same selected optional residual-core cleanup on the complete newer source.

## Integration

The promoted delta changes search/relabel allocation, simplicial selection, subtree adjacency reuse and k4 screening. The exact low-degree reducer, residual graph, fixed-prefix cost and five core jobs are unchanged. Retain every new upstream policy. Add only the existing k5→k4 cycle on the selected core (n=5…4096, directed nnz≤65536, 16M per kernel), with exact core/full rescoring and strict acceptance.

The new baseline had removed `best_flops = f` from its final terminal acceptance as a dead store. Appending more refinement makes it live again, so restore the assignment alongside the accepted permutation. This keeps the cached score synchronized with the retained incumbent. Only `mod.rs` changes executable behavior relative to 08a9935.

The inherited k4 caller now skips windows with nondecreasing current live degrees. This is heuristic screening, not a certificate: path edges (0,1),(1,2),(2,3),(3,4), seed [0,2,1,3,4], has screened first-window degrees [1,2,2,2], yet first-window cost improves 21→16 under [0,1,2,3]. Later offsets may rescue this complete call. Exactness belongs to the DP on evaluated windows; no new claim of exhaustive window evaluation is made.

## Fresh evidence

| Measurement | Promoted 08a9935 | With core cleanup |
|---|---:|---:|
| Exact public score | 0.8389332841047089 | **0.8388909882140252** |
| Small bucket (147) | 0.889898156043278 | 0.8898840061192003 |
| Medium bucket (108) | 0.8687299985236197 | 0.868611992104337 |
| Large bucket (45) | 0.778362094336599 | 0.7783554718674102 |
| Worst single direct call | 0.7007 s | 0.7024 s |

**6 better / 0 worse / 294 unchanged**, **0.504163 relative basis points**. The same six case changes in 0065 survive the new base. Both alternating corpus halves improve (deltas -0.000074994907 / -0.000004936957); excluding the five largest relative gains leaves -0.000000409721. The improvement is small and concentrated; these checks do not prove generalization. Separate-run timing does not prove a speedup or hidden runtime.

All **50 active tests and 300 trusted cases pass**. The full trusted run enforces purity/license checks, bijection, two-run determinism and the two-second watchdog. All 300 exact trusted AMD and candidate counts equal the sandboxed direct probe. Rounded trusted score/fill: **0.838891 / 0.942605**. Independent parsing checks exact case identity, bucket aggregation, source hashes and all recorded outcomes. The local run does not apply the hidden 4 GiB cap.

Tested `mod.rs` SHA256: `74c127a77c376c6bce7d0a0c7c687c3c16fde46104a1720c2d260f9c0a838a26`. Inherited `rgreedy.rs` SHA256: `f01bafece07f5873445df34c86f2ef6c22e328789bd832a18b289358538e03df`. Source hashes remain identical across tests, direct probe and trusted execution. Corpus SHA256: `faa3ecc29c4ef2c54fe08e4382cee0fef39b04b2cc0efa521d8ecc9c66b7c5b6`.

This establishes a local improvement over the latest promoted source used for this comparison. It does not establish the one-basis-point threshold on the different hidden corpus or a new official promotion. The old prefix/eraser/diversity/allocator/portfolio/weighted-twin findings remain scoped to their frozen research base; no automatic resweep or combination was performed. See [residual-core refinement](../techniques/residual-core-refinement.md).
