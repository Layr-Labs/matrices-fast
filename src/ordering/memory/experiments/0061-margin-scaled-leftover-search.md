# 0061 — Margin-scaled leftover search on well-below-anchor matrices

- **Date:** 2026-09-05
- **Author:** Cursor Agent (Emmanuel Duke)
- **Base commit:** `649ae53` (this repo; Yukon-promoted tip `bd451297` / source `c3fae01`, official hidden **0.869723**). Local tree is the 0060 substitutive re-tiering on Layr-Labs/matrices-fast `ea67ff8`.
- **Official Promoted Tip:** 0.869723 (submission `bd451297-3983-4589-82b0-684e6d0de6b8`)
- **Base Development Score:** 0.843658 (fill from probe; buckets 0.8903 / 0.8659 / 0.7919)
- **Candidate Development Score:** 0.843358 (buckets 0.8903 / 0.8650 / 0.7919)
- **Delta:** −0.000300 (−3.00 basis points vs the re-measured tip on the full 300-matrix dev corpus)
- **Status:** WIN (local). Hidden validation pending.

---

## 1. Context

0060 promoted by reallocating existing subtree / LNS work with a binary `best_flops < amd_flops` test. That was the first half of the recommended next step. The second half was never shipped: **scale leftover search by the margin itself** (more tickets at ratio 0.6 than at 0.97), and spend those tickets on work that the size-only first subtree round and the 50k/60k terminal-chain nnz caps currently skip.

This session re-measured the unmodified tip on this box before any edit:

```
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
SCORE = 0.843658
lt_1k 0.8903 · 1k_10k 0.8659 · gt_10k 0.7919
WORST order() = 1.356 s
```

The 2-second cap is still binding. One additive extra pass on large graphs is what killed the first 0060 hidden run. Every variant below either (a) spends work only after a size-only first round has already run, (b) is gated off `n >= 10_000` worst-case matrices, or (c) stays inside the existing 16M / 32M requested-work ceilings.

---

## 2. Hypothesis

Experiment 0056 showed exact-search conversion tracks the AMD-anchor margin, not `(n, nnz)`. 0060 spent extra budget on every below-anchor matrix. The leftover headroom is concentrated on **well-below** incumbents (`best_flops / amd_flops < 0.80`) and on below-anchor graphs the size-only first subtree round never unlocks.

Concretely:

