# 0065 — Returned research and cleanup of the terminal core winner

- **Date:** 2026-09-06
- **Reference source:** `2a28517ddc00dac41a52c270f9601702df982200`
- **Development score:** 0.8390631716578836 → **0.8390208740493201**
- **Status:** selected local improvement; 50 active tests and all 300 trusted cases pass. No new official promotion is established by this record.

## Hypothesis and selected change

The reference ends with a degree-at-most-three elimination prefix followed by an AMD/AMF portfolio on the exact residual core. Its best lifted ordering is compared with the completed whole-graph incumbent, then returned without further cleanup. Existing whole-graph k5/k4 passes run before this terminal candidate. A small residual can therefore expose inexpensive refinement even when its original graph is too large for whole-graph bitsets.

The selected change adds one existing k5→k4 cycle to the selected residual-core order. It applies when `5 <= core_n <= 4096` and directed `core_nnz <= 65536`, with **16,000,000 logical units per kernel**. k4 consumes an accepted k5 improvement. The original terminal candidate is accepted first and remains available throughout. A refined order is admitted only after bijection checks, exact core rescoring, agreement between prefix-plus-core cost and the exact whole-graph score, and strict improvement over the completed incumbent.

Only the caller in `src/ordering/mod.rs` changes for this candidate. The existing kernels, degree-three reducer, dependencies, corpus and trusted harness remain unchanged. Neither the alternative-prefix implementation nor the fill-edge eraser is included.

For a fixed prefix `P`, exact residual `H`, and core order `Q`, the acceptance identity is

`F(G, P ++ lift(Q)) = C(P) + F(H,Q)`,

where `F` is the sum of squared diagonal-inclusive elimination widths. This proves score transfer for a fixed prefix; it does not prove that the greedy prefix is globally optimal. Preserving the completed reference before this additional phase makes successful output pointwise nonworsening in score. Resource failures remain a separate concern.

## Controlled native results

Every comparison uses the same 300 public cases and exact AMD counts: bucket populations 147, 108 and 45. The score is the **weighted arithmetic mean of within-bucket geometric means** of candidate/AMD FLOP ratios, with weights 0.3/0.3/0.4. An independent parser reproduced the exact aggregates and mover counts from the native logs.

| Treatment | Exact development score | Better / worse / same |
|---|---:|---:|
| Frozen reference | 0.8390631716578836 | — |
| Core cleanup: n≤512, nnz≤8192, max degree≤64, 4M per kernel | 0.8390282958122284 | 2 / 0 / 298 |
| Same 4M cleanup, n≤4096 and nnz≤65536, degree cap retained | 0.8390282958122284 | 2 / 0 / 298 |
| Same wider gate, degree cap removed | 0.8390221697788143 | 6 / 0 / 294 |
| **Same wider gate, 16M per kernel** | **0.8390208740493201** | **6 / 0 / 294** |
| Protected fill-edge eraser | 0.8390559712140117 | 1 / 0 / 299 |
| One alternative degree-three prefix | 0.8390631716578836 | 0 / 0 / 300 |

The selected improvement is **0.00004229760856344633**, or **0.504105 relative basis points**: `(before-after)/before * 10000`. It is a small development gain, not an official result. The starting official score is 0.867211; the hidden evaluation is a different measurement and cannot be inferred from this table.

The six gains comprise two small, one medium and three large original graphs. All three bucket geometric means improve: approximately 0.890340770→0.890326613, 0.868720343→0.868602338 and 0.778362094→0.778355472. No structural-family generalization is established by those size counts.

Both alternating halves in original corpus order improve, with score deltas −0.00007500155697715005 and −0.00000493698832715328. Alternating name-sorted halves also improve. Removing the five largest relative case gains leaves a very small negative delta, −0.0000004097212429332586. These checks limit a single-case explanation; they do not make six observed gains broad evidence of generalization.

## Why the controls matter

Widening the residual-size gate while keeping maximum degree 64 changes no case. Removing the degree cap adds four gains and preserves the first two. The bitset kernels already precharge density-dependent operations; the eraser's degree guard was not needed for this bounded cleanup treatment.

The 16M follow-up improves two of those six cases further without adding another improving case. The reason for testing more allowance was explicit: 508 k5-window metadata reservations at 8192 units already cost 4,161,536 units, exceeding the initial 4M allowance for a complete 512-vertex window cycle before graph scans or replay. Increasing the allowance tests additional coverage; it is **not** evidence that a new allocator spends work more efficiently, and does not promise a full cycle for every admitted core.

