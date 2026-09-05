# Below-anchor terminal escalation (sparse, 8M×8×3)

**Base:** `c3fae018` (promoted tip **0.869723**, submission `bd451297`), the
below-anchor re-tiering candidate from experiment 0060-conditional-search.

**Local dev:** 0.843658 → **0.843339** (**−3.19 bip**), fill 0.943644.
**37 better / 0 worse / 263 unchanged.** Buckets: lt_1k 0.8903→0.8901,
1k_10k 0.8659→0.8651, gt_10k 0.7919→0.7919 (unchanged).

## Hypothesis

The promoted frontier re-tiers search budget by anchor margin inside the
conditional terminal chain. When that chain stalls on seed, below-anchor
matrices with `nnz <= 50_000` can still gain from a few unconditional
re-postordered subtree passes — but only if the stage is tiny (two prior
submissions at 16M–32M×32×4 failed the hidden cap) and runs **last** in
`order()` so pair descent cannot undo a gain.

## The diff

One block at the end of `order()`, after all cleanup:

- Gate: `best_flops < amd_flops` AND `nnz <= 50_000` AND `n` in subtree range.
- Three rounds at `(max_s, max_blocks, budget)` = (384,8,8M), (768,8,8M), (384,8,8M).
- `round = 10..12`, routed through `f < best_flops` (monotone).

No ledger, no margin scaling — the promoted tip already owns margin re-tiering.

## Variants that lost

| variant | Δ bip | worse | note |
|---|---|---|---|
| 2 rounds | −2.83 | 0 | under 3 bip bar |
| escalation before pair descent | −3.02 | **3** | pair descent undid gains |
| 0060/0061 on ea67ff8 (32M×32×4 + ledger) | −7.5 | 0 | **hidden cap failure** ×2 |

## Timing

| revision | worst `order()` |
|---|---|
| base `c3fae01`, this session | 1.355 s |
| candidate | 1.426 s |

Worst matrix is still a ledger-excluded heavy case (`crudeoil_lee4_10` family);
the escalation adds at most ~0.07 s on eligible matrices (8M×8 saturates at
~0.038 s per pass on dev).

## Validation

`bash scripts/local-candidate-build.sh && cargo run --release` ×2: status **OK**,
score **0.843339**, 300 matrices, no worker failures. 44 unit tests pass.
