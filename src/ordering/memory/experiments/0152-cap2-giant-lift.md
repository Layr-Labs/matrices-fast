# 0152 — Cap-2 independent set on the giant band (+ the 0151 AmindNorm keep)

- **Date:** 2026-09-11
- **Score:** dev 0.792439 → **0.791831** (−6.1 bips; −2.1 bips beyond the kept
  0151 AmindNorm change); exactly 2 rows changed, 0 regressions
- **Status:** bundle `3c2e1f3` FAILED the hidden 2 s cap; the isolated
  retightened form `3aef8b6b` (nnz ceiling 120k) CLEARED the cap but graded
  **0.842857 = frontier bit-identical** (rejected, score-neutral). Verdicts:
  (a) the timing death lived in the 120k–400k giant band — one extra lift
  plus a basin shift there is enough to kill a run, do not re-open cap 2
  above 120k nnz without a work-priced gate; (b) the hidden corpus contains
  no gabriel09-class row (n ≥ 20k, nnz ≤ 120k) where the cap-2 lift wins.
  The change stays in-tree: cap-clearing, output-neutral where it does not
  win, keeps the dev gain.
- **Files:** `indep_first.rs` (sparse `extra_caps` gains a trailing cap 2 for
  `n >= 20_000`; 0151's AmindNorm gate unchanged)

## Mechanism

The 0151 census (`probe_indep_sets`) found the cap-2 greedy independent set
(g2) wins rows the caps {15,9,5,3} miss. Shipped UNCONDITIONALLY it nets
−4.6 bips but with a real casualty: `crudeoil_lee4_09` **+2.58 %** — admitting
one more set shifts the stage-1b incumbent into a worse downstream basin on
that row (exactly the basin sensitivity 0150 documents: a stage-1b admission
change alters which candidate the subtree/transplant/MINL chain polishes).

The fix is structural, not a fitted window: cap 2 is admitted only on the
giant band `n >= 20_000` — the same monotone threshold the tree already
enshrines as `INDEP_FORCE_MIN_N`. Result:

| row | frontier | with cap-2 giants |
|---|---:|---:|
| gabriel09 (n 21,688, nnz 89,702) | 26,366,112 (1.0000) | **25,811,173 (0.9789)** |
| crudeoil_lee4_09 | 130,655,525 | unchanged (outside the band) |
| edgecross24-115 | 28,685,704 | 26,515,367 (0151's AmindNorm keep) |

298/300 rows bit-identical. On the giant band the 15/9/5/3 caps already run,
so cap 2 adds ONE ledger-bounded lift+AMD walk there (~10–50 ms) — the same
cost class as the existing caps, not a new stage.

## Timing

Probe min-of-2, load ~110 (all numbers inflated; comparative reading only):
worst row `crudeoil_lee4_09` 1.26 s — byte-identical to the frontier there;
`gabriel09` 0.97 s; `nuclear104` 1.20 s; `dt3` 1.14 s. No changed row
approaches the frontier-worst band, and the additive work is one lift inside
an existing admission ledger. NOTE: `yukon run` cap breaches under this box's
load are noise — the unmodified frontier measures 2.2 s on `powerflow0300p`
at load 26; only probe min-of-N comparisons are meaningful here.

## Also censused this session (all negative, do not retry)

- **Scotch/KaHIP on lifted cores** (`SSI_SETS_ND`): one negligible row
  (waternd2 kahip 0.9983, lt_1k). KaHIP's per-call constant (~0.15 s even on
  small cores × up to 13 admitted sets) breached the local cap on slay09h —
  never ship kahip-on-cores outside METIS's own top-k discipline.
- **Exact MinFill on lifted cores** (`SSI_SETS_MF`): zero wins.
- **Relabelled-AMD on lifted cores**: one hit, acopf_case9241pegase_qcqp
  (g-core cn 201k/cnnz 1.14M, seed 3, 0.9979 of pipeline ≈ 0.19 dev bips) —
  one extra AMD pass on the slowest row class for a sub-bip return; skipped
  on timing grounds (that class is where hidden-cap deaths live).
- **α ∈ {2.5, 5} on core metric passes**: no win beyond what α=10 already
  captures (edgecross α-variants are strictly worse than the shipped path).
- **supplychain family**: min-degree = feral-AMD exactly (2,173,490 to the
  digit); demoting the deg-7 middle layer is catastrophic (455×). The hub
  interleave AMD picks there is genuinely strong; the family's 21× lb gap is
  bound looseness, like squfl/emfl in 0151.

## Links

- [0151](0151-amindnorm-on-small-cores.md) — the AmindNorm half of the bundle
  and the hidden-neutral grading that motivated bundling.