The selected direct probe's slowest observed call was 0.8544 seconds, versus 0.8878 for the reference. Those separate-run timings are load-sensitive and do not show that adding cleanup accelerates the solver. Subsequently, the supplied sandboxed build and trusted parent passed all 300 cases, including permutation, determinism and 2-second watchdog checks. Its rounded score is 0.839021 and fill metric is 0.942803. The active candidate suite passed 50 tests. Native source hashes stayed unchanged during these runs.

The 32M sum of kernel allowances is not a whole-pipeline work bound. Kernel setup/replay and atomic primitives have their existing charges; caller cloning, exact rescoring, lifting and the preceding portfolio still contribute time and memory. Official 4-GiB and hidden-runtime behavior are not established by a local timing minimum.

## What the six research returns taught us

The returns supplied specifications, independent Python models and witnesses, with some Rust sketches; they did not supply a validated native replacement. Archive integrity and bounded reference checks were verified before implementation. The exact external GPT Pro model variant was not recorded.

1. **Prefix choice:** a degree-three cutoff conflict can help, but lower deficiency and a smaller core are not dominance certificates. Independent optimal-completion witnesses give 119→110 and 187→189. The native implementation preserves the reference and tries one recorded alternative plus default AMD. It passed 63 tests, made 13 admitted helper invocations and accepted no alternative. The telemetry does not establish that all 13 completed replay or AMD; refusal reasons were not separated. Keep this fixed-gate experiment as a negative result rather than tuning its thresholds to those cases.

2. **Core fill-edge erasure:** protect every edge of the exact residual, including fill introduced by the prefix. Remove only edges passing the current chordality-preserving test, rebuild/validate a perfect elimination order, then recheck exact original-objective transfer. The implementation passed 57 tests. Diagnostics report 46 accepted core refinements, of which 45 produced no improvement in final whole-graph FLOPs against the matched reference. They do not classify every intervening lift/validation/selection outcome, so attributing all 45 to incumbent masking would overstate the evidence. The single final gain is smaller than cleanup's gain on that same case. The measured pointwise minimum of eraser and cleanup outputs equals cleanup; no complementary benefit is demonstrated.

3. **Incumbent diversity:** the verified reversal 87/89→57/50 shows that current score alone cannot certify the best refinement path. It does not prove that the proposed edge-orientation distance beats two lowest-score seeds or the actual native portfolio. Shared preparation, observer precharges and an abortable exact scorer remain implementation work; a candidate-stream diagnostic should precede that refactor.

4. **Global work allocation:** delayed gains and cold replay costs are real. The returned reference has an uncharged rebase transition and its Rust tickets omit ledger identity. The adaptive controller and native scoring bound are incomplete. Establish sound accounting and fixed-schedule continuation before adaptive allocation.

5. **Portfolio selection:** the forest certificate `F >= n+3m` is valid. A six-call mask can both omit useful variants and add calls relative to the reference, while its generated examples never exercise the relevant n≥1000 pruning branch. The proposed shadow-mask pilot has not been run; production selection remains unchanged.

6. **Large-core structure:** true-twin score transfer and contiguous expansion have mathematical support, but existing AMD/AMF dependencies already merge supervariables. A native structural census found **zero of 300** public patterns satisfying the proposed weighted engine's initial gates. This is a reach result for those gates, not a disproof of compression or a hidden-corpus claim. Defer the new engine.

## Follow-ups

Preserve the negative native prefix/eraser results and avoid combining implementations merely because their internal objectives improve. A future eraser diagnostic should record prefix cost, refined lifted cost and each final guard outcome to distinguish missed opportunities from earlier-incumbent dominance. Diversity and portfolio work should first measure actual candidate opportunities under their frozen controls. None of these follow-ups has demonstrated an additional score improvement in this round.

## Links

- Technique: [Residual-core refinement](../techniques/residual-core-refinement.md).
- Literature: [Heggernes–Peyton](../literature/heggernes-peyton-minimal-fill.md), [Culbertson–Guralnik–Stiller](../literature/culbertson-guralnik-stiller-edge-erasures.md).
- Reference code: [immutable terminal caller](https://github.com/Layr-Labs/matrices-fast/blob/2a28517ddc00dac41a52c270f9601702df982200/src/ordering/mod.rs#L1975).
