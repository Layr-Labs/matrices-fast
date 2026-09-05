# 0065 - Relabelled objectives on the residual core

- **Date:** 2026-09-05
- **Base:** 0064 extra ranked-subtree round 5, dev score 0.838890
- **Score:** 0.838890 -> **0.838285** (-0.000605, 6.05 bips), fill 0.942755 -> **0.942538**
- **Status:** WIN locally; hidden validation pending

## Hypothesis
The degree-three reduction exposes a smaller exact instance of the same
elimination objective. Relabelling that core should give AMF and AMD different
deterministic basins at a lower cost than relabelling the full matrix.

## What changed
`src/ordering/mod.rs` retains the valid `CoreLift` from the existing terminal
reduction, then runs eight fixed relabelled AMF and eight fixed relabelled AMD
passes on the core. AMF cycles dense-alpha values `{0.5, 2.5, 5.0, 10.0}`;
AMD cycles four fixed aggressive/dense configurations. Each result is composed
back through the core ids and admitted only after a full-pattern strict-flop
check.

The added family is gated to residual cores with `core_n <= 4000`. The broader
all-core assay spent substantially more time on large non-winning cores while
the restricted gate retained nearly all of its score gain.

## Result
The full 300-matrix trusted run passes. Relative to 0064, the bucket flop
geomeans are:

- `lt_1k`: 0.890341 -> 0.890341
- `1k_10k`: 0.868271 -> 0.867281
- `gt_10k`: 0.778266 -> 0.777498

Seven matrices improve and none regress: `edgecross10-090`,
`rsyn0830m04m`, `ringpack_20_3`, `rsyn0820m04m`, `gabriel09`,
`gasprod_sarawak16`, and `ringpack_20_2`. Fill improves to 0.942538.

The timing probe reports a worst local `order()` call of 0.841 s, with `gams05`
as the slowest row. All 50 active tests pass. The synthetic probe and the
core-relabel assay also pass.

## Why it won
The residual core removes the fixed low-degree fringe, changing the numbering-
sensitive decisions made by AMF and AMD. The objective remains exact under the
fixed prefix, so the best candidate can be compared directly with the completed
pipeline. The `core_n <= 4000` gate excludes expensive non-winning large cores
while preserving the seven observed movers.

## Follow-ups
- Validate the gain on an independent corpus or hidden submission before
  widening the core-size gate.
- Do not use the all-core version without a separate timing margin study; it
  added up to 0.7 s on non-winning large rows in isolation.

## Links
- Prior reduction: [0062](0062-reduce-then-amf-terminal.md)
- Prior timing constraints: [0063](0063-time-margin-by-structure.md)
- Preceding subtree pass: [0064](0064-extra-ranked-subtree-round5.md)
