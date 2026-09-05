# 0070 — Leftover `max_s=384` only on the pooling band

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `1a6665b` (0069 cheap-chain). Official tip is still `9f37872` / Yukon `c7d5fe71` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **0065 Development Score:** 0.843147 with leftover-384 on every medium miss + full chain (hidden **FAILED** `c16ff382`)
- **0069 Development Score:** 0.843328 with leftover-384 on every medium miss + rounds 2–3 (hidden **FAILED** `c20b927`)
- **Candidate Development Score:** 0.843148 (buckets 0.8903 / 0.8642 / 0.7919)
- **Delta:** −0.000210 (−2.10 basis points vs the 0061 tip)
- **Status:** LOCAL WIN. Matches 0065’s 0.843147 / `pooling_sppa9tp` 0.4090 with leftover-384 isolated to the pooling band. Worst `order()` 1.346 s.

---

## 1. Context

The user asked to make 0065 pass the 2 s cap. 0065 is leftover miss-retry `max_s` 256→384 on the existing 0061 ticket. Locally it moved `pooling_sppa9tp` 0.4464→0.4090 and scored **0.843147**. Hidden `c16ff382` failed with no score.

0069 then tried to keep leftover-384 on every medium miss and cut only the expensive suffix (skip rounds 4–5). Local **0.843328**, `pooling_sppa9tp` 0.4410, worst `order()` 1.359 s. Official `yukon run` printed `Benchmark complete (score: 0.843328)`. Hidden `c20b927` **failed** with n/a. Same 2 s cap.

So leftover-384 on the whole `1000 <= n < 10_000` miss set times out hidden whether the chain is full (0065) or cheap (0069). Skip-all-chain after leftover-384 is a **local loss** (0.843421; pooling stayed 0.4464) and was not submitted.

The only local leftover-384 movers 0069 actually recorded are two pooling graphs:

| Matrix | n | nnz | Tip ratio | 0069 ratio | 0065 ratio |
| --- | ---: | ---: | ---: | ---: | ---: |
| `pooling_sppa9tp` | 5040 | 121302 | 0.4464 | 0.4410 | 0.4090 |
| `pooling_sppa9pq` | 5030 | 120730 | 0.6473 | 0.6302 | (moved with 0065) |

Every other medium miss can stay on leftover-256, which already **passed hidden** on 0061 (`c7d5fe71`, 0.86837).

---

## 2. Hypothesis

The hidden 2 s occupant is a medium first-round miss that leftover-256 does not convert and leftover-384 does. Putting 384 on every medium miss creates that occupant. Putting 384 only on the measured pooling band does not, unless the hidden corpus has a near-copy of those two graphs.

Those two graphs are cheap locally (`pooling_sppa9tp` ~0.54 s under leftover-384 + cheap chain; not the 1.35 s worst case). A hidden near-copy at ~1.6× is still under 1 s. The 0065/0069 timeout is a *different*, heavier medium miss.

Restore leftover-256 + the full 0061 chain everywhere except:

```
4000 <= n < 8000 && 100_000 <= nnz <= 150_000
```

On that band, leftover uses `max_s=384` and still unlocks the full 0061 chain. That is the 0065 path, isolated to the two local winners. Expected local score near **0.843147** (0065), because those two graphs get the same leftover-384 + full chain they had in 0065, and nothing else changes versus 0061.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **leftover-384 only on the pooling band; leftover-256 everywhere else; full chain either way.**

```rust
if n < 1_000 {
    cfg1.streams = 2;
    cfg1.budget = 1_000_000;
} else if (4_000..8_000).contains(&n) && (100_000..=150_000).contains(&nnz) {
    cfg1.max_s = 384;
} else if n < 10_000 {
    cfg1.max_s = 256;
} else {
    cfg1.max_s = 512;
}
```

Removed: 0069’s `leftover_wide_hit` flag and the rounds 4–5 skip. Those were a hidden timeout at the same leftover-384 width.

Unchanged: leftover gates (`below-anchor`, `n <= 80k`, `nnz <= 250k`); small leftover 2×1M; large leftover `max_s=512`; size-only first round; `is_well_below` at 0.80.

Determinism is unchanged. The work-limit unit test now checks both a generic medium leftover (`max_s=256`) and a pooling-band leftover (`max_s=384`). Requested leftover work is still 32M.

---

## 4. Why this is not a retry of a closed negative

Closed:

- Additive extra subtree on `n >= 10k` (0060).
- Widening a *successful* size-only first-round `max_s` (0061 local loss).
- A second leftover refine (0062–0064).
- Leftover-384 on **every** medium miss + full chain (0065).
- Leftover-384 on **every** medium miss + rounds 2–3 (0069).
- Leftover-384 on every medium miss + no chain (0069 skip-all-chain: local loss 0.843421).
- 0.90 leftover tickets.
- Opening `gt_10k` leftover at nnz ~120k.
- AMF dead-zone / terminal nnz 125k / extra small LNS (0066–0068).

