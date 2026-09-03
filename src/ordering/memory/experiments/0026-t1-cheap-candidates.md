# 0026 — T1 cheap candidates: wire DegP125/Ammf + ND-leaf AMD + best_flops hygiene

- **Date:** 2026-09-03
- **Parent:** `971649b` (promoted hidden score 0.876273; dev 0.850594)
- **Status:** submitted without a local benchmark run (operator instruction);
  sandboxed build + all unit tests pass. No local score claimed.

## Hypothesis

Three zero-envelope changes, each free upside under the best-of floor:

1. Two already-implemented `custom_metrics` variants (`DegP125`, `Ammf`) under
   the SAME `nnz <= 300k && nnz >= 10n` gate are distinct lotteries at one
   AMF-class pass each (diversity of objective, cf. [0005](0005-relabelled-amf-multistart.md)).
2. `nd_order` / `ndfm_order` leaves (`ND_LEAF=200`, `NDFM_LEAF=100`) ordered by
   degree sort do better under minimum degree on the induced subgraph — the
   textbook hybrid. Leaves are tiny, so one AMD each is microseconds.
3. Rounds 2/3/5 of the subtree chain set `best_perm` without updating
   `best_flops` (round 4 does both). Harmless today (the terminal pass
   recomputes `incumbent_flops`), latent bug for any later pass.

## Change (all in `src/ordering/mod.rs`)

- Custom gate block now iterates
  `[SqDiv, SqPure, DegP125, Ammf] x alpha {1.0, 10.0}` (+4 passes, no new gate).
- New `amd_leaf_order(subset, adj)`: sorts the subset ascending, builds the
  induced CSC, runs `feral_amd::amd_order` inside `catch_unwind`, maps back to
  global indices; `None` on any failure. Both `deg_fill` closures try it first
  and fall back to degree sort. Deterministic, panic-safe (leaf code runs
  outside `consider`).
- Added the three missing `best_flops = fX` assignments (round-5 one is
  dead today — compiler warns `unused_assignments` — kept for consistency).

## Verification

- `bash scripts/local-candidate-build.sh`: builds (warnings only).
- `cargo test --release -p ssi-candidate-worker --offline --locked`:
  25 passed, 0 failed, 12 ignored (probe suite).
- `cargo test --release --offline --locked` (parent): all suites green,
  including `time_cap` (5 passed).
- No `yukon run` was performed in this session, so there is no dev-score
  delta to report. The grader's hidden score is the verdict.

## Lesson / next

- Keep new candidates inside measured gates until `probe_family` prices them;
  T2 (relabel lotteries) and T4 (large-matrix passes) remain the bigger,
  riskier leads. If this grades neutral, the T1 mechanism is exhausted and the
  next increment must come from T2/T3 replacement work.
