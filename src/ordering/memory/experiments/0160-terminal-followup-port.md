# 0160 — Porting linson007's second promotion (52affcb terminal followup)

- **Date:** 2026-09-12 (round 2, ~21:00 PT)
- **Score:** dev 0.791408 → **0.791243** (−1.65e-4; −5.8e-4 total vs the graded
  0158 base `6f10f1e`), **23 better / 0 worse** vs 0159's ABC tree;
  **48 better / 0 worse** vs base across the two sessions' ports.
- **Status:** local candidate only — NOT submitted. Hidden frontier to beat is
  linson007 `52affcb` = submission `07f0e8a` at **0.842377** (20:32).
- **Source:** actual diff `fb35d11..52affcb` (+63 mod.rs, +78 window_dp.rs,
  ±3 rgreedy.rs, +197 test-only probe). `rgreedy.rs`/`window_dp.rs` were
  byte-identical to ours at fb35d11, so both files were taken verbatim from
  52affcb; only the mod.rs block was hand-inserted (two `is_bijection(·, n)`
  arity fixes — their grep-truncated diff hid the second argument).

## Mechanism (their 0206, the promoted shape)

A `terminal_followup` stage appended after the 0203 exchange:

1. **Ledger precheck** `best_flops <= LADDER_FILL_BOUND (2e10)` + window gate
   `6 ≤ n ≤ 12k ∧ nnz ≤ 200k` — reject high-flop rows before scoring.
2. **Fresh admission**: re-scored `final_flops <= 2e10` ∧
   `score_workspace.nnz_l() <= 150_000` (factor-nnz bound — the 0205 killer
   was completion work on high-factor rows).
3. **Dense/hub arm**: one more 8/4/3/64M window on rows the exchange's
   structural gate excluded (`nnz > 16n ∨ max_deg > n/2`).
4. **Two PEO rounds** on the current incumbent via
   `peo_extract::candidates_bounded(·, MAX_N, 200k, 150k)`, strict accept.
5. **Sparse span windows**: `sparse_span_window_descent` — the new
   `window_dp.rs` entry that lifts the span width to 64 with a u64 component
   mask, solves only connected components of ≤ 14 vertices exactly, and leaves
   larger components in their slots (suffix graph unchanged). Rungs
   (48, 4, 19, 16M), (9, 4, 4, 16M), (8, 4, 3, 32M) — the halved allowances
   are the specific fix for 0205's hidden-cap death (0205 ran a late 4M
   watcher + 48M/32M/64M and died at 106 s; its content is NOT ported).

## Layered A/B this session

| tree | dev score | movers vs parent |
|---|---:|---|
| ABC `582a553` (0159) | 0.791408 | — |
| **ABCD** `0160d` (+ followup) | **0.791243** | **23 / 0** (matches their 23/0, −1.65e-4 = their exact dev delta) |

Top D movers: rsyn0810m04m −1.70 %, chimera_mgw-c16-2031-01 −1.33 %,
pooling_digabel19 −0.87 %, crudeoil_lee2_06 −0.47 %, crudeoil_lee1_07 −0.45 %.

## Verification

- **123 active tests pass** (121 + 2 new sparse-span tests = their count),
  45 ignored.
- Full 300-row probe: scores above, `evidence/0160-probe-ABCD.log`.
- Official sandboxed harness run: see log entry (300/300, score/fill below).
- Timing: DEFERRED again (Lean comparator still loading the box). Comparative
  probe WORST under load: ABC 0.748 s → ABCD 0.757 s (+9 ms for the followup
  stage — cheap, consistent with their "maximum new minimum 0.993 s" reading
  scaled to this box).

## Hidden-cap reasoning

- Their 0206 = this exact stage graded **0.842377** (cleared the cap) on a
  tree whose terminal profile is otherwise our ABC. The ledger precheck +
  150k factor gate + halved allowances ARE the cap fix; we ported the promoted
  shape, not the failed 0205.
- The stage is bounded: window allowances 16M+16M+32M (+ the gated 64M dense
  arm only on rows the exchange skipped), PEO rounds capped at 150k Lnnz, all
  admissions structural.

## Reproduction

```sh
git checkout <0160 commit>
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
# followup ablation (test builds): probe::terminal_followup::enabled() seam
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "0160"
```

## Links

- [0159](0159-frontier-ports-window-descent-fence-draw-kernels.md) — the
  exchange/fence/draw/kernel layers this builds on; frontier sources
  `52affcb` (07f0e8a), their 0205/0206 experiment pages in the fetched tree.
