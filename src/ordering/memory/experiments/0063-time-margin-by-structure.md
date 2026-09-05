# 0063 — Time margin by structure: partitioner cascade, structural windows on the late phases

- **Date:** 2026-09-05
- **Base:** `784bfe5` + [0062](0062-reduce-then-amf-terminal.md) (reduce-then-AMF terminal; submission `db6d00ce`)
- **Score:** dev 0.843358 (crown) → **0.839063** (-0.509% rel, 50.9 bip vs crown); independent out-of-sample corpus (591 MINLPLib KKT patterns outside dev) 0.863823 → **0.856455** (-0.853% rel, 85.3 bip)
- **Status:** GO on both corpora (dev -51 bip, out-of-sample -85 bip vs the crown; worst order() 1.084 / 0.951 s vs crown 1.823 / 1.574 s same box); submitted for hidden validation

## Why

Every upload by every solver since 07:44 UTC on 2026-09-05 (eight uploads, four solvers, the crown's author
included) was killed by the 2 s cap on a hidden matrix, `db6d00ce` (0062 alone, faster than the crown on every
slow dev row) among them. The crown's pipeline sits at the kill line on the current grader; no tree that carries
it whole can land. The only lever that matters today is wall-clock margin, and 0062's gain (−69 bip dev /
−97 bip out-of-sample) is the budget to buy it with.

## Attribution (the slow subset: 169 dev + out-of-sample rows where the crown takes >= 0.6 s)

Ten single-family ablations of the 0062 tree, each measured on the subset (`sweep_r6b/`, `make_variant.py`,
`analyze_sweep.py`); Δscore is the full-corpus translation (+ = worse), time is Σ over the subset:

| family switched off / halved | Σ time | crudeoil_lee3_08 | Δ dev | Δ out-of-sample | verdict |
|---|---|---|---|---|---|
| all partitioners (METIS/Scotch/KaHIP, 8 gates) | −29.3 s | 1.49 → 1.15 s | +80 bip | +38 bip | too costly whole |
| EXTRA partitioner variants only (tuned/hi-trial/param/multi, 5 gates) | −10.6 s | −0.1 s | +45 bip | **+7 bip** | dev-fitted → RETIRE |
| BASE METIS/Scotch/KaHIP only | −1.6 s | 0 | +11 bip | +17 bip | keep |
| hand-rolled RCM/Sloan/ND/GGGP/MinFill | −10.6 s | −0.1 s | +0.4 bip | +0.7 bip | free → RETIRE |
| AMF alpha sweep + robust AMD envelope | −10.5 s | −0.13 s | +0.6 bip | **+124 bip** | keep |
| subtree chain entirely | −37.9 s | −0.31 s | +131 bip | +63 bip | keep |
| subtree budgets ×0.5 | −14 s | −0.12 s | +7 bip | +11 bip | take (margin) |
| exact-LNS streams ×0.5 | −6 s | −0.02 s | +3 bip | +4 bip | leave |
| pair/five/four-pivot + simplicial ×0.5 | −0.7 s | 0 | −2 bip | +1 bip | leave |
| relabel budget ×0.5 / off | −6 s / non-monotone | 0 | ≈0 | ≈0 | leave |

The extra partitioner variants are the clearest over-fit in the tree: 45 bip on dev, 7 bip out-of-sample, and
they are the single most expensive family on the crudeoil / mpbp / chp classes where they never win.

## What changed

`mod.rs` only, four structural edits on top of 0062:

1. **Partitioner cascade.** `consider` now takes the incumbent explicitly (`&mut best_flops, &mut best_perm`) so
   the pipeline can read it between candidates. `flops_before_part` is snapshotted before default METIS;
   default METIS, Scotch and KaHIP stay unconditional; `part_extra = n < 1_000 || nnz <= 8_000 || best_flops < flops_before_part`
   (after default METIS) gates tuned / hi-trial METIS and tuned Scotch, and `part_extra2` (the same test after
   default KaHIP) gates the METIS parameter variants and KaHIP multi. A first version that gated everything on
   default METIS alone lost 2x on maxcsp-langford-3-11 (n=660, a KaHIP win) and 1.3x on multiplants_stg6.
2. **`EXTRA_RELABEL_MAX_N = 6_000`, `EXTRA_RELABEL_MAX_NNZ = 50_000`** on the 0061 extra well-below relabel
   tickets (harness r6 assay p2: bit-identical outputs on all 891 rows; −0.30/−0.33 s on the two slowest
   out-of-sample rows). The unreachable 12-ticket branch becomes a constant 16.
3. **`RCM_MAX_N`, `SLOAN_MAX_N`, `ND_MAX_N`, `NDFM_MAX_N` = 1_000** (MinFill gate unchanged).
4. **Subtree budgets halved for `n >= 1_000`** (`MID_BUDGET`, `LARGE_BUDGET`, and `if n >= 1_000 { cfg.budget /= 2 }`
   after every chain-round / terminal-deep / extra-ticket budget assignment).

