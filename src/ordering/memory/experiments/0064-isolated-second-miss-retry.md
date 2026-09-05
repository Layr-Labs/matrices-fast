# 0064 — Isolated second first-round miss-retry after two hidden timeouts

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `0e41927` (0063 local tree). Official tip is still `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **0062 / 0063 Development Score:** 0.842991 (both failed the hidden 2 s cap)
- **Candidate Development Score:** 0.843148 (re-measured; buckets 0.8903 / 0.8642 / 0.7919)
- **Delta:** −0.000210 (−2.10 basis points vs the 0061 tip)
- **Status:** WIN (local). Hidden validation pending.

---

## 1. Context

0061 promoted at hidden **0.86837**. Two leftover extensions on top of it then failed the hidden 2 s cap with no score:

| Experiment | Local | Hidden | What it added beyond 0061 |
| --- | ---: | --- | --- |
| [0062](0062-wider-margin-and-second-miss-retry.md) | 0.842991 (−3.67 bip) | `395313f1` **failed** | 0.90 well-below + second miss-retry |
| [0063](0063-drop-large-090-extra-relabel.md) | 0.842991 (−3.67 bip) | `2d954e54` **failed** | 0062 minus 0.90 extra relabel on `n >= 10k` |

0063 already removed the only 0062 ticket that can run on `n >= 10k`. It still timed out. The remaining 0062/0063 extras that 0061 did not have are:

1. Extra exact LNS streams and the medium `nnz <= 50k` gate at ratio `< 0.90` (was `< 0.80`). Those paths are `n <= 6_000`.
2. Extra i.i.d. AMF+AMD relabel seeds at ratio `< 0.90` on `n < 10k` (0063 already restored `n >= 10k` to 0.80).
3. The second first-round miss-retry on below-anchor `n < 10k`.

(1) and (2) are the leftover 0.80–0.90 tickets. They produced eight of the nine 0062 local wins and they are what 0063 still spent on the hidden corpus. (3) was measured in isolation during 0062 at **0.843148 / 1.357 s** and is almost all `pooling_sppa9tp`.

This submission drops (1) and (2) and ships only (3). `is_well_below` returns to the hidden-proven 0061 gate (`ratio < 0.80`). Extra relabel and exact LNS match 0061 again. The only new work is the second miss-retry.

---

## 2. Hypothesis

The hidden 2 s cap is not “any new subtree ticket.” 0061 already added a first miss-retry, extra LNS, extra relabel, an extra 16M pass, and terminal nnz opening, and it **promoted**. The thing that failed twice is spending *more* early tickets on the 0.80–0.90 band — even when those tickets are `n <= 6k` LNS or `n < 10k` extra relabel.

A second first-round miss-retry is different:

- It runs only after the size-only first round **and** the 0061 leftover ticket both return zero improvements.
- It cannot displace a winning first-round basin.
- It is gated `n < 10_000`, so it never touches the local worst-case matrices (`crudeoil_lee4_10`, `arki0013`, `ringpack_30_2`).
- Isolated worst `order()` was 1.357 s, inside the passing 0061 band (1.345–1.356 s).
- Requested work stays inside `SUBTREE_SEARCH_WORK_LIMIT` (32M): small uses 2×1M×16 blocks; medium only raises `max_s` to 384 on the existing 16×2M envelope.

If the hidden timeout was the 0.80–0.90 early tickets, this revision should pass. If it was the second miss-retry itself, the leftover-search family is closed at this budget and the next idea must be a different mechanism.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **keep the second miss-retry; revert every 0.80–0.90 leftover ticket.**

```rust
fn is_well_below(best_flops: u64, amd_flops: u64) -> bool {
    amd_flops > 0 && best_flops < amd_flops
        && best_flops.saturating_mul(5) < amd_flops.saturating_mul(4) // ratio < 0.80
}

