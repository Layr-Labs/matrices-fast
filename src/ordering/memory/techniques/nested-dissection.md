# Nested dissection (ND)

## What it is
A divide-and-conquer ordering: find a small vertex separator that splits the
graph into two balanced halves, order the separator **last**, and recurse on
each half. Numbering the separator last confines fill to the separator block,
which is asymptotically optimal on grid-like / well-structured problems
(George 1973; METIS-class multilevel partitioners, Karypis & Kumar 1998).

## How it works (enough to implement)
- Partition: find a balanced edge/vertex separator (multilevel: coarsen →
  partition coarse graph → refine, e.g. Kernighan–Lin / Fiduccia–Mattheyses).
- Order each part by recursion; place separator vertices after both parts.
- At small subgraphs, switch to a local heuristic (minimum degree) — this is
  the standard **ND + MD hybrid**.

## Cost profile vs the cap
**This is the live risk.** The demo ND+AMD hybrid in this knowledge base wins on
grid-like structure but **breaches the 2 s cap on dense matrices**: its exact-MD
inner loop is O(deg²) per pivot, which explodes on dense KKT rows / hub nodes
even at modest n (and the corpus reaches n ≈ 340k). Any ND path must:
- switch its base-case ordering to a **quotient-graph MD** (near-linear, like
  [AMD](amd.md)), not exact MD; and
- gate the expensive partitioning by **density (nnz / max-degree)**, not just n.

## Where it wins / loses
- **Wins:** larger, structured / grid-like families — the main open headroom,
  since AMD already handles the dense KKTs well.
- **Loses / dangerous:** dense, hub-heavy patterns — separators are large and
  the inner loop blows the cap. Detect and fall back to AMD-style here.

## Status in `src/ordering/`
**In the portfolio, via libraries and hand-rolled variants** — no longer a lead.

- `feral_metis` (multilevel ND, AMD base case) — three work levels plus, since
  [0002](../experiments/0002-measured-gates-metis-kahip.md), five *shape*
  variants (`max_imbalance`, `nd_to_amd_switch`, an extra seed).
- `feral_scotch` (default + more separator trials), `feral_kahip` (Fast, plus
  seed 2 and Eco mode).
- Hand-rolled `nd_order` (BFS median-level separator) and `ndfm_order` (greedy
  graph-growing + minimum-side separator), both with hard work budgets and
  iterative task stacks.

> **Correction (2026-07-26).** This page previously said ND was "lead only" and
> that the demo hybrid "breaches the cap on dense matrices". Both are stale. The
> cap-breach risk was specific to that demo's exact-MD O(deg^2)/pivot base case;
> the library partitioners all use a near-linear AMD base case and stay inside
> their gates. What IS true is that KaHIP is the single most expensive candidate
> measured (up to 0.65 s at n~22k), which is why it has the tightest gate.

**The honest result:** ND is *not* the headroom this knowledge base expected it
to be. A 12-variant sweep across METIS/Scotch/KaHIP improved only 7 of 260
matrices ([0002](../experiments/0002-measured-gates-metis-kahip.md)). It does win
big where it wins — `mpbp_34` 0.567->0.452, `mpbp_35` 0.588->0.469 (both KaHIP,
both in the heaviest bucket) — but on most of this corpus's KKT patterns AMD
still beats every separator.

## Links
- Literature: _(add George 1973 and Karypis–Kumar 1998 notes)_
- Compare: [amd.md](amd.md)
