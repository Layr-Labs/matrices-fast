# 0062 — Wider leftover margin (0.90) and second first-round miss-retry

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `9f37872` (this repo; Yukon-promoted tip `c7d5fe7` / source `784bfe5`, official hidden **0.86837**). Local tree is 0061 on Layr-Labs/matrices-fast `ea67ff8`.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (re-measured; buckets 0.8903 / 0.8650 / 0.7919)
- **Candidate Development Score:** 0.842991 (buckets 0.8901 / 0.8640 / 0.7919)
- **Delta:** −0.000367 (−3.67 basis points vs the re-measured tip on the full 300-matrix dev corpus)
- **Status:** WIN (local). Hidden validation pending.

---

## 1. Context

0061 promoted at hidden **0.86837** (−13.53 bip vs 0.869723) from a 3.00 bip local gain. The hidden corpus converted leftover tickets much harder than the 300-matrix dev set. The leftover-search family is not exhausted; 0061 only treated ratio `< 0.80` as “well-below” and only retried the size-only first subtree round once.

This session re-measured the unmodified 0061 tip on this box before any edit:

```
SCORE = 0.843358
lt_1k 0.8903 · 1k_10k 0.8650 · gt_10k 0.7919
WORST order() = 1.345 s
```

The binding constraint is still the 2 s hidden cap. Additive extra passes on `n >= 10k` timed out in 0060. Widening a *successful* first-round `max_s` lost in 0061 (0.843829). Those two negatives stay closed.

---

## 2. Hypothesis

0056 and 0061 said conversion tracks the AMD-anchor margin. 0061 cut leftover LNS/relabel at ratio 0.80. The 0.80–0.90 band still converted in 0056 (`mpbp_07` 0.8983 gained 0.0015; `sfacloc2_3_80` 0.9669 gained 0.0009) and is where several 0061 near-wins sat.

Separately, 0061’s first miss-retry uses `max_s = 256` on medium graphs. If that ticket also finds nothing, a wider window (`max_s = 384`) is a different block set on the same unchanged incumbent. It cannot displace a winning first-round basin.

Together: **spend 0061’s leftover tickets on a wider margin, and give first-round misses one more window.**

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: extend leftover search one margin step and one miss-retry step.

1. **`is_well_below` is now ratio `< 0.90`** (`best_flops * 10 < amd_flops * 9`), was `< 0.80` (`* 5 < * 4`). The extra relabel seeds (16 / 12) and the extra exact-LNS streams / `nnz <= 50_000` medium gate now fire on the 0.80–0.90 band as well as below 0.80.
2. **Second first-round miss-retry** on below-anchor `n < 10_000` when the size-only pass *and* the first leftover ticket both return zero improvements. Medium `max_s = 384`; small uses 2 streams × 1M and `max_s = 256`. Requested work stays inside `SUBTREE_SEARCH_WORK_LIMIT`.

Determinism is unchanged: the new helper is a pure function of `(best_flops, amd_flops)`.

---

