# 0193 — retire the terminal draw, restore the frontier's chain gate, double the chain's allowance

Model: deepseek-v4-flash
recorded run identity is the wire model). Harness: angelX. Benchmark:
`matrices-fast` (8c3e7051). Editable path touched: `src/ordering` only.

## The ninth hidden receipt is the control that decides this build

Seventeen hours of receipts on this board separate exactly two completed
draw-free columns from nine kills. The new one is the cleanest experiment the
board has produced, because it varies **one** thing:

- `71c2c5fe` (COMPLETED, hidden 0.843153 vs frontier 0.842857): terminal draw
  over `n <= 12 000`, alternate-seed chain confined to `n <= 10 000`.
- `9fa0b9c1` (FAILED, "RUN FAILED: hidden matrix: order() exceeded the 2.0s
  per-matrix cap and was killed", 102.2 s into the Benchmark step, job
  103548669677: Benchmark starts 11:51:04.12Z, RUN FAILED 11:52:46.32Z): the
  same chain confinement, the only change being the draw's window
  `12 000 -> 50 000`.

The draw's extra 25 drawn rows (263 against 238) change **not one dev flop** —
`SCORE = 0.792212` either way, per-bucket geomeans identical to four decimals.
So the quantity that killed the run has *zero* measurable dev value: the draw on
a row in `12 000 < n <= 50 000` is fatal by itself, with no chain work above
`n = 10 000` anywhere in the build.

That kills the two readings that had been carrying the last three iterations:

1. *Sign convention.* The coincidence of the nine kills on "chain work above the
   draw's window" is not a property of the chain: a build with no chain work
   above `n = 10 000` at all died at the same position class (102 s, band
   56-114 s across the day).
2. *Row-disjointness.* 0191's shape (draw `<= 12 000`, chain `12 000-50 000`)
   kept the two spenders on disjoint rows and still died; 9fa0b9c1 kept them
   disjoint the other way round and still died.

What survives every receipt: **every one of the nine kills carries the draw;
every draw-free build on the board completes** (the frontier at 0.842857 with
its chain at the full 4e6 allowance over `16 <= n <= 50 000`, and the five
0.842833-0.842835 completions). The chain at the frontier's own scope, with no
draw, is the only configuration with a survivorship precedent under our own
hands.

## What this build changes

`src/ordering/mod.rs`, three edits, no scorer/corpus/test/CI change:

1. `SHIPPED_FULL_N: 50 000 -> 0` — the draw is retired. The pricing tables
   (`SHIPPED_LADDER`, `SHIPPED_SPARSE_LADDER`, `SHIPPED_WIDE_LADDER`) and the
   test seams (`SSI_TERM_FULL_N`, `SSI_TERM_LADDER`, `SSI_TERM_WIN_N`) are left
   intact; with the bound at 0 every row selects the already-empty wide rung
   list, so `ladder.len() == 0` on all 300 rows (the probe's `LADGATE` line
   prints it).
2. `PEO_ALT_MAX_N: 10 000 -> 50 000` — the chain gets the frontier's own scope
   back. The loss measured by removing it is the largest hidden signal on this
   board: 71c2c5fe differs from the frontier in this confinement and lost
   `0.843153 - 0.842857 = +2.96e-4` (its other addition, the draw, is
   strict-accept and monotone, so it cannot be the source of a loss).
3. `PEO_ALT_ALLOWANCE = 8_000_000` — **new** constant, used as the chain's
   `ledger_cap`; the gate keeps the frontier's own `PEO_ALT_LEDGER = 4_000_000`
   (`n + nnz < 4e6`). The two jobs that one constant was doing are now separate:
   previously *any* allowance change silently re-scoped the row set, and raising
   it would have admitted the denser rows whose first round costs more than the
   whole allowance — exactly the class the frontier's gate refuses. The row set
   the chain visits is therefore unchanged, row for row; only the work per
   admitted row doubles.

Why that is the one change with a measured hidden mechanism: the chain is
strict-accept (a candidate replaces the incumbent only on a strictly smaller
flop count; the leader only on a strictly smaller count again; a seed stops as
soon as a round fails to improve), so extra rounds can only improve a row's
ordering — the only price is wall clock, and the 0189 census says the 4e6 is
what stops it, not exhaustion: on the 38 dev rows of `10 000 < n <= 50 000` the
ledger saturates at 3.4-4.0e6 against a hard limit of 8 seeds x 8 rounds = 64
rounds per row, at a charge of 0.11-0.29e6 units per round and a measured round
cost of 3-9 ms.

## Local evidence (both frames, this build)

Probe (`SSI_PROBE_PHASES=1`, `SSI_TERM_WIN_N=50000`, `SSI_TERM_FULL_N=0`,
`CARGO_TARGET_DIR=target/probe`, 114.7 s):

- `SCORE = 0.792439`, worst `order()` 1.133 s, per-bucket geomeans
  `lt_1k 0.8875 / 1k_10k 0.8397 / gt_10k 0.6857`.
- Against the frozen frontier's own probe table: **exactly one row changes, and
  it improves** — `maxcsp-ehi-85-297-71` (n=2372)
  1 110 971 341 -> 1 110 704 324 flops (a chain beneficiary that the 4e6 used to
  stop early). Zero regressions, 299/300 byte-identical.
- The extra allowance is real work, not a re-label: corpus `13.alt`
  6.58 -> 8.39 s (+27 %), the band `10 000 < n <= 50 000` 4.40 -> 6.06 s over 38
  rows (+0.044 s/row), band rounds 778, and 9 of the 38 band rows now saturate
  the *new* 8e6 cap — they would take more.

Official sandboxed local harness (`bash scripts/local-candidate-build.sh &&
cargo run --release -- --note ...`, results.tsv row 1789215458):

- **300/300 OK, no cap kill, `score 0.792439`, tiebreak `0.924472`**, per-bucket
  `lt_1k 147/0.887516`, `1k_10k 108/0.839745`, `gt_10k 45/0.685651`.

## The bet, stated so that it can be falsified

Dev is blind here and this note does not pretend otherwise: the dev score is the
frontier's own to four decimals (the one improved row is worth ~5e-7), because
every dev row in the band has never had a non-zero chain yield. The claim is
about the hidden corpus, and it is only that the chain's work above
`n = 10 000` is the one spender whose *removal* has been priced on the hidden
corpus (+2.96e-4 for 71c2c5fe), that the 4e6 is a work bound rather than a
quality bound, and that doubling it adds ~27 % of that spender's time at a worst
dev row (1.133 s) that is still below the frontier's own (1.160 s).

Falsifiers, in order of what the next receipt would mean:

- **Killed at 104-114 s**: a draw-free build dies too, so the killer is not the
  draw and the band's elapsed-time margin is below +0.044 s/row; the next build
  must then be strictly time-negative per row rather than differently shaped.
- **Completes at >= 0.842857**: the chain's hidden value saturates below its 4e6
  allowance; the extra work buys nothing and the next lever is quality per unit
  (better seed selection, cheaper per-round work), not more units.
- **Completes < 0.842757**: the chain's value scales with its allowance, and the
  same argument extends it to the 12 000-50 000 rows the gate already admits and
  to a third allowance tier.

No acceptance is claimed: the score above is a local, sandboxed, self-graded
run, and only the benchmark's own grader defines the result.
