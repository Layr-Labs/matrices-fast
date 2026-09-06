# 0065 — Subtree recursion on the K = 3 core (harness r7 lever L-CORE-RECURSION-SUBTREE-r7)

- **Date:** 2026-09-05
- **Base:** S5 of [0064](0064-multi-depth-prefixes.md) on the `2a28517` tree (hidden 0.867211)
- **Score (lever alone, on the bare 2a28517 crown, harness assay):** out-of-sample 0.856455 → **0.855843** (−7.1 rel bp); dev 0.839063 → **0.838635**; 14 better / 0 worse on EACH corpus; 40/40 hash-drawn held-out witnesses tie or better
- **Score (folded on S5 = this submission):** dev 0.836273 → **0.835979**; out-of-sample 0.855311 → **0.854948**; combined vs the crown: dev −37 bip (18 / 0), out-of-sample −18 bip (24 / 1 at 0.2%)
- **Status:** SUBMITTED with 0064 (S6). Pinned 2-core min-of-3 on the 32 slowest rows: crown 1.241 s → 1.117 s, no row slower by more than 0.05 s; 53 tests

## Hypothesis

The exact-ranked K = 3 argmin of 0062 is the LAST word of the pipeline, but every polisher (the subtree-refinement chain,
the relabel windows, the LNS streams) stands upstream of the terminal block: the core's ordering ships raw. Because the
objective splits exactly (`flops(full) == prefix_flops + flops(core)`), any strict improvement of the core ordering on the
core graph is a strict improvement of the spliced full ordering — so the crown's own subtree-refinement chain can be run
on the core graph, with the same laws, before splicing.

## Mechanism

`refine_core(cn, core_col_ptr, core_row_idx, core_pat, cp, f_core, f_amd_core)` runs the subtree-refine chain on the core
graph from the K = 3 argmin and returns the refined core permutation only if its trusted core flops are strictly lower.
Gates (structure only): `nnz < REDUCE_RECURSE_MAX_NNZ = 150_000`; the raw spliced candidate must already be within
`REDUCE_RECURSE_MARGIN = 11/10` of the incumbent (a core that cannot win is not polished); `DEEP_MAX_CORE_N = 80_000`,
`DEEP_MAX_CORE_NNZ = 250_000` bound the chain's work. Exact by the split (unit test binds `flops(full) == prefix + core`),
monotone (admitted through the same strict `<`), panic-fenced, deterministic (no timing reaches the output).

## Result (harness r7.1, crown alone, pinned interleaved min-of-3)

Worst call unchanged on both corpora (the worst row of each corpus takes a bit-identical path); at most 0.047 s added to
any row; 14 / 0 per corpus; 40/40 witnesses. Critic class: structural — the win holds under any draw.

## Links

- [0062](0062-reduce-then-amf-terminal.md), [0064](0064-multi-depth-prefixes.md); harness record `matrices_mage/harness/chronicles/2026-09-05_r7_core-recursion-validated.md`
