# 0091 — Exact minimum fill on the residual cores, and how to price a bounded search

Base: `996e8d6` (dev 0.826558219775327, hidden 0.859087).
Result: dev **0.824729222140265**, **18.28998** absolute bips, 6 wins / **0**
losses / 294 unchanged. Fill tiebreak 0.937028 → 0.936976. 69 tests pass.

## The gap this exploits

The core portfolio runs eight AMF alpha tickets, AMD, and up to four extra
reduction depths on the residual core `reduce` builds. **Every one of them is a
minimum-degree variant**, and at the extra depths they are ranked by a proxy
(`ndiv + nms_ldl`) rather than by the exact objective. Minimum *fill* is a
different objective, and the shipped `minfill_order` never sees these cores: its
gate is on the raw `(n, nnz)` window, not on core size, so it is excluded from
exactly the cores where a second objective could pay.

An unbounded screen over the reachable band measures **26.76 dev bips on 12
rows**. On the captures production's own exact MinFill already reaches, a
perfect search wins **nothing** — all the value is at the extra depths, at
`cn > 1000`, and outside the raw `n`/`nnz` window.

## What shipped

One pass inside `order_core`, after the AMF/AMD passes have picked and
`refine_core` has run, at **every reduction depth**:

```text
ledger = CORE_MINFILL_LEDGER (16_000_000)          // once per original row
if 8 <= cn <= CORE_MINFILL_MAX_CN (4_000)
   && core_nnz <= CORE_MINFILL_MAX_CORE_NNZ (30_000)
   && ledger > 0
    (p, charged) = minfill_core_order(core, ledger)
    ledger -= charged
    accept p iff flops_of(core, p) < flops_of(core, portfolio_pick)
```

Gated on **core size only** — never on the raw `(n, nnz)` window, never on
identity. Acceptance is a strict decrease of the exact core objective, so the
pass cannot lower the portfolio's own pick. The reduction it consumes is paid
on every row inside `REDUCE_MIN_N`/`REDUCE_MAX_NNZ` regardless.

