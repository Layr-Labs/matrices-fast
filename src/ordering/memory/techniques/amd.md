# AMD (Approximate Minimum Degree)

## What it is
The competition's baseline ordering, anchored at score **1.00**. A greedy
fill-reducing heuristic: repeatedly eliminate the node of (approximately)
minimum degree, using a quotient-graph representation so degree updates stay
near-linear instead of rescanning the whole graph.

## How it works (enough to implement)
- Represent the eliminated graph as a quotient graph (variables + "elements"),
  so cliques formed by elimination are stored compactly rather than explicitly.
- Each step: pick the min approximate-degree variable, eliminate it, merge its
  neighborhood into a new element, update approximate degrees of affected
  variables. The "approximate" degree bound is what makes it cheap and is the
  key trick vs exact minimum degree.
- Full derivation: see `literature/` once an AMD paper note is written
  (Amestoy, Davis & Duff 1996 — in the repo README references).

## Cost profile vs the cap
Near-linear in practice — the quotient graph keeps per-pivot work bounded. This
is why AMD survives the dense KKT rows that break an exact-MD inner loop. Treat
AMD's cost profile as the bar any expensive candidate path must not exceed.

## Where it wins / loses
- **Wins:** dense KKT / hub-node patterns — already strong here, so headroom
  against AMD on these is thin.
- **Loses (relatively):** large grid-like / structured problems, where global
  nested dissection beats a purely greedy local heuristic.

## Status in `src/ordering/`
**It is the anchor of the portfolio.** `order()` calls
`feral_amd::amd_order(&core)` with library-default options — bit-for-bit the
grader's baseline — and takes it as the floor before considering anything else.
That is what guarantees `ratio <= 1.0` on every matrix; see
[best-of-portfolio](best-of-portfolio.md).

The portfolio also carries **~10 further AMD variants** as ordinary candidates,
because AMD's two options change the ordering materially:
`aggressive` (element absorption) x `dense_alpha` in {-1, 1, 2, 5, 10, 16}, where
`dense_alpha < 0` disables dense-row detection entirely so AMD never defers the
dense KKT coupling rows. Those are gated to `n < 150k && nnz < 130k`.

> **Correction (2026-07-26).** An earlier version of this page said
> "`src/ordering/amd.rs` is a stdlib-only quotient-graph AMD port ... 0.9992".
> That file no longer exists — the hand-rolled port was replaced by the
> `feral-amd` crate declared in `deps.toml`, and the corpus has been rebaselined
> since (300 matrices), so 0.9992 is expired. Current score:
> [index.md](../index.md).

The old conclusion still holds and is worth keeping: **there is no headroom in
doing AMD-vs-AMD.** What the corpus shows now is stronger than that — 122 of 300
matrices tie AMD at exactly 1.000 even against METIS, Scotch, KaHIP, RCM, Sloan,
AMF and two hand-rolled ND variants. On this corpus AMD is not a soft baseline.

## Links
- Literature: _(add Amestoy-Davis-Duff 1996 note)_
- Compare: [nested-dissection.md](nested-dissection.md)
