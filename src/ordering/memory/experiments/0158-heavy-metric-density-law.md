# 0158 — Re-derive the heavy-metric dead band as the density law it already used

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792573** (no change; 0 of 300 rows change their
  permutation or their exact flops)
- **Status:** compliance re-derivation with zero dev effect; adopted

## Hypothesis

The heavy-tier quotient-graph pivot-metric block queued `k` variants under a
four-step ladder in `nnz` with a hole cut in it:

| `nnz` | variants |
|---|---:|
| `< 200_000`, sparse only (`nnz <= 6 n`) | 2 |
| `200_000..=500_000` | **0** |
| `500_000..700_000` | 4 |
| `700_000..=1_400_000` | 1 |

The hole's own doc comment says what it is: *"Dead window: between 200k and
500k nnz every dev row was pure cost (zero wins …), so the block is skipped
there entirely."* Both edges were read off one corpus, which makes it a fitted
two-sided window and not a structural gate.

The hypothesis: the rows the hole removes are **dense**, and the block's low
band already carried a density guard (`nnz <= 6 n`, the same guard the no-dense
AMD pass uses). If that is right, the guard applied at every size subsumes the
band, and the fitted constants can go without losing anything.

## What changed

`src/ordering/mod.rs`, two predicates:

```rust
// was: (nnz >= DEAD_MIN && nnz <= DEAD_MAX) || (nnz < DEAD_MIN && nnz > 6 n)
let metric_dead_window = nnz > HEAVY_SPARSE_MAX_AVG_DEG * n;

// was: if nnz >= GIANT_MIN {1} else if nnz < DEAD_MIN {2} else {MAX_VARIANTS}
let band_cap = if nnz >= HEAVY_METRIC_GIANT_MIN_NNZ { 1 } else { 2 };
```

`HEAVY_METRIC_DEAD_MAX_NNZ` is deleted. `band_cap` is now monotone
non-increasing in `nnz`, and pointwise ≤ the ladder it replaces everywhere the
ladder ran at all.

## Result

**Dev score unchanged to the last digit** (0.792573 / 0.924518) and an exact
per-row flop comparison shows **0 of 300 rows moved**. Six rows compute
something different and none of them ends up anywhere different:

| row | n | nnz | variants before | after |
|---|---:|---:|---:|---:|
| `cont6-qq` | 120 395 | 557 994 | 4 | 2 |
| `pooling_sppb5pq` | 18 529 | 674 470 | 4 | 0 (dense) |
| `pooling_sppc3pq` | 23 173 | 893 724 | 1 | 0 (dense) |
| `transswitch2383wpr` | 59 853 | 277 562 | 0 | 2 |
| `transswitch2736spr` | 69 651 | 331 010 | 0 | 2 |
| `unitcommit_200_100_1_mod_8` | 146 830 | 476 332 | 0 | 2 |

The block is a strict best-of, so queuing more or fewer variants only moves a
row if one of them wins; on all six, none does.

## Why it won / lost

Neither on score. It is a compliance re-derivation that happens to be free: the
fitted band was doing the work of a density predicate the same block already
applied one tier lower, and the tier it protected (700k+, one variant) is the
only place the block's output is load-bearing on this corpus.

**A rejected alternative is worth recording.** The other compliant form is the
*monotone envelope*: a step ladder with a zero in the middle has exactly one
monotone non-increasing law pointwise ≤ it — stop at the first zero, i.e. skip
everything at `nnz >= 200_000`. That was built and measured: **+2.107 relative
dev bip**, all of it from a single row, `faclay75` (n = 272 878,
nnz = 1 379 706), which loses 2.79 % of its flops. The envelope deletes the
giant tier as collateral, and the giant tier is where the value is. **The
work-dominating form of a window is not automatically the right one.**

## Caveats

- Not strictly work-dominating: three sparse mid rows gain up to two metric
  passes each. The addition is bounded twice over, by `band_cap = 2` and by
  `HEAVY_METRIC_BUDGET / nnz`.
- `HEAVY_METRIC_MAX_NNZ = 1_400_000` is left in place. It is an upper
  half-space with a genuine cost reading, but it sits 20 294 above `faclay75`,
  the largest row on this corpus that the block reaches — worth re-deriving if
  anyone finds a cost law for it.

## Follow-ups

- The same treatment applies to `HEAVY_RELABEL_AMF_{SPARSE,DENSE}_*`
  (`mod.rs:1164-1167`), two `nnz` bands separated by a 20 000-wide hole whose
  lower edge sits 1 024 below `kissing2`'s `nnz`.

## Links

- [0154](0154-monotone-relabel-restart-law.md) — the same pattern applied to the
  relabel restart ladder
