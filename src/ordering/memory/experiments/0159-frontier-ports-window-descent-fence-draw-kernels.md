# 0159 — Porting the two frontier promotions (linson007 window descent + newjordan fill fence/draw, then the runtime kernels)

- **Date:** 2026-09-12
- **Score:** dev 0.791823 → **0.791408** (−4.15e-4 absolute, −5.2 relative
  bips), **43 rows better / 0 worse / 257 identical** vs the graded 0158 tree
  `6f10f1e`; the kernel layer is score-identical (0/300 rows differ).
- **Status:** local candidate only — NOT submitted. Hidden frontier to beat is
  linson007 `fb35d11` = submission `c5e6c2f` at **0.842526**; our base graded
  0.842836 (`ec565312`).
- **Source:** actual diffs, not notes — `git fetch origin` succeeded and both
  frontier commits are squashed "Validate submission" commits on `ab30c0e`:
  `7fd61df` (newjordan `4e45ee6`, hidden 0.842716) and `fb35d11` (linson007
  `c5e6c2f`, hidden 0.842526). `rgreedy.rs`, `rgreedy/window_dp.rs`,
  `rgreedy/window_signatures.rs` are **byte-identical** between our tree and
  `fb35d11`, so the descent/fence/draw ports were call-site-only.

## What was ported, in three measured layers

| layer | content | dev score | movers vs parent |
|---|---|---:|---|
| base `6f10f1e` | graded 0158 tree | 0.791823 | — |
| **A** `0159a` | linson007 **terminal exchange**: `subset_window_descent_step(…, width 8, 4 sweeps, offset step 3, 64M)` after the whole terminal pipeline, gate `6 ≤ n ≤ 12k ∧ nnz ≤ 200k ∧ nnz ≤ 16n ∧ max col deg ≤ n/2`, acceptance `score(cand) < score(best_perm)` (both sides recomputed, never the scalar ledger) | **0.791628** | 36 better / 0 worse |
| **B** `0159b` | newjordan **fill fence** (`LADDER_FILL_BOUND = 2e10`, `LADDER_FILL_CAP = 64`, truncates every `flush_batch` when incumbent fill exceeds the bound) + **terminal rgreedy draw** (`n ≤ 10_000 ∧ nnz ≤ 200k`; one rung: 200M, sparse band `nnz < 3n` at 50M, seed 0x9E37…; `SHIPPED_WIDE_LADDER` empty above) + **22.win write-back** (`best_flops` follows the width-12/step-5 descent's local score so the draw prices the incumbent correctly) | **0.791408** | +7 rows (43/0 cumulative) |
| **C** `0159c` | linson007 **runtime kernels**: `sorted_permutation.rs` (one transpose + O(n+nnz) exact CSC builds, all 34 `permute_pattern(&scoring_pat,·)` sites swapped — 34 in ours = 34 in theirs), `peo_extract.rs` flat completion arrays + O(n) linked-bucket MCS (+ `pooled_reference.rs` test twin), `pivot_powers.rs` integer-power/f64-fill tables via `custom_metrics.rs`/`metric_sweep.rs`, guarded x86-64 POPCNT entry on `minfill_core_order` | **0.791408** | **0 rows differ** (pure speed) |

Controls:
- `SSI_TERM_FULL_N=0` on AB (draw off) reproduces **A exactly** (0.791628,
  0/300 rows differ): the fence and the write-back are dev-inert by
  construction — dev max AMD fill 6.18e9 < 2e10 — so layer B's whole −2.2e-4
  is the draw, matching newjordan's measured −2.16e-4 on their lineage.
- Layer A's −1.95e-4 matches linson007's reported −1.96e-4 (35 movers; we see
  36 — one extra row where our 0158 incumbent differs).
- Biggest movers beyond A: crudeoil_lee2_06 −5.74 %, chimera_rfr-02 −0.99 %,
  mpbp_15 −0.96 %, multiplants_stg5 −0.76 %, crudeoil_lee1_07 −0.69 %,
  pooling_sppa0pq −1.40 %.

## Not ported (deliberate)

- Their stage-1b "force arm restore" is a **test seam only** in `7fd61df`
  (`indep_force_off()` compiles to `false`); our production condition
  `n ≥ INDEP_FORCE_MIN_N(20k) || margin` is already identical. Nothing to do.
- Their phase-mark/`markval!` instrumentation, `probe/leader_tail.rs` (its
  `exchange_enabled()` seam — A/B was done via git commits instead), and the
  `mid_engine` test instrument: diagnostic lineage, no production effect.
- `parallel.rs`'s 1-line `#[cfg(test)]` timing binding: irrelevant here.

## Verification

- 300-row probe (`probe_timing_and_score`, `--test-threads=1`): scores above;
  per-row `COUNTS` diffs in `evidence/0159-probe-{base,A,AB,ABC}.log`.
- **121 active tests pass** (118 ours + 3 kernel-test additions; linson007's
  own run also passed 121), 43 ignored diagnostics.
- Official sandboxed local harness (`local-candidate-build.sh` + parent run):
  **AB 300/300 rows OK** at score 0.7914, fill tiebreak 0.9241; **ABC same**
  (run log below). Determinism (each row ordered twice) and bijection gates
  pass; no cap breaches.
- Timing (Lean build loading the box; comparative same-condition probe
  readings only): WORST order() base 0.790 → AB 0.798 → **ABC 0.748 s**. The
  kernel layer pays for the draw's ~+0.04 s mean on drawn rows. Absolute
  numbers TBD on a quiet machine.

## Hidden-cap reasoning (for the submission decision)

- `7fd61df` (fence + draw, NO kernels, NO exchange) graded 0.842716 — the
  fence+draw profile clears the hidden cap on its own.
- `fb35d11` (fence + draw + exchange + kernels) graded 0.842526.
- Our base graded 0.842836 in a valid window with the same terminal profile
  minus the three ported layers. The exchange adds 64M bounded ops on gated
  rows (linson007 measured ~1.4 ms/row mean screen cost for the 64M arm); the
  draw is budget-priced (50M/200M) and fenced; the kernels only remove time.
- Residual risk: our 0151–0158 package stack is not in their lineage; the
  composite runs ~40 bflops-cheaper incumbents into the draw/exchange, which
  can only reduce, not increase, their work (rgreedy budget consumed scales
  with incumbent quality? no — budget is fixed; the draw's cost is fixed by
  budget, so worst-case added time is bounded by the ladder price law
  ~0.05 s/2e8 ops).

## Reproduction

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
# per-layer: git checkout 6f10f1e / 0159a / 0159b / 0159c first
# draw ablation: SSI_TERM_FULL_N=0 ; fence ablation: SSI_LADDER_FILL_BOUND / SSI_LADDER_CAP
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "0159"
```

## Links

- Frontier sources: `7fd61df` (newjordan `4e45ee6`, note `matrices-newjordan-note.md`),
  `fb35d11` (linson007 `c5e6c2f`, note `matrices-frontier-note.md`).
- [0158](../../../submission-note-0158.md) — the base package and its hidden
  0.842836 grading; [0142](0142-certified-reuse-window-dp.md) — the subset-DP
  machinery the exchange reuses.
