# 0208 — Reduce, then lift: the independent-set lift on the exact degree-<=3 residual core, run last

- **Date:** 2026-09-14
- **Score:** dev 0.790266 → **0.790045** (−2.8 relative bip, 4 better / 0 worse); held-out (591 MINLPLib
  KKT patterns disjoint from dev) 0.790330 → **0.789748** (−7.4 bip, 6 better / 0 worse)
- **Status:** package of two levers on the crown `99de589` (bbf5849); measured identically on 5212fea one
  promotion earlier; submitted

## Hypothesis

The reduce (0062) and the lift (0143 / 0207) are two exact objective splits that had never been composed:
the lift only saw the raw pattern, the reduce core only saw the minimum-degree family. The residual core
after degree-<=3 elimination is a different, denser graph; on some rows the lift orders it far better
(pooling_sppb0tp −17.9 %, crudeoil_pooling_ct4 −3.3 %, nuclear25 −2.7 %). Placement matters twice:
mid-pipeline the lift fires once per reduction depth (+1 s on big cores) and hands a different incumbent
to the chain (6–8 rows worse per corpus); TERMINAL it runs once, is spliced through the reduce, scored
exactly and accepted only on a strict decrease of the full score.

## What changed

- `mod.rs`: `TERMINAL_CORE_LIFT_LEDGER = 500_000`; the pipeline's K=3 reduce (`cl3`) is kept alive
  (`saved_cl3`) and reused by a terminal block placed before the banked terminal walks:
  `indep_first::run_narrow(&core_pat, TERMINAL_CORE_LIFT_LEDGER, u64::MAX)` → `core_lift::splice` →
  exact score → strict accept. Gate: the reduce block's own (`n >= REDUCE_MIN_N && nnz <= REDUCE_MAX_NNZ`,
  `cn >= INDEP_MIN_N && cn < n && core_nnz <= REDUCE_MAX_CORE_NNZ`); `catch_unwind` around the lift.
- `indep_first.rs`: `run_narrow` = the lift with phase 2 restricted to cores within 5 % of the best phase-1
  total (stage 1b keeps its 3/2), and — for the terminal call only — candidate sets restricted to the four
  kinds a census of every terminal win found responsible for all ten (ginf, sc_inf, sc_15, sc_9; the cap-k
  and sc_5 sets never won); `second_level` gains `Pass::Metric(V::DegPlusDegme)` beside `DegDivNvSqrtWf`.

Diff: 2 files, +59 / −3. No matrix name, no `(n, nnz)` cell.

## Results (this box, x86, `taskset -c 0-1`)

| tree | dev | held-out | rows dev / held-out | cost on the ≥ 0.80 s class |
|---|--:|--:|---|---|
| lift in `order_core` (mid-pipeline, ledger 2M), on 5212fea | 0.787624 | 0.786409 | 22/8 / 27/6 | +0.5..1.4 s on big cores |
| terminal, ledger 2M | 0.789631 | 0.787596 | 6/0 / 6/0 | +1 s on 20–60k-node cores |
| terminal, ledger 500k, full envelope | 0.790185 | 0.789750 | 5/0 / 6/0 | 16 rows +0.07..0.10 s |
| terminal 500k, narrow phase 2 + winning set kinds | 0.790185 | 0.789750 | 5/0 / 6/0 | 11 rows +0.05..0.09 s, WORST under the crown |
| crown 99de589 (bbf5849) | 0.790266 | 0.790330 | — | — |
| **package on 99de589 (shipped)** | **0.790045** | **0.789748** | **4/0 / 6/0** | envelope in the note |

126 tests. Closed alongside: second-level margin 1 % / core >= 500 / ledger 800k (0 rows), cap-5 (loses
the four 0207 wins), near-miss admission (0 rows), two X2 sets (0 held-out rows), a lite lift (AMD-only:
0 wins), a phase-1 bail on the incumbent ratio (drops the winners: sppb0tp's phase-1 total is 8.4x the
incumbent and only phase 2 turns it into a win), ledger 250k (loses sppb0tp).

## Caveat

Six held-out rows carry the gain, one of them (pooling_sppb0tp, −17.9 %) most of it; dev moves on four
different rows. No shape predicate separates the cores the lift wins on from the ones it merely pays for
(independent-set fraction and 2-colouring conflicts overlap), so the cost on ~10 slow rows (+0.05..0.09 s
on this box, worst call still under the crown's) is the price of the mechanism, disclosed in the note.
