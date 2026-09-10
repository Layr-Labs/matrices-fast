# 0147 — Chained transplant rounds (sibling shape + measured rejections)

- **Date:** 2026-09-09
- **Score:** tip `62654a5` dev **0.792300 → 0.792254 (−0.46 bips)** with
  up-to-two chained rounds; sibling 87e7f7b (exactly one chained round)
  reported **0.792246 (−0.54)** / hidden **0.843142 (−0.31, REJECTED)**.
- **Status:** shape kept at exactly-one-chained-round (SIBLING-FAITHFUL);
  extensions closed by measurement, submit held (known-rejected hidden EV).
- **Files:** `mod.rs` phase-14 block (+26/−0).

## Shape

Snapshot `best_flops` before the shipped transplant block; re-run the
identical `refine_with_donors` call on the new incumbent iff the shipped pass
strictly improved the row. Donor substrate credit dukemawex, chaining shape
credit darthweenies (87e7f7b). Deterministic (exact-flop conditioning, fixed
donor order); strict admit every round.

## Measured on this box (Mac, loaded)

| variant | dev | note |
|---|---|---|
| tip | 0.792300 | baseline |
| +1 chained round (sibling-exact) | ≈0.792246 (−0.54, sibling box) | hidden-validated by sibling (no timeout), hidden −0.31 |
| +up-to-2 chained rounds | 0.792254 (−0.46) | 3rd round nets −0.08 (extra cascade); lee4_09 marginal |
| chained rounds @2M ledger | FAIL (lee4_06 SIGKILL) | even warm mid rows cannot take additive depth |
| chained round-2 @1M | FAIL (lee4_10 SIGKILL under load) | marginal by itself; load blamed in part (box at ~90/8) |
| chained round-2 @150k cap | FAIL (rsyn0810m04m SIGKILL under load) | box too loaded to measure; form kept in reserve |

Movers reproduce sibling's (rsyn0820m04m −88, chimera_lga-01 −47,
methanol200 −19 row-bips) including the 3 cascade regressions
(multiplants_stg5 +49.6, crudeoil_lee1_07 +62.3, chimera_rfr-02 +7.8):
mid-pipeline wins reshuffle downstream conditional gates (0143 tax).

## Conclusions (do not retry without new evidence)

1. The chain stops at ONE by measurement: round 2 is net-negative on dev
   and timing-marginal on lee4-class rows.
2. No static `(n,nnz)` gate separates warm mid rows (lee4_06 unit 65k,
   methanol 88k) from cool ones (rsyn 23k) — hotness is structural, not
   dimensional. Unit-gated depth cannot thread them; the 150k-cap form died
   to load before measuring and stays unproven, not disproven.
3. Hidden EV of this family ≈ −0.3 (sibling measured). Below the ~1-bip bar
   with no variance story that gets there. HOLD from submitting: deterministic
   code reproduces deterministic hidden scores, so resubmitting this shape
   reproduces a known rejection. Next submit needs ≥ −2 dev bips by a
   different mechanism (MINL cool-row depth is the open candidate).
4. Box discipline: at load ~90/8, local 2 s-cap verdicts are noise (four
   straight runs each killed a different row). Trust SCORES (deterministic),
   never local pass/fail, until load clears. Sibling's hidden validation is
   the timing ground truth for this shape, not this box.

[Index](../index.md) | [Log](../log.md)