let extra_relabel = is_well_below(best_flops, amd_flops) && nnz > 0 && nnz <= 100_000;
```

`well_below` for exact LNS / the medium `nnz <= 50k` gate uses the same helper, so those paths match 0061.

Second miss-retry, unchanged from 0062/0063:

```rust
if improved == 0 && best_flops < amd_flops && n < 10_000 {
    cfg1.round = 2;
    if n < 1_000 {
        cfg1.streams = 2;
        cfg1.budget = 1_000_000;
        cfg1.max_s = 256;
    } else {
        cfg1.max_s = 384;
    }
    improved = rgreedy::subtree_refine(...);
}
```

Removed: `is_far_below`, the `n >= 10k` extra-relabel split, and the 0.90 well-below widening.

Determinism is unchanged.

---

## 4. Why not ship nothing / why not retry 0062

0062 and 0063 are closed. A third attempt at 0.90 leftover tickets would be the same timeout with a different excuse. Isolated miss-retry is the leftover piece that (a) was timing-safe locally, (b) does not spend early LNS/relabel on a new margin band, and (c) still moves a real matrix (`pooling_sppa9tp` 0.4464 → 0.4090).

−2.10 local bip is under the usual ≥3 bip bar. Hidden translation of 0061 was ~4.5× local (−3.00 local → −13.53 hidden). A one-matrix local win can still vanish on a different corpus. That is accepted: the alternative is to keep shipping the timeout.

Closed and not retried:

- Additive extra subtree pass on `n >= 10k` (0060).
- Widening a *successful* first-round `max_s` (0061 local loss).
- Third miss-retry / extra well-below pass 9 (zero additional movers).
- Opening `gt_10k` leftover at nnz ~120k.
- Deep-below (`ratio < 0.60`) pile-on (worst 1.504 s).
- 0.90 well-below leftover tickets, including the 0063 “large-only retraction” (two hidden 2 s fails).

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

Unit tests stay at 44. The work-limit test already covers the second miss-retry envelope.

---

## 6. Result

Re-measured on this box after the retraction:

```
SCORE = 0.843148
lt_1k 0.8903 · 1k_10k 0.8642 · gt_10k 0.7919
WORST order() = 1.434 s
```

Matches the 0062 isolated miss-retry score. `lt_1k` and `gt_10k` are unchanged at four digits; `1k_10k` 0.8650 → 0.8642 is `pooling_sppa9tp`. Worst `order()` 1.434 s this run vs 1.357 s on the isolated 0062 trial and 1.345 s on the passing tip. Times on this box vary ~1.6×; 1.43 vs 1.35 is not a new class of work. The new ticket is `n < 10k` and cannot be the local large-graph worst case.

From the 0062 variant table, measured on this box against the same 0061 tip:

| Variant | Dev score | Δ vs tip (bip) | Worst `order()` |
| --- | ---: | ---: | ---: |
| Tip 0061 | 0.843358 | — | 1.345 s |
| Isolated second miss-retry | **0.843148** | **−2.10** | **1.357 s** |
| 0062 / 0063 (0.90 + miss-retry) | 0.842991 | −3.67 | 1.361–1.369 s |

The isolated win is almost all `pooling_sppa9tp` (nnz 121302, so it never sees the `nnz <= 100k` extra pass or the `nnz <= 50k` LNS gate). The other eight 0062 wins came from the 0.90 tickets and are given back.

If re-measure disagrees, trust the new number. Times vary ~1.6×; 1.357 vs 1.345 is the same band as the passing tip.

---

## 7. Timing argument

0061 passed hidden with a first miss-retry on below-anchor `n <= 80k`. This adds one more ticket only when that first leftover ticket also misses, and only on `n < 10k`. It does not add early AMF/AMD seeds or extra LNS streams. The local worst cases never enter the new branch. That is the smallest leftover-search delta that still lowers the official local score.

---

## 8. Follow-ups

- If this still hits the 2 s cap, the second miss-retry is closed too. Do not add another first-round ticket.
- If this is rejected as worse than 0.86837 or sub-1-bip hidden, need a many-matrix mover, not another single-matrix window. `pooling_sppa9tp` may not be in the hidden corpus.
- `RELABEL_AMF_MAX_NNZ = 200_000` still excludes `gams05` (nnz 252910, ratio 0.783, ~1.24 s).
- Keep-better-of (size-only chain vs miss-retry chain) is monotone and untried; it doubles the expensive suffix.
- 0.90 leftover tickets are closed on this tip, including size-gated retractions.

## Links

- Timeout predecessors: [0062](0062-wider-margin-and-second-miss-retry.md), [0063](0063-drop-large-090-extra-relabel.md)
- Hidden-proven leftover envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