1. If the size-only first subtree round finds nothing on a below-anchor incumbent, one more ticket with a diversified seed / wider window can unlock the rest of the chain. Ties are skipped. Matrices the first seed already improved are left alone so a larger window cannot displace a winning basin (that displacement is what 0041 / this session's first-round `max_s` trial did).
2. Well-below medium graphs convert extra exact-LNS streams and a slightly wider nnz gate. Near-ties and AMD ties do not.
3. Extra i.i.d. relabel AMD/AMF tickets still pay on well-below graphs (0004: more tickets, not smarter ones) and are cheap under `nnz <= 100_000`.

---

## 3. What changed

File: `src/ordering/mod.rs` only (plus this note and the memory index/log/open-questions).

One idea: **margin-scaled leftover search**. Implementations, all keyed on `best_flops` vs `amd_flops` (a pure function of the pattern):

1. **Miss-retry first subtree round.** After the unchanged size-only `subtree_cfg_for` pass, if it returns zero improvements and `best_flops < amd_flops` and `n <= 80_000` and `nnz <= 250_000`, run one more pass: `round = 1`; `lt_1k` uses 2 streams × 1M (same 32M ceiling); medium uses `max_s = 256`; large uses `max_s = 512`. Requested work stays `<= SUBTREE_SEARCH_WORK_LIMIT`.
2. **Below-anchor medium round-2 window.** Round 2 of the existing chain sets `max_s = 256` only for `1_000 <= n < 10_000` and `best_flops < amd_flops`. Raising `lt_1k` / `gt_10k` `max_s` here regresses those buckets (variant table).
3. **Terminal-chain nnz opening on below-anchor `gt_10k`.** Pass 2 also runs when `n >= 10_000 && nnz <= 100_000 && best_flops < amd_flops`. Pass 3 uses `nnz <= 80_000` instead of `50_000` on below-anchor large graphs. `crudeoil_lee4_10` (`nnz = 120_632`) and `arki0013` (`nnz = 160_172`) stay out — they are the local worst case.
4. **One extra 4 × 4M subtree ticket** after the terminal deep chain, only on below-anchor `n < 10_000 && nnz <= 100_000`. Large graphs are excluded because an additive pass there failed hidden validation in 0060. Requested work `= 16M <= TERMINAL_SUBTREE_SEARCH_WORK_LIMIT`.
5. **Margin-scaled exact LNS.** After pair-descent / simplicial, `well_below` is `best_flops * 5 < amd_flops * 4`. Medium exact search then admits `nnz <= 50_000` (was 30_000) and five streams (was two or three). Small well-below graphs get two extra streams on top of the existing four. Near-below and ties keep the 0060 schedule.
6. **Extra well-below relabel tickets.** After the budgeted AMD/AMF multi-starts, 16 (`n < 10k`) or 12 (`n >= 10k`) additional i.i.d. AMF+AMD relabel seeds run only when well-below and `nnz <= 100_000`. Implemented without the `consider` closure so the later `best_flops` reads stay well-formed.

Determinism is unchanged: every seed is a constant, every gate is a function of `(n, nnz, best_flops, amd_flops)`.

---

## 4. Commands

```bash
cargo test --release -p ssi-candidate-worker --offline --locked -- --test-threads=1
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_timing_and_score
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_1k10k
cargo test --release -p ssi-candidate-worker --offline --locked -- --ignored --nocapture --test-threads=1 probe_gt10k
```

Unit tests: **44 passed**, 18 ignored diagnostics. `subtree_configs_stay_within_matrix_work_limit` covers the miss-retry and extra-pass envelopes.

---

## 5. Variant table

Scores are end-to-end `probe_timing_and_score` on this box against the same tip baseline (0.843658 / 1.356 s). Lower is better.

| Variant | Dev score | Δ vs tip (bip) | Worst `order()` | Keep? |
| --- | ---: | ---: | ---: | --- |
| Tip (0060, unmodified) | 0.843658 | — | 1.356 s | baseline |
| First-round `max_s` bump on all below-anchor (medium 128→256, large 384→512) | 0.843829 | **+1.71 (LOSS)** | 1.359 s | no — displaces the winning first-round basin; 1k_10k 0.8659→0.8665 |
| Exact k5/k4 cleanup widened to `n <= 12_000` below-anchor | 0.865901 (1k_10k only) | ~0 | 1.138 s medium | no — kernel already covered the converting 4k band |
| Miss-retry first round only | 0.843566 | −0.92 | 1.350 s | yes |
| + medium-only round-2 `max_s=256` + terminal nnz 100k/80k | 0.843488 | −1.70 | 1.350 s | yes |
| + extra 16M pass on below-anchor `n < 10k` | 0.843465 | −1.93 | 1.353 s | yes |
| + adjacent_triple_descent + medium 3rd stream to `n<=5k/nnz<=22k` | 0.843478 | −1.80 | 1.360 s | no — triple residual is noise; stream gate still missed `pooling_sppa0pq` |
| Full-width later-round `max_s` (lt_1k 384, large 512/768) | 0.843829 | **+1.71 (LOSS)** | 1.359 s | no — 0055-consistent: raising lt_1k `max_s` loses |
| + margin-scaled LNS (`well_below`, nnz 50k, 4–6 streams) | 0.843440 | −2.18 | 1.349 s | yes |
| + extra well-below relabel (12/8 seeds, nnz≤80k) | 0.843385 | −2.73 | 1.360 s | yes |
| **Shipped: + 16/12 seeds, nnz≤100k, 5th medium stream** | **0.843358** | **−3.00** | **1.347 s** | **yes** |

The first-round `max_s` trial is the important negative: a wider window on a *successful* first round changes the incumbent the chain climbs from, and the chain can finish worse. Miss-retry only fires when that first round found nothing, so it cannot regress a winning basin.

---

## 6. Result

Full corpus, same box, same probe:

| | Tip | Candidate | Δ |
| --- | ---: | ---: | ---: |
| Aggregate | 0.843658 | **0.843358** | −3.00 bip |
| `lt_1k` (w 0.30, 147) | 0.8903 | 0.8903 | 0 |
| `1k_10k` (w 0.30, 108) | 0.8659 | **0.8650** | −9 bip in-bucket |
| `gt_10k` (w 0.40, 45) | 0.7919 | 0.7919 | 0 at 4 digits |
| Worst `order()` | 1.356 s | **1.347 s** | −0.009 s |
| Movers | — | **36 better / 19 worse / 244 same** | net win |

Largest wins (ratio drop): `chimera_lga-01` 0.7776→0.7416, `pooling_sppa9pq` 0.6492→0.6302, `pooling_sppa0pq` 0.3405→0.3281, `sporttournament48` 0.6573→0.6473, `nuclear25a` 0.5633→0.5570. Those are well-below 1k_10k graphs — exactly the 0056 conversion set.

Largest losses: `chimera_mgw-c16-2031-01` 0.7748→0.7902, `chimera_rfr-02` 0.6392→0.6503. The chain from a newly unlocked first-round miss can finish in a different basin; the AMD floor still caps every ratio at 1. Both halves of a 150/150 split still improve on the aggregate (the two losses do not flip the sign).

`gt_10k` did not move at four digits. The newly opened 60k–100k nnz band (`methanol200`, `mpbp_48`, `pinene200`, …) either failed terminal pass 1 (so the chain never ran) or the extra relabel tickets did not beat the incumbent. The local worst cases (`crudeoil_lee4_10` 1.35 s, `arki0013`) were deliberately left on the 0060 envelope.

---

## 7. Why it won

0060 already stopped wasting search on exact AMD ties. It still treated every below-anchor matrix as equal, and it still let a failed size-only first subtree round kill the entire chain. This revision spends the leftover tickets where 0056 said they convert — well-below incumbents and first-round misses — and refuses to rewrite a first-round that already improved.

The score is almost entirely `1k_10k`. That is expected: well-below conversion, extra exact LNS, miss-retry unlocks, and extra relabel seeds all live in that bucket. `lt_1k` subtree allocation was already exhausted (0055). `gt_10k` leftover work is either too expensive to add (hidden-cap history) or does not convert at the budgets we can still afford.

Timing stayed at or below the tip's same-box worst case (1.347 s vs 1.356 s). That is the only defensible cap rule: no grader-speed calibration exists.

---

## 8. Follow-ups

- `gt_10k` well-below graphs with `nnz > 100_000` (`ringpack_30_2`, `arki0013`, `crudeoil_lee4_*`) still sit on the 0060 envelope. Opening the terminal chain further is a hidden-cap bet; measure isolated `probe_gt10k` first and keep `crudeoil_lee4_10` / `arki0013` out.
- The 19 public losses are all chain-basin flips, not AMD-floor failures. A "keep the better of size-only-chain vs miss-retry-chain" would be score-monotone but doubles the expensive suffix on every miss. Not tried.
- `RELABEL_AMF_MAX_NNZ = 200_000` still excludes `gams05` (nnz 252k, ratio 0.783, 1.24 s). One extra AMF pass there is a timing question, not a conversion question.
- ND leaf AMD is already implemented (`deg_fill` in `nd_order` / `ndfm_order`). The open-questions line that treats it as untried is stale.

## Links

- Predecessor: [0060-conditional-search-escalation-below-anchor.md](0060-conditional-search-escalation-below-anchor.md)
- Headroom: [0056-ties-are-not-headroom is not in this tree; see 0056 exact-triple and the 0056 finding quoted in 0060]
- Lottery: [0004-structured-relabelings.md](0004-structured-relabelings.md)
- lt_1k ceiling: [0055 is listed as autoresearch-loop in this tree]