**Measured and rejected:** `ROBUST_MAX_NNZ = 150_000` (was 600_000). Zero rows change on the 169 slowest rows and it
saves 0.2-0.5 s per row on the dense large class (gams05 1.20 -> 0.71 s, nuclear104 1.01 -> 0.79 s,
unitcommit_200_100_1_mod_8 1.18 -> 0.98 s), but the full out-of-sample corpus shows the envelope wins above the
gate on rows the slow subset never contains: parabol_p (n=24005, nnz=188574) 0.8188 -> 1.0000, arki0005
(n=7522, nnz=254840) 0.8009 -> 0.9246, dev cont6-qq 0.8899 -> 0.9141. A per-variant probe of the five robust
options on those rows is the open lead: if one variant carries the wins, the other four can be gated.

## Result

| corpus | crown | candidate | better / worse | worst order() |
|---|---|---|---|---|
| dev | 0.843358 | 0.839063 | 33 / 55 | 1.823 s -> 1.084 s |
| out-of-sample | 0.863823 | 0.856455 | 32 / 71 | 1.574 s -> 0.951 s |

**dev movers (33 rows better, 55 worse):** nuclear10a (gt_10k, n=17493, nnz=163816): 0.9912 -> 0.7602; crudeoil_pooling_dt2 (gt_10k, n=18742, nnz=75910): 0.8372 -> 0.7418; nuclear104 (gt_10k, n=39098, nnz=257806): 0.9925 -> 0.8998; pinene200 (gt_10k, n=19995, nnz=97990): 0.9407 -> 0.8567; gasprod_sarawak81 (gt_10k, n=22536, nnz=75636): 1.0000 -> 0.9400; crudeoil_pooling_dt3 (gt_10k, n=30660, nnz=152210): 0.9488 -> 0.9002; faclay75 (gt_10k, n=272878, nnz=1379706): 1.0000 -> 0.9667; hydroenergy2 (1k_10k, n=2092, nnz=6236): 0.8543 -> 0.8263; faclay35 (gt_10k, n=26778, nnz=132380): 1.0000 -> 0.9687; faclay30 (gt_10k, n=16678, nnz=82242): 0.9988 -> 0.9729; sonet21v6 (1k_10k, n=8232, nnz=40744): 1.0000 -> 0.9839; methanol400 (gt_10k, n=23999, nnz=151728): 0.7367 -> 0.7289; methanol200 (gt_10k, n=11999, nnz=76128): 0.7364 -> 0.7296; unitcommit_200_100_1_mod_8 (gt_10k, n=146830, nnz=476332): 0.9839 -> 0.9764.

**out-of-sample movers (32 rows better, 71 worse):** pooling_sppb5stp (gt_10k, n=27255, nnz=738608): 0.9329 -> 0.6098; mpbp_04 (1k_10k, n=4734, nnz=20800): 0.9799 -> 0.9162; edgecross20-040 (1k_10k, n=9503, nnz=38518): 0.8021 -> 0.7538; faclay30h (gt_10k, n=16678, nnz=82242): 0.9988 -> 0.9729; sonet20v6 (1k_10k, n=7070, nnz=34964): 1.0000 -> 0.9863; faclay20h (1k_10k, n=4753, nnz=23700): 0.9981 -> 0.9863; sonet19v5 (1k_10k, n=6023, nnz=29758): 0.9982 -> 0.9877; sonet18v6 (1k_10k, n=5085, nnz=25096): 0.9984 -> 0.9893; arki0015 (1k_10k, n=3476, nnz=15928): 0.9696 -> 0.9615; sonet17v4 (1k_10k, n=4250, nnz=20948): 0.9981 -> 0.9899; rsyn0805m04m (1k_10k, n=4172, nnz=12180): 0.8336 -> 0.8286; pooling_sppc3stp (gt_10k, n=33387, nnz=967510): 0.9937 -> 0.9880; pooling_sppc1stp (gt_10k, n=19673, nnz=516464): 0.9837 -> 0.9786; pooling_sppa5stp (1k_10k, n=4207, nnz=73170): 0.6408 -> 0.6383.

`cargo test --release -p ssi-candidate-worker`: 50 passed, 0 failed (18 ignored probes).

## Follow-ups

- Partitioner CASCADE (run the extra variants only when default METIS beat the incumbent) measured in
  `sweep_r6b/casc*.log` — the refinement that would recover the dev value of the extras if it keeps the time.
- A per-matrix WORK LEDGER (skip late phases once the requested-ops sum crosses a cap) is the structural law that
  would bound unseen classes; needs rgreedy to report consumed ops.

## Links

- [0062](0062-reduce-then-amf-terminal.md), [best-of-portfolio](../techniques/best-of-portfolio.md)
