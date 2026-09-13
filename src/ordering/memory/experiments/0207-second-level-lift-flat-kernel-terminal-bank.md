# 0207 — Second-level lift of the winning Schur core, a flat exact symbolic kernel, two dead terminal restarts re-spent

- **Date:** 2026-09-13
- **Score:** dev 0.791802 → **0.791782** (−0.25 relative bip, 2 better / 0 worse); held-out (591 MINLPLib
  KKT patterns disjoint from dev) 0.791105 → **0.790596** (−6.43 bip, 14 better / 0 worse)
- **Status:** package of three levers on the crown `475be33` (newjordan e07fe7ae; measured identically on linson007's 0206 one promotion earlier); submitted

## Hypothesis

1. The identity behind stage 1b (0143) — an independent set eliminated first has order-free exact cost —
   applies again inside the lifted core. On the WINNING core, when the first lift already beats the raw
   incumbent and the core has ≥ 1,000 nodes, lift the cap-3 greedy independent set `X2` of `core1`,
   order `core2` by AMD / AMF / one quotient metric, and replace the stage candidate only on a decisive
   win (`f2·50 ≤ f·49`). Own ledger 400k edge-touch units (1/20 of the stage's), charged before any
   work, funded by retiring the never-adopted `DegDivNvDegme` core pass.
2. The 37 remaining vendor symbolic sites in `mod.rs` (+ `minl::filled_graph`) go through one flat `u32`
   kernel, bit-identical and cheaper — the time that pays for (1) on exactly the rows where (1) fires.
3. Terminal SmallScore restarts 3 and 4 are dead when restart 2 is a no-op; skip them and spend the two
   walks last, seeded from the reverse-MCS PEO of the completion and from an alternate stream.

## What changed

- `indep_first.rs` (+103): `INDEP_L2_LEDGER = 400_000`, `INDEP_L2_MIN_CORE_N = 1_000`,
  `INDEP_L2_MARGIN = (49, 50)`; `run` takes the incumbent; `second_level(lc, ledger)`; the metric pass
  list drops `DegDivNvDegme`. `mod.rs`: the stage-1b call passes `best_flops`.
- new `symbolic_flat.rs` (402 lines, from 0151): `analyze`, `flops`, `analyze_sorted`, `prep_subtree`;
  `flops_of`, the 13 subtree preambles, the three final-refine preambles, the seven sorted consumers,
  the peel and `minl::filled_graph` route through it; zero vendor symbolic calls remain outside tests.
- `mod.rs` terminal restarts + `peo_extract::reverse_candidate_bounded`: `terminal_bank` skips restarts
  3/4 after a no-op restart 2 and spends two seeded walks last (`cutoff_paired_swap_refine_seeded`,
  `cutoff_plateau_refine_seeded`; `TERMINAL_*_SEED`, `TERMINAL_SEED_MAX_LNNZ = 100_000`), strict exact
  accept, gate `12 <= n <= 1_000 && nnz <= 12_000`.

Diff: 4 files +281 / −266 plus the new file. No matrix name, no `(n, nnz)` cell.

## Results (this box, x86, `taskset -c 0-1`)

| tree | dev | held-out | rows dev / held-out |
|---|--:|--:|---|
| crown 52affcb (0206) | 0.792081 | 0.791265 | — |
| + terminal bank | 0.792061 | 0.791249 | 2/0 / 8/0 |
| + bank + kernel | 0.792061 | 0.791249 | identical (kernel bit-identical) |
| **+ bank + kernel + second-level lift** | **0.792061** | **0.790762** | **2/0 / 12/0** |
| crown 475be33 (newjordan) | 0.791802 | 0.791105 | — |
| **package on 475be33 (shipped)** | **0.791782** | **0.790596** | **2/0 / 14/0** |

Lift rows (held-out): `crudeoil_lee4_05` −12.55 %, `crudeoil_lee2_08` −8.95 %, `crudeoil_lee3_06`
−6.99 %, `gabriel05` −0.55 %. On the previous crown, under the harness (40/40 witnesses tie): 4/0,
−5.83 bip, dev identical. 126 tests; harness run-twice `score.json` = probe. Envelope: see the note.

## Caveat

Three rows of one family carry most of the gain and dev's five `crudeoil_lee` rows do not move; the
hidden transfer rides on that family's presence. Not shipped: the crown author's 0205 restored on this
package (−1.2 bip more on 27 held-out rows) — its value sits in admitting factors above 150k nonzeros to
the terminal window passes, and that line alone costs +31 % on the ≥ 0.80 s class (the cap killer);
per-row cost laws at 10M and 20M units recovered none of its rows.