`minfill_core_order` is a bitset minimum-deficiency search with **index-only
ties**: `deficiency(v) = C(deg,2) − Σ_{u∈N(v)} popcount(N(u) ∧ N(v)) / 2`, on
`⌈cn/64⌉`-word rows, deficiencies cached and recomputed only on
`N(v) ∪ N(N(v))` after each pivot (exact: eliminating `v` changes no other
vertex's neighbourhood and adds edges only inside `N(v)`).

**The ledger is per ROW, not per capture.** A row has up to five in-gate cores,
and a per-capture allowance lets it spend five of them. The per-row form gives
up 0.54 bips (18.29 against 18.83) and cuts the rows over +50 ms from 48 to 3
in the surrogate's own accounting. Both dominant winners complete inside
13 915 020 and 8 986 024 words on their single in-gate capture, so **14M words
is the floor** and 16M is the measured knee — 32M scores identically for twice
the search.

### Three implementation facts that are worth more than the score

1. **A one-word summary per bitset row pays for itself.** The gate admits
   `cn <= 4000` (so `w` up to 63) at `core_nnz <= 30 000` (so mean degree ≤ 15).
   A full `0..w` scan therefore spends `w / |occupied words|` more than the
   neighbourhood occupies. Marking which words of a row may be nonzero — and
   walking only those in the deficiency evaluation, the pivot union, the degree
   recomputation and the `N(N(p))` union — is exact, because AND-ing or OR-ing
   a zero word changes nothing.
2. **Pack the selection key.** `(deficiency << 32) | index` in a `u64` makes
   the unsigned word order *identical* to the `(minimum deficiency, smallest
   index)` tie rule, so pivot selection is a min-reduce over a compact array of
   live vertices with an `O(1)` `swap_remove`, instead of an `n`-wide scan over
   a `live` flag and a separate `defic` array. A core deficiency is at most
   `C(cn−1,2) < 2³²` at this gate, so the packing is lossless.
3. **Charge an unpayable sweep in closed form and skip it.** The initial
   deficiency sweep costs exactly `Σ_v (deg(v)·w + 1)`, known before any word
   is touched. A call handed a nearly-spent ledger used to run it anyway —
   `2 · core_nnz · w` words per capture for a result the pivot loop discards
   immediately — and that term, not the ledger, dominated the worst-case
   per-row bound. Charging it analytically is output-identical (degrees are
   already final, so the degree-ordered tail such a call returns is unchanged)
   and **drops the per-row word bound from 36.2 M to 17.26 M**.

Changing an implementation without re-earning its score needs an oracle: the
pre-rewrite version is retained under `cfg(test)`, and permutation **and**
ledger charge are asserted equal on all **418 admitted production cores** plus
1000+ synthetic shapes across the whole `w` range, complete graphs, duplicate
CSC entries and budgets that exhaust before, during and after the search. The
charge per evaluation stays the full-scan `|N(v)| · w + 1` precisely so the
search truncates at the same point.

## The measurement method, which is the transferable part

**Do not difference two 300-row `order()` timings to read a change worth 1.5 s.**
On a two-core pinned box, an arm compared against *itself* read 20–60 rows over
+25 ms and 10–20 over +50 ms, with individual untouched rows spanning 228 ms
across five trials. Against a tail gate that allows 20 and 8, the pair says
nothing.

Instead, **time the changed call at its own call site.** A `#[cfg(test)]` hook
after the production call re-runs the search on the identical `(core, budget)`
pair, times it, and asserts the outputs match. One ~3 min corpus pass yields
the per-row distribution with no contribution from the other 133 s of `order()`,
and the charged-word half of the reading is fully deterministic — the ledger's
own denomination, identical on every host.

| statistic | measured at the call site | from a surrogate reimplementation | gate allows |
|---|---:|---:|---:|
| rows touched | 193 | 193 | — |
| S1 rows > +25 ms | **5** | 24 | ≤ 20 |
| S2 rows > +50 ms | **0** | 3 | ≤ 8 |
| S3 worst row | **35.85 ms** | 107.1 ms | ≤ 100 ms |

**A surrogate transfers where it does not truncate and over-prices where it
does.** The same surrogate predicted 72/48/179.5 against a measured 81/47/167.2
for a per-capture allowance — where almost nothing truncates — and then
over-priced the per-row form by 4.8× on S1, because 64 of its captures truncate
and the probe finishes an elimination production only sorts. Its named worst
row, `glider400` at 107.1 ms, costs **6.16 ms** in production.

The same correction hits the **rate**. On the shipped path the pass runs at
**436–679 charged words/µs**; the surrogate's 710 is optimistic by 1.6×. The
one term a word ledger does not bound — the `O(cn²)` selection scan — needs its
own rate, measured by handing the search a perfect matching so every vertex has
deficiency 0 and the scan is isolated: **0.43–0.59 ns per pair**.

## Why the corpus maximum is not the exposure, and what is

`core_nnz <= 30_000` is the half of the gate that keeps this off the big rows,
and it is measured rather than argued: **all ten of the slowest dev rows take
zero admitted calls**, including the corpus maximum `faclay75`, whose only
capture has `core_nnz = 410 700`. So do `crudeoil_lee4_10`, `gabriel10`,
`acopf_case9241pegase_qcqp`, `nuclear104`, `gams05`, `arki0013`,
`unitcommit_200_100_1_mod_8` and `ringpack_30_2`. In a 5+5 interleaved tail
escalation the candidate's corpus maximum came in **44.7 ms below** the base's.

The real exposure is the `multiplants_*` family — the `lt_1k` route toward the
cap at `n ≈ 800`, 1.30 s locally — which takes 24–27 ms, roughly 375 ms below
the corpus maximum.

**And the honest one: this pass spends its ledger on 193 of 300 rows, win or
lose, for 6 winners.** That is the same shape as an earlier bundle in this tree
that gained 9.10 dev bips, put its corpus maximum 59 ms *below* the tip's, and
still failed hidden validation — a stage that spends unconditionally on every
admitted row rather than only where an earlier strict gain has paid for that
row. Hidden failure is opaque here, so that is a hypothesis and not an
established cause, but it is the leading one and it applies. The next form to
test is the same objective **conditioned** so it fires on a fraction of the
admitted rows: the portfolio's own passes disagree on a hard core and agree on
an easy one, and that disagreement is already computed.

## The other standing caveat

The gain is concentrated. Dropping the `ringpack` source prefix leaves **1.607**
of the 18.29 bips; the sorted-name corpus halves give 0.787 / 35.789. Whether
the hidden corpus holds dense residual cores where minimum fill beats minimum
degree is exactly what the dev corpus cannot say.

## Reproduce

```sh
bash scripts/local-candidate-build.sh && cargo run --release   # 0.824729
cargo test --release -- --nocapture minfill_core_order_matches_reference
cargo test --release -- --ignored --nocapture probe_core_minfill_cost
cargo test --release -- --ignored --nocapture minfill_core_order_scan_rate
cargo test --release -- --ignored --nocapture \
  minfill_core_order_matches_reference_at_gate_corner
```
