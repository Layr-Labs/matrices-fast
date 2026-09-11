# 0160 — Re-derive `peo_alt_danger` as the cone three sibling sites already use

- **Date:** 2026-09-11
- **Score:** 0.792573 → **0.792583** (+0.126 relative bip; 2 rows worse, 0 better)
- **Status:** compliance re-derivation, graded

## Hypothesis

`mod.rs:3881` gates the alternate-seed PEO chains on

```rust
let peo_alt_danger = (3_000..8_000).contains(&n) && nnz >= 9_000;
```

and the three comment lines above it name the corpus rows each edge was drawn
around (`lee1_07` for the band, `mpbp_15` for the lower edge's history,
`chimera` for what must stay inside). A two-sided `n` window drawn around named
corpus families is instance special-casing in a structural costume, and the
rules say inherited code is not exempt.

The tree already contains the compliant form of **this exact predicate, three
times**, all from the same iteration family, all upward-closed cones, all
removing work when true:

| line | text | effect when true |
|---|---|---|
| `mod.rs:1777` | `let danger = n >= 1_800 && nnz >= 9_000;` | 2 METIS densify switches instead of 4 |
| `mod.rs:2664` | `let danger_timing = n >= 1_800 && nnz >= 9_000;` | 4-ticket exact-search floor instead of 6 or 10 |
| `mod.rs:3445` | `&& !(n >= 1_800 && nnz >= 9_000)` | skip residual-core exact entirely |

So the re-derivation writes itself.

## What changed

`src/ordering/mod.rs`, one predicate:

```rust
let peo_alt_danger = n >= 1_800 && nnz >= 9_000;
```

## Work domination, checked pointwise

The cone strictly contains the band: `3_000 <= n < 8_000` implies `n >= 1_800`,
and the `nnz` half is identical. So `danger_new >= danger_old` at every
`(n, nnz)`, every pattern that skips the chains today still skips them, and no
pattern gains chain work. Checked over all 300 dev rows: **0 violations**.

Of the **280** dev rows eligible for the stage (`16 <= n <= PEO_ALT_MAX_N`,
`n + nnz < PEO_ALT_LEDGER`), **36** skip today and **92** skip under the cone.
The intermediate `n >= 3_000 && nnz >= 9_000`, which deletes only the
identity-derived *upper* edge, reaches 80.

*(These counts are in `Pattern::nnz()` units — off-diagonal. The corpus JSONL's
`nnz` field is `n` larger and screening against it gives different, wrong
answers.)*

## Result

| | base | this |
|---|---:|---:|
| dev score | 0.792573 | **0.792583** |
| fill tiebreak | 0.924518 | 0.924518 |
| worst `order()`, 16 vCPU probe | 1.320 s | **1.171 s** |
| rows over 1.0 s | 19 | **10** |

**Two rows move, both worse, and nothing else changes:** `mpbp_15` +0.275 %,
`maxcsp-ehi-85-297-71` +0.157 %.

All four rows the stage's own comment credits as its beneficiaries — `mpbp_34`
(n = 11 556), `mpbp_35` (n = 11 120), `arki0013` (n = 44 909), `gabriel09`
(n = 21 688) — are inside the newly-skipped set, and **none of them moves**. The
stage's documented wins no longer exist on this tree; only its cost does, on 56
further rows.

## Why it won / lost

It is a compliance change, priced. The 0.126 bip is the two rows where the
chains still earn something, and the rest of the stage's reach is pure cost —
which is the same conclusion the full retirement reached
([0152](0152-stage13-retirement-isolated.md), 0.164 bip over 4 rows) at
three times the blast radius.

## Caveats

**This is a partial stage-13 retirement, and stage 13's handoff to the terminal
stages is the most cap-hostile edge in this tree.** Retiring the stage outright
died at 94 s ([0152](0152-stage13-retirement-isolated.md)); writing its score
back into `best_flops` — the opposite direction, and dev-invisible — died at 99 s
and again at 100 s ([0157](0157-stage13-best-flops-writeback.md)). Both of those
change what `14.transplant` / `15.minl` / `15.peel` are handed, and so does this.
The large local time reduction above is **not** evidence of cap safety: the tree
with the largest local time reduction ever measured here is the one that
SIGKILLed at 94 s.

## Links

- [0152](0152-stage13-retirement-isolated.md) — full retirement: dev loss and cap death
- [0157](0157-stage13-best-flops-writeback.md) — the opposite edit to the same handoff
