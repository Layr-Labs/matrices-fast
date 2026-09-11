# 0151 — AmindNorm metric pass on small lifted cores

- **Date:** 2026-09-11
- **Score:** dev 0.792439 → **0.792039** (−4.4 bips), fill tiebreak 0.924472 → 0.9243;
  gt_10k 0.685651 → 0.684514; lt_1k / 1k_10k unchanged
- **Status:** win, single-hunk submission (one metric variant, one gate)
- **Files:** `indep_first.rs` (phase-2 task list: `V::AmindNorm` gated to
  `cn <= 12_000 && cnnz <= 100_000`)

## Hypothesis

0145 shipped six quotient-metric passes on the top metric cores of the
independent-set lift but never swept the remaining `custom_metrics` variants
(`Ammf`, `AmindNorm`, `DegDivNvWfP15`, …). The 0151 census
(`probe_indep_sets SSI_SETS_CM=1` over all 256 admitted rows × 13 sets × 14
passes) found exactly one marginal win beyond the pipeline ∧ shipped sets:

| row | set | pass | ratio |
|---|---|---|---|
| edgecross24-115 (n 16.7k, nnz 77k) | x9 (cn 8651, cnnz 85k) | **AmindNorm** | pipeline 0.8173 → **0.7960** at 15 ms |
| hydroenergy1 (n 1046) | g2 | amf | 1.0000 → 0.9986 (not taken; needs a new set admitted on every sparse row ≤ 20k for 0.3 bip — poor cost/benefit) |

`AmindNorm` = approximate minimum increase in neighbor degree, normalized by
neighbor count — an AMF-family objective distinct from the six shipped ones.

## The gate is load-bearing

The first attempt added `AmindNorm` to the shipped list unconditionally and
**FAILED the local 2 s cap** on `crudeoil_pooling_dt3` (n 30.7k, nnz 152k):
its admitted cores (cn 15–22k, cnnz 128–144k) make the AmindNorm walk cost
**0.105–0.915 s per core** (vs 15 ms on the edgecross core) and score
141–261× AMD there. The variant's cost does NOT track the other six —
same situation as `HEAVY_METRIC_WF2_MAX_NNZ` for `extra_deg_div_nv_wf2`.
The shipped gate `cn <= 12_000 && cnnz <= 100_000` sits below the measured
blow-up band (128k+) and covers the measured win (85k), mirroring the
per-variant ceiling pattern.

## Result (full dev corpus, one run, same box)

| | baseline (ab30c0e) | with AmindNorm |
|---|---:|---:|
| score | 0.792439 | **0.792039** |
| gt_10k | 0.685651 | 0.684514 |
| rows changed | — | **1** (edgecross24-115, 28685704 → 26515367 = **−7.57 %**) |

The single raw census win (−2.6 %) more than doubles after acceptance because
the lift becomes the incumbent and the subtree / transplant / MINL chain
polishes it; the other 299 rows are **bit-identical** (strict `<` acceptance;
the variant runs after the six, so task order and tie-breaking are unchanged).

## Timing (SSI_PROBE_REPEAT=2, loaded box, top-22 rows)

| | baseline | with change |
|---|---:|---:|
| worst (`crudeoil_lee4_09`) | 1.288 s | 1.084 s |
| `edgecross24-115` | 0.737 s | 0.604 s |

The deltas are within the documented ~1.6× run-to-run band; the added walk is
15 ms on the winning core and bounded by the gate elsewhere. Timing-neutral
against the tip that cleared the hidden cap three times.

## Generalisation argument

The mechanism is a *portfolio candidate family* — AmindNorm on a lifted core
either strictly beats the incumbent or is rejected — so on the hidden corpus it
can only move rows it wins. The gate is on core structure (cn, cnnz), never on
instance identity. If the hidden corpus contains no edgecross-like row the
submission grades score-neutral (and is rejected as such); the fill tiebreak
also improved, which breaks exact ties in our favour.

## Negative results recorded this session (do not retry)

- **Exact min-degree with FIFO/random tie-breaks as a candidate family.**
  An apparent large win on the squfl/emfl/supplychain ties (supplychain 0.2865!)
  was an artifact: a bucket-pointer that never scanned DOWN stranded
  low-degree vertices, so the flop sum was partial. Correctly implemented,
  exact-MD beats raw AMD on 94 rows but **never beats the pipeline final**
  (min-of over both = 0.792439 exactly). Verified against the fill-free lower
  bound — always sanity-check a "win" against `n + 3e + 2t`.
- **The squfl/emfl/supplychain gt_10k ties are near-optimal, not headroom.**
  Structure (squfl030-150): 30 super-hubs (deg 150) × 150 hubs (deg 30) with
  4500 chains (leaf→deg3→deg2→hub), each (super-hub, hub) pair exactly once.
  The collapse forces a K(30,150) fill core; AMD's 252605 decomposes exactly
  as chains 58500 + deg3 40500 + hubs 144150 + clique tail 9455, and every
  interleaving alternative (kill S early, kill H before d3, kill d3 after H)
  is worse — the ties sit at the forced-fill optimum, 3.7× above a lower bound
  that no ordering can reach. Same conclusion for `polygon75`, `camshape400`,
  the `autocorr_bern*` rows (at the exact LB) and near-ties
  `watercontamination0303r` (1.018×), `meanvar-orl400_05_e_8` (1.008×).
- **Set-variant census** (g2/g4/g6/g16/f-inf/f3/f9/f16/x-inf beyond shipped):
  one marginal g2 win (−0.14 % ln, hydroenergy1, not worth a global set).

## Links

- [0145](0145-second-colour-class-metric-cores.md) — the lifted-core metric
  machinery this extends; [0150](0150-stage1b-window-removal-isolated.md) —
  the tip this builds on (hidden 0.842857).
