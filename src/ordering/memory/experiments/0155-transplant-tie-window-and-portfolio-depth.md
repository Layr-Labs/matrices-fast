# 0155 — Two probes at the terminal transplant and at portfolio depth

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792573** (transplant window, adopted) and
  0.792573 → **0.792513** (portfolio deepening, reverted)
- **Status:** one inert path deleted; one deepening measured and rejected

## Probe A — the transplant's near-AMD tie window is inert

### Hypothesis

`transplant_probe::refine_with_donors` admits an incumbent either because it is
already below AMD, or through a second predicate, `sparse_large_tie`:

```
(15_000..120_000).contains(&n) && (40_000..500_000).contains(&nnz)
    && nnz <= 6 * n && inc_f * 100 <= amd_flops * 101
```

Two two-sided windows. Neither bounds the pass's cost — that is already done by
`TRANSPLANT_LEDGER` and by the `3 * (n + nnz) <= TRANSPLANT_LEDGER` entry test —
so they select a population. A window has two half-space re-derivations, the one
that contains it and the one it contains; measure both.

### Result

| tree | dev | `lt_1k` | `1k_10k` | `gt_10k` |
|---|---|---|---|---|
| base | 0.792573 | 0.887516 | 0.840192 | 0.685651 |
| windows deleted, `nnz <= 6 n` kept | **0.792573** | identical | identical | identical |
| whole predicate deleted | **0.792573** | identical | identical | identical |

Opened to every sparse tie row, the predicate fires on **48 of the 300 dev
patterns** — all at an AMD ratio of exactly 1.000 — and **improves none of
them**. 46 of those 48 are outside the old window, so the window's own
population is 2 rows, and they do not improve either.

### Why it lost

A donor transplant rearranges subtrees of the incumbent's elimination tree and
accepts only a strict decrease. On a row where the incumbent *is* AMD's
ordering to within 1 %, every donor in the runner-up pool is a portfolio
ordering that already lost to it, and the subtree structure they could
contribute is the structure AMD already found. There is nothing on the other
side of the tie band.

### What changed

`src/ordering/transplant_probe.rs`: the predicate is deleted, leaving
below-AMD incumbents only. That is the re-derivation that also removes work.

## Probe B — deepening the portfolio at its cheap end

### Hypothesis

`1.portfolio` is the pipeline's highest-value stage per second, and the
relabelled multi-start's only knob is the restart count. The cheapest restarts
are on `nnz <= 20_000` patterns, where the budget law is
`(600_000 / nnz).min(48)`. If average value per second carried to the margin,
raising that budget should buy score cheaply.

### What changed

`RELABEL_LOW_WORK` 600 000 → 900 000 and `RELABEL_LOW_MAX` 48 → 64, nothing
else. Modelled over the dev corpus this is **+1 592 restarts (+15 %)** and
**+10.8 %** of the multi-start's `Σ restarts × nnz`.

### Result

| bucket | base | deepened |
|---|---|---|
| `lt_1k` | 0.887516 | **0.887520** |
| `1k_10k` | 0.840192 | **0.839989** |
| `gt_10k` | 0.685651 | 0.685651 |
| **weighted** | 0.792573 | **0.792513** |

**+0.76 relative bip for +10.8 % of the stage's work** — and `lt_1k`, where
almost all of the new restarts land, gets **worse**.

### Why it lost

Two things, and the second is the more useful.

1. **The marginal rate is nothing like the average rate.** `1.portfolio` returns
   0.867 ln-units per second averaged over the 212 rows it gains on; at the
   margin, at the cheap end, half a stage's work again buys under a bip.
2. **More candidates is not weakly better.** The multi-start is a best-of, so
   the *leader* it hands on can only improve — but the leader is the input to
   ten downstream stages whose own gain depends on which basin they are handed.
   A better stage-1 leader can end as a worse final ordering, and on `lt_1k` it
   does. Any "this can only help" argument that stops at the stage boundary is
   wrong in this pipeline.

Reverted.

## Links

- [0154](0154-monotone-relabel-restart-law.md) — the restart law both probes sit on.
- [0152](0152-stage13-retirement-isolated.md) — the same basin effect, in the
  other direction.
