# 0063 — Drop large-graph 0.90 extra relabel after the 0062 hidden timeout

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `6b36a28` (0062 local tree). Official tip is still `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip, re-measured; buckets 0.8903 / 0.8650 / 0.7919)
- **0062 Development Score:** 0.842991 (failed hidden 2 s cap)
- **Candidate Development Score:** 0.842991 (re-measured; buckets 0.8901 / 0.8640 / 0.7919)
- **Delta:** −0.000367 (−3.67 basis points vs the 0061 tip). Same local score as 0062.
- **Status:** WIN (local). Hidden validation pending.

---

## 1. Context

0062 combined two leftover-search extensions on top of the promoted 0061 tip:

1. Widen `is_well_below` from ratio `< 0.80` to `< 0.90`.
2. A second first-round miss-retry on below-anchor `n < 10_000` when the size-only pass and the first leftover ticket both return zero.

Local score 0.843358 → **0.842991** (−3.67 bip). Official `yukon run` printed `Benchmark complete (score: 0.842991)`. Worst local `order()` 1.369 s, inside the passing 0061 band (1.345–1.356 s on this box, ~1.6× noisy).

Hidden submission `395313f1` **failed** with no score. That is the 2 s cap, not a worse hidden geomean. 0060 already taught that additive leftover work on `n >= 10k` is what the hidden corpus kills even when the local worst case looks fine.

This session does not retry 0062. It removes the one 0062 ticket that can run on large graphs.

---

## 2. Hypothesis

0062 added three families of leftover tickets at the new 0.80–0.90 band:

| Ticket | Size gate | Can hit `n >= 10k`? | Local movers in 0062 |
| --- | --- | --- | --- |
| Extra i.i.d. AMF+AMD relabel (16 / 12 seeds) | `nnz <= 100_000` | **yes** (12 seeds) | none of the nine named movers |
| Extra exact LNS streams / medium `nnz <= 50k` | `n <= 6_000` | no | `blend721`, `mpbp_15`, `rsyn0810m04m`, tinies |
| Second first-round miss-retry | `n < 10_000` | no | `pooling_sppa9tp` |

The nine 0062 wins were `pooling_sppa9tp`, `pooling_adhya4pq`, `blend721`, `mpbp_15`, `netmod_kar1`, `rsyn0810m04m`, plus three tinies. All of those have `n < 10_000`. `gt_10k` was unchanged at four digits (0.7919). So the large-graph 0.90 extra-relabel ticket spent wall time and bought **zero** local score.

0061 already spent extra relabel on well-below (`< 0.80`) large graphs with `nnz <= 100k` and **promoted** at hidden 0.86837. That envelope is proven. The new risk is only the 0.80–0.90 slice of those same large graphs: twelve more AMF+AMD seeds on a hidden matrix that is already expensive.

**Drop the 0.90 extra-relabel widening on `n >= 10k`. Keep everything else from 0062.** Large extra relabel stays on the 0061 `ratio < 0.80` gate.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **do not spend 0062's new leftover relabel tickets on large graphs.**

```rust
fn is_well_below(...) -> bool { /* ratio < 0.90 */ }
fn is_far_below(...) -> bool { /* ratio < 0.80 */ }

let extra_relabel = nnz > 0 && nnz <= 100_000 && if n >= 10_000 {
    is_far_below(best_flops, amd_flops)
} else {
    is_well_below(best_flops, amd_flops)
};
```

Unchanged from 0062:

- Second first-round miss-retry on below-anchor `n < 10k` after two empty first-round tickets.
- Exact LNS / medium gate still use `is_well_below` at 0.90. Those paths are `n <= 6_000`.
- Extra 4×4M subtree pass still `n < 10k`.
- Terminal-chain nnz opening still the 0061 below-anchor `gt_10k` envelope (`nnz <= 100k` / `80k`), which excludes `crudeoil_lee4_10` and `arki0013`.

Determinism is unchanged: both helpers are pure functions of `(best_flops, amd_flops)`.

---

## 4. Why this is the timeout fallback, not a new pile-on

The written 0062 fallback was “drop early extra tickets on the new 0.80–0.90 band first; keep the second miss-retry.” Extra relabel is the early ticket that can land on `n >= 10k`. Extra LNS cannot. Isolated miss-retry alone was −2.10 bip and almost all `pooling_sppa9tp` — a one-matrix luck risk on a different hidden corpus.

This submission keeps the miss-retry **and** the cheap 0.90 LNS tickets that produced the other eight local wins, and drops only the ticket that (a) can timeout hidden and (b) did not move the local `gt_10k` bucket. That is still one idea: a size-gated retraction of 0062, not a new search.

Closed and not retried here:

- Additive extra subtree pass on `n >= 10k` (0060 hidden timeout).
- Widening a *successful* first-round `max_s` (0061 local loss 0.843829).
- Third miss-retry / extra well-below pass 9 (zero additional movers).
- Opening `gt_10k` leftover at nnz ~120k (`crudeoil_lee4_10` vs `ringpack_30_2`).
- Deep-below (`ratio < 0.60`) extra LNS/relabel pile-on (worst 1.504 s).

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

Unit tests: **44 passed** on the 0062 tree; the work-limit test is unchanged (no new subtree config).

---

## 6. Result

Re-measured on this box after the retraction:

```
SCORE = 0.842991
lt_1k 0.8901 · 1k_10k 0.8640 · gt_10k 0.7919
WORST order() = 1.361 s
```

Same aggregate and buckets as 0062. Worst `order()` 1.361 s vs 0062's 1.369 s and the passing tip's 1.345 s. No large 0.80–0.90 extra-relabel win was hiding in `gt_10k`. The retraction cost **zero** local score.

---

## 7. Timing argument

0061 extra relabel on large well-below (`< 0.80`, `nnz <= 100k`, 12 seeds) **passed hidden**. 0062 added the same 12 seeds to the 0.80–0.90 band of those large graphs and **failed hidden**. Local worst cases (`crudeoil_lee4_10` nnz 120632, `arki0013` nnz 160172, `ringpack_30_2` nnz 121458) stay outside `nnz <= 100k`, so the local worst-case timer never saw the new large ticket. The hidden corpus can have a large matrix with `nnz <= 100k` and ratio in (0.80, 0.90). That matrix is exactly what this retraction removes.

Second miss-retry stays `n < 10k`. Isolated it was 1.357 s. Extra LNS stays `n <= 6k`. The 0060 lesson is not “never add tickets”; it is “do not add tickets on large graphs that the local worst-case set does not represent.”

---

## 8. Follow-ups

- If this still hits the 2 s cap, drop the remaining 0.80–0.90 tickets (LNS + small/medium extra relabel) and ship isolated second miss-retry only.
- If this is rejected as worse than 0.86837 or sub-1-bip hidden, the leftover family needs a many-matrix mover, not another single-matrix window. Do not retry 0062.
- `RELABEL_AMF_MAX_NNZ = 200_000` still excludes `gams05` (nnz 252910, ratio 0.783, ~1.24 s). Raising it is a timing question, not this submission.
- Keep-better-of (size-only chain vs miss-retry chain) is monotone and untried; it doubles the expensive suffix.

## Links

- Predecessor: [0062-wider-margin-and-second-miss-retry.md](0062-wider-margin-and-second-miss-retry.md)
- Hidden-proven extra relabel envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Large-graph additive timeout: [0060-conditional-search-escalation-below-anchor.md](0060-conditional-search-escalation-below-anchor.md)
