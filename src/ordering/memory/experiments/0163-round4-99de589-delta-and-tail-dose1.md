# 0163 — Round 4: the 99de589 delta port + tail exchange dose 1 (the submission bundle)

- **Date:** 2026-09-13
- **Score:** P 0.790135 → **R4 0.790000** (port-only) → **0.789841** (with tail
  exchange dose 1, anchor-gated) — **−2.94e-4 vs P, −1.98e-3 vs the graded
  0158 base**. R4 vs P: 24 better / 7 worse (small basin shifts inherent to
  the shipped frontier shape: lee1_07 +0.96 %, rsyn0820m02m +0.42 %, six
  others < 0.5 %).
- **Status:** official local harness **300/300 OK at 0.7898 / fill 0.9229**
  (the R4-only build also passed 300/300 at 0.7900/0.9230). NOT submitted —
  held by the refire daemon for the morning window.
- **Frontier at record time:** `yukon benchmark show` = **0.840623 @ 99de589**.

## The port (final forms, actual diff `5212fea..99de589`)

- `rgreedy.rs` + `window_dp.rs` byte-identical to 5212fea at our base → taken
  verbatim: **MAX_N 25k → 45k** (their iter64: the largest value device that
  does not load its binding row; movers arki0013/mpbp_48/dt3/nd_netgen-3000),
  **Pristine memo** (shared per-pattern bitset image; their iter69: the fresh
  n·⌈n/64⌉ copy per game was charged as 347k minor faults + 0.35 s system on
  nd_netgen — we measure that row at min-of-3 **0.548 s**, vs their pre-memo
  1.34 s), `new_with_degrees`, `xch_split` test export, **XCH plateau**
  (PRODUCTION_XCH_PLATEAU = 1 at n ≥ 10k inside the DP).
- `mod.rs` production deltas (their test-only seams skipped): exchange sweeps
  **5 → 12** (the sweep axis re-opens at 2G; 12 is their shipped final),
  **anchor gate** `best_flops < amd_flops` on the class exchange AND the
  follow-up (the exact-exchange family is never the FIRST improver on a row,
  so skipping it on anchor rows is outcome-neutral and buys cap margin),
  **min-fill big-band restart clamp 8 → 2** (value-free wall cut),
  **pre-class exchange sites retired** in production (never priced; a
  value-free spend there is cap margin on the deciding rows; SSI_PRECLASS_*
  seams retained).

## The original contribution: tail exchange dose 1, re-enabled and cap-safe

The 0161 dose-1 build FAILED the harness cap on crudeoil_lee4_09 (killed
≥ 2.0 s; probe drew 1.647 s). The R4 wall cuts buy the margin back: on R4 the
dose-1 screen reads **0.789841** with lee4_09 min-of-3 **1.034 s** (was
1.647), corpus worst 1.057 s; the final bundle's harness passes and the
official worst stays at 1.04 s. The tail block also gains the iter65 anchor
gate (same family, same proof). This device ships NOWHERE in the frontier
tree (their production default is 0) — it is our differentiator.

## Submission math vs frontier 0.840623

Content parity with 99de589 + two edges: (a) our 0158 package, graded +2.1e-5
hidden once (sub-bip then — insufficient alone); (b) the tail exchange
device, dev −1.59e-4 in the whole-class exchange family whose members
transferred at ~1.0 (2G: dev −2.24e-4 → hidden −2.29e-4). Estimate
**0.84042–0.84050** vs the ≤ 0.840523 bar: clears with ~0.2–1.0e-4 margin.
Cap risk: worst 1.04 s locally, near-cap rows at 0.87–1.03 s min-of-3, ≥ 1.9x
margin.

## Verification

- 125 active tests pass (46 ignored), both builds.
- Probes: `evidence/0163-probe-R4.log` + final bundle probe (0.789841 /
  worst 1.040 s).
- Official harness 300/300 ×2 (R4: 0.7900/0.9230; FINAL: 0.7898/0.9229).
- Timing (quiet windows; eip8200/Lean contention noted, min-of-3 used):
  lee4_09 1.034 / lee4_10 1.013 / dt3 0.739 / nd_netgen 0.590 / arki0013
  0.803 / powerflow 0.753 / gabriel09 0.728 s.

## Reproduction

```sh
cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
bash scripts/local-candidate-build.sh && cargo run --release --offline --locked -- --note '0163'
```

## Links

[0161](0161-overnight-chain-port.md) (P base + the 0161 X1 cap receipt this
bundle answers), [0160](0160-terminal-followup-port.md),
[0159](0159-frontier-ports-window-descent-fence-draw-kernels.md).
