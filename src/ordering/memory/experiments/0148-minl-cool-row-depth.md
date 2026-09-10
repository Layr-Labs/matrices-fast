# 0148 — MINL cool-row depth (null) + chained-transplant verification

- **Date:** 2026-09-09/10
- **Score:** tip `62654a5` dev **0.792300 → 0.792247 (−0.53 bips)** for
  chained-1 + MINL-deep combined; MINL-deep net ≈ +0.01 (null-to-negative) →
  REVERTED. Kept tree = chained-1 only (expect 0.792246, sibling-measured).
- **Status:** MINL-depth closed; chained shape held from submitting
  (known-rejected hidden EV ≈ −0.3); submit bar set: no submit below −2 dev
  bips by a new mechanism.
- **Files:** `mod.rs` phase-14 chained block (+20/−0, kept);
  `minl.rs` parameterization + call-site gate (REVERTED, null).

## MINL cool-row depth (CLOSED)

Rationale: MINL introduction bought −5.6 dev bips at 40M ops/16 rounds
(0095); rows where the budget cuts the descent short have headroom.
Threaded `(ops_budget, max_rounds)` through `minl_candidates` (single caller)
and doubled both (80M/24) gated on input unit ≤ 60k — all hot rows
(lee4_06 65k and up) keep the production bound; rows that already complete
are unaffected by construction.

Result: combined run 0.792247 vs sibling's chained-only 0.792246 on their
box → MINL-deep net +0.01 bips (one micro row-ratio, deterministic). No
breadth, no upside: cool rows evidently already complete within 40M/16, and
the rows that don't are warm ones the gate (correctly) excludes. The deep
budget has no one to help. Do not retry MINL depth without evidence that a
specific cool row exhausts 40M/16 (no such row is known).

## Chained-transplant verification (this box, load ~11/8)

Full `yukon run` PASSED (load had dropped from ~90): 0.792247 with buckets
lt_1k 0.887442 (+0.05 vs tip — the stg5 cascade tax), 1k_10k 0.839654,
gt_10k 0.685295. Movers reproduce 87e7f7b. The up-to-2-chained variant from
0147 was NOT re-run; the one-round cap stands.

## Submit discipline (new rule)

Deterministic code reproduces deterministic hidden scores, so resubmitting a
shape with measured hidden outcome (87e7f7b: −0.31, rejected) reproduces a
rejection. No submit unless dev shows ≥ −2 bips by a NEW mechanism
(translation 0.2–0.6x needs that for the ~1-bip bar; subtree-depth
mechanisms have translated ≥1x before and are the exception to size up).

## Box discipline (confirmed)

Load 93/8 → four straight full runs each SIGKILLed a DIFFERENT row
(lee4_06, lee4_09, lee4_10, rsyn0810m04m). At that load, local 2 s-cap
verdicts are pure noise; scores stay deterministic and trustworthy. A Lean
build + agent swarm owned the box. Sibling hidden validations (87e7f7b
completed Benchmark) are the timing ground truth, not this box. Check
`uptime` before trusting any local pass/fail; do not chase load-ghosts.

## Next (in order)

1. Subtree selective depth by the 0053 method on CURRENT rounds: attribute
   which round/budget binds on cool score-dense rows, deepen behind a
   structural gate. Rationale: subtree-depth translated 1.18–1.34x
   (0050/0051) — the only family with measured amplification.
2. Residual-core multi-K replacement phases (open question).
3. Nothing additive on hot rows, ever again, without a measured cut funding
   it first.

[Index](../index.md) | [Log](../log.md)