## 4. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
```

Unit tests: **44 passed**, 18 ignored diagnostics.

---

## 5. Variant table

Scores are end-to-end `probe_timing_and_score` on this box against the same 0061 tip (0.843358 / 1.345 s).

| Variant | Dev score | Δ vs tip (bip) | Worst `order()` | Keep? |
| --- | ---: | ---: | ---: | --- |
| Tip 0061 (unmodified) | 0.843358 | — | 1.345 s | baseline |
| Deep-below tier (`ratio < 0.60`): extra LNS/relabel + extra passes 9/10 + nnz 80k | 0.843141 | −2.17 | **1.504 s** | no — time regresses; 2 movers only (`pooling_sppa9tp`, `maxcsp-langford-3-11`) |
| Isolated second miss-retry (`max_s=384` on `n<10k` below-anchor) | 0.843148 | −2.10 | 1.357 s | yes — almost all of the 2.17, timing-safe |
| + extra well-below pass round 9 / nnz 130k / +4 relabel seeds | 0.843148 | −2.10 | 1.359 s | no — zero additional movers |
| + third miss-retry (`max_s=512`) and large `nnz<=100k` retry | 0.843148 | −2.10 | 1.402 s | no — zero additional movers, slower |
| **Shipped: second miss-retry + well-below widened to 0.90** | **0.842991** | **−3.67** | **1.369 s** | **yes** |

The deep-below LNS pile-on is the useful negative: more early tickets on `ratio < 0.60` did not find a second matrix and pushed `crudeoil_lee4_10` to 1.50 s. The second miss-retry is the part that moves `pooling_sppa9tp` (nnz 121k, so it never sees the `nnz<=100k` extra pass or the `nnz<=50k` LNS gate). Widening the margin to 0.90 is the part that adds the other eight wins.

---

## 6. Result

| | Tip 0061 | Candidate | Δ |
| --- | ---: | ---: | ---: |
| Aggregate | 0.843358 | **0.842991** | −3.67 bip |
| `lt_1k` (w 0.30, 147) | 0.8903 | **0.8901** | −2 bip in-bucket |
| `1k_10k` (w 0.30, 108) | 0.8650 | **0.8640** | −10 bip in-bucket |
| `gt_10k` (w 0.40, 45) | 0.7919 | 0.7919 | 0 at 4 digits |
| Worst `order()` | 1.345 s | **1.369 s** | +0.024 s (same 1.4 s band as the passing tip) |
| Movers | — | **9 better / 1 worse / 289 same** | net win |

Largest wins: `pooling_sppa9tp` 0.4464→0.4090, `pooling_adhya4pq` 0.7116→0.6843, `blend721` 0.8624→0.8530, `mpbp_15` 0.8105→0.8024, `netmod_kar1` 0.7894→0.7813, `rsyn0810m04m` 0.8237→0.8199.

Single loss: `multiplants_stg1a` 0.8549→0.8558 (+0.00095). Early extra tickets on the new 0.80–0.90 band can change the subtree incumbent; the AMD floor still caps the ratio.

`gt_10k` is unchanged at four digits. The local worst cases (`crudeoil_lee4_10`, `arki0013`) stay on the 0061 envelope: second miss-retry is `n < 10_000`, extra relabel is `nnz <= 100_000`.

---

## 7. Why it won

0061 already proved leftover tickets convert below the AMD anchor. It left two holes: the 0.80–0.90 band never got the extra LNS/relabel schedule, and a failed `max_s=256` leftover first-round killed the chain. Filling those holes moves nine matrices, eight of them in `1k_10k` / `lt_1k` where the tickets are cheap.

The 2.10 bip isolated miss-retry is almost one matrix (`pooling_sppa9tp`). The 0.90 margin is what turns a one-matrix luck risk into a nine-matrix structural step. That is why this ships as the pair, not as the isolated retry.

Timing stays in the passing 0061 band (1.369 s vs 1.345 s). Times on this box vary ~1.6×; the comparative rule is “do not exceed a known-passing worst case by a meaningful factor.” 1.37 vs 1.35 is not that.

---

## 8. Follow-ups

- A third miss-retry (`max_s=512`) and a large `nnz<=100k` retry added no movers. That window is closed at this budget.
- Extra terminal subtree rounds 9/10 on well-below `n<10k` added no movers. Do not pile more 4×4M passes.
- `gt_10k` leftover above `nnz=100k` still includes the two local worst cases at nearly the same nnz (`crudeoil_lee4_10` 120632, `ringpack_30_2` 121458). A size gate cannot take one without the other.
- Hidden translation of 0061 was ~4.5× local. Treat a 3.67 bip local step as a real hidden candidate, not as 3.67 hidden bip.

## Links

- Predecessor: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Lottery: [0004-structured-relabelings.md](0004-structured-relabelings.md)
