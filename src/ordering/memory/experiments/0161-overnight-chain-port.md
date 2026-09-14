# 0161 — Porting the overnight six-promotion chain (final forms) + the tail-exchange dose curve

- **Date:** 2026-09-13
- **Score (port-only candidate, SHIPPED):** dev 0.791243 → **0.790135**
  (−1.11e-3; −1.69e-3 total vs the graded 0158 base), **54 better / 3 worse**
  vs ABCD; 70/2 vs base for the rejected X1 build. Buckets 0.8873 / 0.8372 /
  0.6815.
- **Original-improvement pass:** the ported-but-unshipped `SSI_XCHG_TAIL`
  device was dosed on our tree — real value (dose 1: **0.789948**, dose 2:
  0.789892) but the dose-1 build **FAILED the official local harness cap on
  crudeoil_lee4_09 (killed ≥ 2.0 s)**; reverted to 0 with the receipt. The
  pool arm breached locally at 2.565 s (dead). This closes the device for
  unpriced re-enablement; the lt_1k tie census (54/147 remain, mostly
  metric-inert tiny graphs per the 0200 finding) is left for a priced attempt.
- **Status:** local candidate only — NOT submitted. Hidden frontier to beat
  `5212fea` = submission `43c1ca7` at **0.840782**.
- **Source:** actual diffs; every mechanism ported at its FINAL form (Main's
  iterated-mechanism rule). Files byte-identical to `52affcb` were taken
  verbatim from `5212fea`: `rgreedy.rs` (MAX_N 12k→25k + `max_n_limit()`
  seam + `search_par_specs` pub), `window_dp.rs`, `peo_extract.rs`
  (`reverse_candidate_bounded`), `minl.rs`, `symbolic_flat.rs` (new flat
  symbolic kernels with vendor-equivalence tests). `indep_first.rs` taken from
  `5212fea` then our three graded hunks re-applied (0152 cap-2 giant band,
  0156 METIS nip16, both receipts intact). mod.rs hand-ported: PRODUCTION_
  SPAN_WINDOWS (9 widths), PRODUCTION_EXCHANGE_LEDGER = 2G, PEO_ROUNDS = 4,
  exchange window 12/4/5, class_n = 25k everywhere, terminal bank + seeded
  walk streams, L2 lift call-site, inert fanout/xchg seams. NOT ported: their
  `indep_force_*`/`indep_arb` seams (frame-divergence risk; our probes are
  production-frame), force_audit, the ~22 `symbolic_flat` call-site refactors
  in mod.rs (speed-only; the module ships for minl.rs).

## Chain mechanisms at final form

| promotion | mechanism (final form) |
|---|---|
| e07fe7a 0.841768 | `indep_first::run` incumbent arg + **L2 lift** (cap-3 set on the winning core, 400k ledger, 49/50 decisive margin, core ≥ 1k) |
| 83a8f4f 0.841666 | exchange window **8/4/3 → 12/4/5** (hidden-validated) |
| 78c434c 0.841502 | symbolic_flat kernels + **terminal bank** (skip no-op restarts 3/4, spend as 2 re-seeded walks w/ reverse-MCS PEO seed at the very end) |
| a34c109 0.841366 | ledger/PEO step (512M/4 rounds shape) |
| 52c744d 0.841011 | **MAX_N 12k → 25k** (the band extension; 2G ledger multiplier) |
| 43c1ca7 0.840782 | exchange ledger **2G** + 5 sweeps + span schedule 9 widths |
| (their 0205/iter38-46 negatives) | late watcher, 4-rung draw ladder, fanout 4-stream, cost-keyed fence, early arbitration — all dead or inert; fanout ported inert (budget 0), rest not ported |

## Layered A/B (quiet machine, deterministic COUNTS)

| tree | dev | movers |
|---|---:|---|
| ABCD `cd84b82` | 0.791243 | — |
| **P** (chain port) | **0.790135** | 54 / 3 (regressions: edgecross24-115 +6.6 % basin shift — our 0151 AmindNorm row, L2/hydro-window interaction; chimera_mgw +0.15 %; hydroenergy2 +0.25 %) |
| P + XCHG_TAIL=1 | 0.789948 | +16 rows; **harness cap FAIL on lee4_09** → reverted |

Top P movers: crudeoil_lee2_06 −3.8 %, methanol400 −3.9 %, gabriel09 −2.4 %,
chp_shorttermplan2d −2.4 %, pooling_sppa0pq −2.2 %, mpbp_15 −1.8 %,
crudeoil_lee4_10 −1.7 %, crudeoil_lee4_06 −1.4 %, lee4_09 −1.4 %.

Controls: DegDivNvDegme re-added to the retired variant list = score IDENTICAL
(their retirement holds on our tree too).

## Verification

- **125 active tests pass** (121 + 4 chain tests), 45 ignored.
- Full 300-row probe: `evidence/0161-probe-{P,FINAL}.log`.
- **Official sandboxed harness, P build: 300/300 OK** at 0.790135 / fill
  0.9232 (the X1 build failed on lee4_09 — the receipt that chose P).
- Timing (QUIET machine, valid): probe worst 1.042 s single-draw;
  **min-of-3 on the near-cap class: lee4_09 0.901 s, lee4_10 0.891 s**,
  lee4_06 0.836 s, powerflow0300p 0.733 s, gabriel09 0.730 s — ≥ 2.2x margin
  to the 2.0 s cap. Run-to-run band ~1.6x per index.md; the X1 build's 1.647 s
  draw on lee4_09 illustrates it.

## Hidden-cap reasoning

Their `5212fea` profile (everything P carries in the tail) graded 0.840782 —
cap-clearing with worst ~1.4 s on a slower pinned-4-vCPU box. Our composite
adds only our graded small-core tickets (n-banded away from the lee4_09
class) on top; min-of-3 puts the class at 0.9 s here. The X1 receipt shows the
next unit of spend on that class is fatal locally — do not add tail work
without a work-priced gate.

## Reproduction

```sh
git checkout <0161 P commit>
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
# dose screens: SSI_XCHG_TAIL={1,2} [SSI_XCHG_POOL=3]; class band: SSI_TERM_CLASS_N / SSI_MAX_N
bash scripts/local-candidate-build.sh && cargo run --release --offline --locked -- --note "0161"
```

## Links

- [0159](0159-frontier-ports-window-descent-fence-draw-kernels.md),
  [0160](0160-terminal-followup-port.md) — the earlier layers. Frontier
  sources `475be33 a26c201 7df69b9 178caa7 de6e8d3 5212fea` (+`256152b`).
