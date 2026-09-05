# 0001 — AMD (quotient-graph approximate minimum degree)

- **Date:** 2026-06-30
- **Score:** before ≫1.00 (identity stub) → after **0.9992** (geomean flop ratio vs AMD; lower is better)
- **Tiebreak (fill):** 0.9998
- **Status:** win (matches the baseline; the expected ceiling for an AMD-vs-AMD port)
  — **superseded, see the correction below**

> **Correction (2026-09-01).** This page is kept as the dated record of the run;
> nothing below it describes the tree as it ships today. Three things expired:
> `src/ordering/amd.rs` no longer exists (the hand-rolled port was replaced by
> the `feral-amd` crate declared in [`deps.toml`](../../deps.toml), so `order()`
> does not delegate to `amd::order`); the `SSI_TEST_SLEEP_MS` hook is gone (the
> time cap is exercised via the harness's `--test-time-cap` flag); and the dev
> corpus has been rebaselined from the 279 matrices used here to today's 300, so
> the 0.9992 / 0.9998 figures are not comparable to current scores. For where
> AMD sits in the shipped portfolio see [amd](../techniques/amd.md), and for the
> current score [index.md](../index.md). The *conclusion* of this experiment —
> no headroom in doing AMD-vs-AMD — still holds and is why the portfolio treats
> AMD as an anchor rather than a target.

## Hypothesis
Replace the identity stub with a real AMD ordering. Since the harness baseline
*is* AMD (`feral_amd::amd_order`), a faithful quotient-graph port should land at
≈1.00 — confirming the implementation is correct and giving every later
experiment (nested dissection, min-fill, refinement) a competitive starting
point to beat instead of the non-competitive identity stub.

## What changed
- New `src/ordering/amd.rs`: stdlib-only port of CSparse `cs_amd` (Amestoy,
  Davis & Duff 1996) — quotient graph (variables + elements), approximate
  external degree, mass elimination, supernode/indistinguishable-variable
  detection, aggressive element absorption, dense-node sink, assembly-tree
  postorder via iterative `tdfs`.
- `src/ordering/mod.rs`: `order()` now delegates to `amd::order`; kept the
  `SSI_TEST_SLEEP_MS` time-cap hook. Added structural unit tests (bijection,
  arrow→hub-last, tridiagonal, disjoint cliques, determinism, empty/singleton).

## Result
- 279/279 dev matrices: valid, deterministic, all under the 2 s cap.
- Geomean flop ratio **0.9992**, fill ratio **0.9998** — statistically a tie
  with feral's AMD, as expected.
- Edges the baseline slightly on a handful (best per-matrix ratios ≈0.993–0.995),
  loses by a hair on none materially. Differences are
  tie-breaking / dense-threshold details between the two AMD variants, not a
  structural advantage either way.

## Why it won / lost
It "won" only in the sense of replacing a far-worse-than-1.00 stub with a
correct AMD. There is **no headroom against AMD by doing AMD** — the small
per-matrix deltas are noise from minor heuristic differences (dense threshold,
hash bucketing, tie ordering). Real gains require a *different* algorithm family
on the structured/grid-like matrices where greedy local MD is beaten globally.

## Follow-ups
- Nested dissection on the large grid-like families is the open headroom — see
  [nested-dissection](../techniques/nested-dissection.md). Logged in open-questions.
- Consider AMD as the small-block ordering *inside* nested dissection (hybrid).

## Links
- Techniques: [amd](../techniques/amd.md), [nested-dissection](../techniques/nested-dissection.md)
- Literature: _(Amestoy-Davis-Duff 1996 note still to be written)_
- Prior: [0000-identity-baseline](0000-identity-baseline.md)
