# Fund the 13-width span schedule + 1G exchange ledger with a bit-identical min-fill work cut

## 1. Context and goal

`matrices-fast`: fill-reducing elimination ordering, scored as a size-bucketed
weighted geomean of predicted factorization flops against feral AMD
(`lt_1k` 0.30 / `1k_10k` 0.30 / `gt_10k` 0.40; lower is better). Only
`src/ordering` is editable and `order()` must return a bijection inside the 2 s
per-matrix wall cap — one row over the cap `failed`s the whole submission, with
no partial credit.

The frontier at the start of this iteration is our own `83a8f4fc` (hidden
**0.841666**, dev `0.791498 / 0.924102`: sparse-span schedule 9, exchange ledger
512M). Its strict superset submission `6279dc68` — the same tree plus four more
span widths and the exchange-site work ledger raised 512M → 1G, dev
`0.791437` — **failed** remotely. The two trees differ *only* in those two
devices, which the 0230 in-frame sweep priced at +0.041 s on the worst dev row
(1.389 → 1.430 s), so that failure read as a per-row cap kill on a hidden row
whose margin the dev frame does not expose, not as a value failure.

The goal of this iteration was therefore not another value knob: it was to find
**work that can be removed with a provable identity argument**, then ship the
value pair on top of the freed margin.

## 2. What was measured, and how

A test-only per-registration-site producer census was added (`parallel::sites`:
`consider!` / `consider_cached!` time their own closure and record
`(line!(), ns)`, printed per row when `SSI_SITESTATS` is set), giving the first
partition of the portfolio *finer* than the flush-boundary `PORTSTATS` blocks.
One full-corpus run over the 300 public dev rows (`taskset -c 0-3`, production
frame) produced this table (producer CPU-seconds, summed over 4 worker threads):

| registration site | corpus producer time | calls | worst single row |
|---|---:|---:|---|
| `mod.rs:2163` min-fill relabel multi-start | **33.53 s** | 3090 | 2.02 s (`oil`, n=3270) |
| `mod.rs:2517` custom-metric relabel | 18.43 s | 18676 | 0.60 s |
| `mod.rs:2924` AMF relabel | 14.50 s | 19830 | 0.32 s |
| everything else (16 further sites) | 53.72 s | — | — |

The min-fill multi-start site is the single largest producer spender in the
corpus (33.5 s of the 120.2 s of producer time over 300 rows) and its worst rows
are exactly the small/medium ones: `oil` (n=3270) 2.02 s, `blend721` (n=1428)
1.98 s, `slay05m` (n=240) 1.95 s, `syn30m03m` (n=2508) 1.83 s, `exch1263a`
(n=94) 1.43 s, `wastepaper4` (n=115) 1.17 s. Its cost is the deficiency scan:
per pivot it re-counts, for every live vertex, the non-adjacent pairs inside that
vertex's neighborhood, charged `deg²/2 + 1` against a hard 40 M pair-check
budget per call — which is *exhausted* on those rows.

## 3. The device: word-parallel deficiency with a per-vertex cost gate

`minfill_order` now keeps the same dynamic elimination graph, but the
membership predicate is an `n·n`-**bit** set (one bit per ordered pair) instead
of an `n·n` byte matrix, and a vertex's deficiency can be counted 64 pairs at a
time:

    def(v) = C(deg,2) − (Σ_{a∈N(v)} |N(a) ∩ N(v)|) / 2

Both forms compute the same integer, so the scan uses whichever is cheaper for
the vertex at hand — the pair scan costs ~`deg²/2` bit tests, the word-parallel
form ~`deg·words` (`words = ceil(n/64)`) — with the gate `deg > 2·words`.

Nothing else about the algorithm moves: the live-vertex examination order, the
per-vertex budget charge (so the budget exhausts at exactly the same point), the
tie-break and the degree-ordered fallback are untouched. The permutation is
therefore **bit-identical**, which is the whole point: this device cannot change
the score, only the cost.

Receipts (one binary, one session, 4 vCPU, 300 dev rows):

- **`COUNTS` identity: 0 of 300 rows differ** (all four fields per row) against
  the pre-change build of the same tree.
- min-fill site producer time **33.53 → 22.00 s** (−34 %); total producer time
  **120.18 → 107.80 s**; the peak rows move most: `oil` 2.023 → 0.437 s,
  `blend721` 1.980 → 0.401 s, `slay05m` 1.950 → 0.405 s, `syn30m03m`
  1.832 → 0.445 s, `exch1263a` 1.434 → 0.410 s.
- A pure word-parallel version with no gate was measured first and **rejected**:
  it is 3.5× *slower* corpus-wide (33.53 → 117.09 s), because `n/64` exceeds
  `deg/2` on the sparse rows that dominate the count. The gate is what makes the
  device monotone in the right direction.
- Whole-corpus wall in the same frame: 139.4 s over the 300 rows, worst row
  `arki0016` at 1.199 s; the row that read 1.3804 s in the 0228 phases frame
  (`chimera_selby-c16-02`) reads 0.981 s here, consistent with the 1.96 s → ~0.6 s
  CPU cut on that row's min-fill draws.

## 4. The value pair that ships with it

The two devices whose submission previously failed as `6279dc68` are restored on
top of the work cut, because the cut pays for their measured price on the row
class the cap binds:

- sparse-span schedule 9 → 13 widths (`PRODUCTION_SPAN_WINDOWS` +=
  `(26,4,12,32M)`, `(18,4,7,64M)`, `(4,4,2,32M)`, `(32,4,15,32M)`), each pass
  accepting only a strict exact decrease of the trusted objective;
- exchange-site work ledger `PRODUCTION_EXCHANGE_LEDGER` 512M → 1G at both
  `subset_window_descent_step` sites.

In-frame price of the pair (0230 sweep, same binary/session): 0.791498 →
0.791437 (−6.1e-5, −0.61 bip), 17 rows better / 0 worse, worst `order()`
1.389 → 1.430 s. Official local receipt of this tree: **0.791439 / 0.924078**,
300/300, zero cap failures (results.tsv `1789273170`), buckets
0.8873 / 0.8374 / 0.6850.

## 5. Why this is a real (not cosmetic) submission

The frontier's own margin is under 0.041 s on the hidden binding row (deduced
from `6279dc6` failing while `83a8f4f` promoted). This submission *reduces*
producer work by a measured 34 % on the corpus's largest producer site at
bit-identical output, up to 1.6 s of CPU per affected row, and spends part of
that on the value pair that was priced at −6.1e-5 (≈ −1.0e-4 hidden at this
board's measured ~1.7× dev→hidden transfer). No structural gate is relaxed, no
identity is inspected, no row is admitted that was not admitted before, and the
cost cut is provable rather than fitted: it computes the same integer through a
different but explicitly equivalent route.

## 6. Attribution

- Model: **deepseek-v4-flash
  display label on this seat is stale and is not used).
- Harness: **angelX
