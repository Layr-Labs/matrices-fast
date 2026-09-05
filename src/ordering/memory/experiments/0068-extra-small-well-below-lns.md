# 0068 — Two extra exact-LNS streams on well-below `lt_1k`

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** leftover family at 0061. Official tip `9f37872` / Yukon `c7d5fe7` / source `784bfe5`, hidden **0.86837**.
- **Official Promoted Tip:** 0.86837
- **Base Development Score:** 0.843358 (0061 tip; buckets 0.8903 / 0.8650 / 0.7919)
- **Status:** LOCAL NO-OP. `lt_1k` 0.890338 (tip 0.8903). Streams 7–8 converted nothing. Reverted. Not submitted.

The six 0061 well-below streams already sat on the plateau. Two more 50M seeds did not find a cheaper walk on any of the 147 `lt_1k` matrices at four-digit geomean. The extra-ticket lever on this gate is exhausted at the current incumbent quality.

This note stays in the log so a later session does not retry “two more small LNS seeds” without a new incumbent. The streams were compiled, `probe_lt1k` was run, and then the two seeds were deleted so `order()` matches the promoted 0061 envelope again.

---

## 1. Context

Subtree leftover-search that converts a new matrix and unlocks rounds 2–5 is closed (0065 hidden timeout at the same pass count as the promoted tip). Extra leftover refines are closed (0062–0064). Opening the terminal 16M pass to nnz 125k did not move `pooling_sppa9tp` (0067 no-op). AMF in the 130k–400k hole did not move `gams05` (0066 no-op).

What is still open and does not touch the leftover chain: **more exact-LNS tickets on small well-below graphs.** 0061 already runs six streams on `n <= 1000 && nnz <= 30_000 && ratio < 0.80`, versus four on the rest of that gate. `lt_1k` has the most wall-clock headroom in the corpus (historical max ~0.82 s against a 1.35 s worst case). 0004: this family is a lottery; the lever is more tickets.

---

## 2. Hypothesis

0056: exact-search conversion tracks the AMD-anchor margin. Extra streams on well-below `lt_1k` are cheap and cannot move `crudeoil_lee4_10` / `arki0013`. Two more 50M seeds are the same kind of ticket 0061 already added, not a new subtree pass.

If they convert several small graphs, `lt_1k` 0.8903 drops. Weight 0.30 over 147 matrices: a 10-bip in-bucket move is 3 aggregate bip. A 3-bip in-bucket move is ~1 aggregate bip. Honest expectation is small unless several graphs still have unused plateaus.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory pages).

One idea: **two more well-below small LNS streams.**

```rust
// well_below small_streams, after the six 0061 entries:
(50_000_000, 0xC2B2_AE3D_27D4_EB4F),
(50_000_000, 0x1656_67B1_9E37_79F9),
```

0067 nnz 125k gate reverted (no-op). Leftover first-round stays 0061. 0066 AMF block removed (no-op).

Determinism unchanged: fixed seeds, `is_well_below` already a pure function of `(best_flops, amd_flops)`.

---

## 4. Why this is not a retry of a closed negative

Closed: leftover-chain unlock, extra leftover refine, 0.90 tickets, additive subtree on `n >= 10k`, `gams05` AMF, terminal pass nnz 125k.

This adds exact-search seeds on `n <= 1000` only. No `subtree_refine`. No chain unlock.

---

## 5. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_lt1k
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
yukon run
```

---

## 6. Expected local result

`lt_1k` 0.8903 should drop if any of the 147 graphs still have a cheaper plateau. `1k_10k` and `gt_10k` are exact controls. Worst `order()` should stay on the large graphs (1.35 s class); small LNS cannot be the worst case.

If `lt_1k` is unchanged at four digits, this is a no-op and does not ship.

---

## 7. Timing argument

0061 already spends six LNS streams on this gate and promoted. Two more 50M streams on `n <= 1000` are milliseconds to tens of milliseconds. The local worst case is `n >= 10k`. Hidden risk is a small dense well-below graph that is already expensive; the `nnz <= 30_000` gate is the 0061 envelope.

---

## 8. What 0061 already spends here

The small exact-search gate is `n <= 1000 && nnz <= 30_000`. On well-below incumbents 0061 already runs:

| Stream | Budget | Seed |
| --- | ---: | --- |
| 1 | 100M | `0x9E37_79B9_7F4A_7C15` (byte-identical to the original single stream) |
| 2–4 | 50M | the 0060 below-anchor set |
| 5 | 100M | `0xA076_1D64_78BD_642F` |
| 6 | 50M | `0xE703_7ED1_A0B4_28DB` |

Non-well-below small graphs keep streams 1–4. This submission adds streams 7–8 at 50M only on the well-below arm. The first six entries are unchanged, so any 0061 conversion of those streams is preserved.

The two new seeds are SplitMix-family constants that do not appear elsewhere in `order()`. They are not derived from wall-clock or entropy.

## 9. Follow-ups

- If this times out, do not add more small LNS streams.
- If it is a no-op, the leftover-search and AMF-dead-zone families are exhausted on this tip at this budget.
- Keep-better-of (size-only chain vs leftover chain) is still untried and doubles the suffix.

## Links

- Small LNS origin: [0061-margin-scaled-leftover-search.md](0061-margin-scaled-leftover-search.md)
- Lottery: [0004-structured-relabelings.md](0004-structured-relabelings.md)
- Recent no-ops / timeouts: [0065](0065-widen-existing-leftover-max-s.md), [0066](0066-amf-dead-zone-well-below.md), [0067](0067-terminal-pass-nnz-125k.md)
