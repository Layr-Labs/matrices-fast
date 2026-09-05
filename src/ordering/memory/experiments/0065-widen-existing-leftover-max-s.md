# 0065 — Widen the existing leftover miss-retry window (no extra pass)

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `550cdcf` (0064 local tree). Official tip is still `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **0064 Development Score:** 0.843148 (failed hidden 2 s cap)
- **Candidate Development Score:** 0.843147 (re-measured; buckets 0.8903 / 0.8642 / 0.7919)
- **Delta:** −0.000211 (−2.11 basis points vs the 0061 tip)
- **Status:** LOCAL WIN, **HIDDEN FAIL** (submission `c16ff382`, 2026-09-05). No hidden score — the 2 s cap. Widening leftover `max_s` *and then unlocking the chain* is closed. The window itself is revived without that chain in [0069](0069-leftover-384-without-chain.md).

---

## 1. Context

Three leftover-search extensions on top of the promoted 0061 tip all failed the hidden 2 s cap with no score:

| Experiment | Local | Hidden | Extra `subtree_refine` calls? |
| --- | ---: | --- | --- |
| [0062](0062-wider-margin-and-second-miss-retry.md) | 0.842991 | `395313f1` **failed** | yes (second miss-retry) + 0.90 tickets |
| [0063](0063-drop-large-090-extra-relabel.md) | 0.842991 | `2d954e54` **failed** | yes + remaining 0.90 tickets |
| [0064](0064-isolated-second-miss-retry.md) | 0.843148 | `71f76302` **failed** | yes (second miss-retry only) |

0064 isolated the second miss-retry. That ticket is `n < 10k`, stays inside the 32M work envelope, and was 1.357–1.434 s locally. It still died hidden. So the failure mode is **one more `subtree_refine` on first-round misses**, not the 0.90 margin and not `n >= 10k`.

0064's local win was almost all `pooling_sppa9tp` (0.4464 → 0.4090). The second leftover ticket found it with medium `max_s = 384`. The first leftover ticket — the one 0061 already ships and that **passed hidden** — uses medium `max_s = 256`.

This submission does not add a pass. It puts `max_s = 384` on the existing 0061 leftover ticket and deletes the second refine.

---

## 2. Hypothesis

0061 said widening `max_s` on a *successful* size-only first round loses (0.843829): a larger window displaces the winning basin. The leftover ticket only runs when that size-only round returned **zero** improvements. There is no winning basin to displace.

0064 proved `max_s = 384` on that miss path converts `pooling_sppa9tp`. It also proved a *second* refine on that path is unshippable. The remaining move is to run the wider window as the **first** leftover ticket.

Requested work does not change: medium leftover is still `2_000_000 × 16 × 1 = 32M`. `max_s` only changes which blocks are eligible, not the budget × blocks × streams product. One `subtree_refine` call, same as 0061.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **widen the existing leftover miss-retry; do not add a refine.**

```rust
cfg1.round = 1;
if n < 1_000 {
    cfg1.streams = 2;
    cfg1.budget = 1_000_000;
} else if n < 10_000 {
    cfg1.max_s = 384; // was 256 in 0061
} else {
    cfg1.max_s = 512;
}
```

Removed: the 0062/0064 second leftover refine (`round = 2` on `n < 10k`).

Unchanged: `is_well_below` at ratio `< 0.80`; extra relabel / extra LNS the 0061 schedule; large leftover `max_s = 512`; size-only first round.

Determinism is unchanged. The work-limit unit test now checks `max_s = 384` on the medium leftover config.

---

## 4. Why this is not a retry of a closed negative

Closed:

- Additive extra subtree pass on `n >= 10k` (0060).
- Widening a *successful* first-round `max_s` (0061 local loss).
- A second leftover refine (0062 / 0064 hidden timeout).
- Third miss-retry / extra well-below pass 9.
- Opening `gt_10k` leftover at nnz ~120k.
- Deep-below pile-on.
- Any 0.90 leftover tickets, including size-gated retractions.

This is a parameter change on a pass that already ran on the hidden-promoted 0061 tip. It does not add a refine, does not widen the successful first round, and does not spend 0.90 tickets.

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

---

## 6. Result

Re-measured on this box after deleting the second refine:

```
SCORE = 0.843147
lt_1k 0.8903 · 1k_10k 0.8642 · gt_10k 0.7919
WORST order() = 1.355 s
```

Same `1k_10k` drop as 0064 (0.8650 → 0.8642): the first leftover ticket at `max_s = 384` found the `pooling_sppa9tp` win. Worst `order()` 1.355 s matches the passing 0061 tip (1.345–1.356 s), not the 0064 extra-pass 1.43 s. No basin displacement on the miss path.

−2.11 is under the usual ≥3 bip bar. The alternative after three hidden timeouts is to keep adding passes. Hidden translation of 0061 was large; a one-matrix local win can still vanish. That is accepted.

---

## 7. Timing argument

0061 leftover miss-retry passed hidden. This keeps the same number of leftover `subtree_refine` calls, the same 32M envelope, and the same gates (`below-anchor`, `n <= 80k`, `nnz <= 250k`). The only delta is which blocks the medium leftover ticket may take (`max_s` 256 → 384). A wider window can do more *useful* work inside the same budget; it cannot request more budget.

0064 added a refine and died. 0065 does not add a refine.

---

## 8. Follow-ups

- If this hits the 2 s cap, leftover `max_s` on the miss path is closed too. Do not add another first-round ticket.
- If this is rejected as worse than 0.86837 or sub-1-bip hidden, need a many-matrix mover. `pooling_sppa9tp` may not be in the hidden corpus.
- `RELABEL_AMF_MAX_NNZ = 200_000` still excludes `gams05` (nnz 252910, ratio 0.783).
- Keep-better-of (size-only chain vs leftover chain) is monotone and untried; it doubles the expensive suffix.
- Stealing relabel restarts from AMD ties and spending them on `n < 1k` well-below is untried and does not add a subtree pass.

## Links

- Hidden-proven leftover envelope: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Extra-pass timeouts: [0062](0062-wider-margin-and-second-miss-retry.md), [0063](0063-drop-large-090-extra-relabel.md), [0064](0064-isolated-second-miss-retry.md)