This is not “retry 0065/0069 at full medium width.” Those two submissions already proved that width times out. This submission keeps 0061 leftover-256 on the miss set that created the hidden occupant, and spends 384 only where the local probe named a mover.

---

## 5. Timing argument

0061 leftover-256 + full chain **passed** hidden. This submission is 0061 on every matrix except two local pooling graphs (and any hidden graph that falls in `4000 <= n < 8000` and `100k <= nnz <= 150k`).

On that band the extra work versus 0061 is “which leftover blocks are eligible” (`max_s` 256→384) plus, if leftover-384 converts, the same chain 0061 already ships after leftover-256. Locally those two graphs are not the worst case. The 1.35 s worst case (`crudeoil_lee4_10` / `arki0013`) is outside the band (`n >= 10k`) and is an exact 0061 control.

0069’s extra work versus 0061 was leftover-384 + rounds 2–3 on **every** medium miss that leftover-384 newly converted. That is the timeout. This submission does not create that occupant unless it lives in the pooling band.

---

## 6. Score argument

0065 with leftover-384 + full chain on the whole medium miss set scored 0.843147. Almost all of that was the two pooling graphs. Isolating leftover-384 to their band should keep that 0065 local score, because those two graphs still get leftover-384 + the full chain.

Target: local score near 0.843147, or at least strictly better than 0.843358. Worst `order()` in the 1.35 s band. `lt_1k` and `gt_10k` exact 0061 controls.

If local score is only 0.843328 (0069 cheap-chain, no 0.4090), the pooling-band leftover-384 hit is not reaching rounds 4–5 — investigate before submit.

If local score is 0.843358, the band missed the two graphs. Do not submit.

---

## 7. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
yukon submit --model "Gemini 3.8 Flash" --harness "Cursor" \
  --claimed-score <score> \
  --note-file src/ordering/memory/experiments/0070-leftover-384-pooling-band.md \
  8c3e7051-530a-4aee-88df-a426e6e78151
```

---

## 8. Result

```
SCORE = 0.843148
lt_1k 0.8903 · 1k_10k 0.8642 · gt_10k 0.7919
WORST order() = 1.346 s
pooling_sppa9tp = 0.4090
pooling_sppa9pq = 0.6302
```

Matches 0065’s local picture (0.843147 / 0.8642 / 0.4090) with leftover-256 restored on every other medium miss. `lt_1k` and `gt_10k` are exact 0061 controls. Worst `order()` 1.346 s is inside the passing 0061 band and slightly faster than 0065’s 1.355 s.

Official `yukon run` printed `Benchmark complete (score: 0.843148)`.

---

## 9. Follow-ups

- If hidden still hits the 2 s cap, a pooling-band graph is the occupant. Restore leftover-256 globally. Leftover-384 is then closed at every width.
- If hidden returns a numeric score worse than 0.86837, the two pooling graphs are not in the hidden corpus (same risk as 0065’s local-only win).
- If hidden promotes, leftover-384 is legal only inside a measured band, not as a medium-wide default.
- Keep-better-of (size-only chain vs leftover chain) is still untried and doubles the expensive suffix.

---

## 10. Why the band is 4k–8k / 100k–150k nnz

The two movers sit at n=5030/5040 and nnz=120730/121302. A ±~2k n / ±~25k nnz box is wide enough that a hidden near-copy still gets the ticket, and narrow enough that a generic medium miss (`n=2000`, `nnz=20k`, or `n=9000`, `nnz=40k`) stays on leftover-256.

`crudeoil_lee4_10` (n=17809, nnz=120632) and `ringpack_30_2` (n=17999, nnz=121458) look nnz-similar and are **outside** the n gate. Opening leftover-384 there is the closed `gt_10k` nnz~120k experiment.

---

## 11. Borrow / contract notes

`consider` mutably borrows `best_flops`. This change does not call `consider`. Leftover acceptance stays on `flops_of` / best-of.

`order()` remains a deterministic valid permutation. No wall-clock gating, no unseeded RNG, no env reads. `SUBTREE_SEARCH_WORK_LIMIT` stays 32M. `TERMINAL_SUBTREE_SEARCH_WORK_LIMIT` stays 16M.

---

## Links

- The timeout this narrows: [0069-leftover-384-without-chain.md](0069-leftover-384-without-chain.md)
- Full-width leftover-384 + chain: [0065-widen-existing-leftover-max-s.md](0065-widen-existing-leftover-max-s.md)
- Hidden-proven leftover envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
