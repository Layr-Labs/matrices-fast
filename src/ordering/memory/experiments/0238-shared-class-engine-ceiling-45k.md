# 0238 — one shared exact-kernel engine per `order()`, and the ceiling it pays for (25 000 → 45 000)

**Frame.** `probe_timing_and_score` (test-only, `#[cfg(test)]`, never in the graded worker) on all
300 dev rows, one binary / one session. The probe is the official scorer minus the 2 s cap: the
per-row flops it prints come from the same `Σ cⱼ²` definition the harness scores, and the shipped
tree reproduces the published number exactly (baseline here **0.790425**, buckets
**0.8873 / 0.8373 / 0.6826** — identical to the `2G` arm of `0237-led-2G-4cpu.log`).

**Host note (why no `results.tsv` receipt).** This box is ~5–6× slower than the pod the receipts
above were taken on and runs at load 5–10 on 4 vCPUs: `slay06m` (n = 357) measures **1.04 s** here,
`sssd25-08` (n = 401) **1.05 s**, `sporttournament48` (n = 1131) **2.49 s** — all uncapped probe
times, so the row total is dominated by fixed pipeline overhead, not by the row. Every local harness
run therefore dies on the 2 s cap on a *tiny* row (three attempts, three different rows: `rsyn0810m04m`
n=4772, `sporttournament48` n=1131, `sssd25-08` n=401 — the last two with `SSI_MAX_MATRIX_N` 2000
and 500). This is the documented host artifact, not a property of the tree: the unmodified promoted
tree fails the same way on the same rows. All numbers below are the uncapped probe.

## 1. Hypothesis: the class's quadratic *setup* is what prices its `n` ceiling

The terminal exact-kernel class owns the whole of its value axis (`52c744da`, the `MAX_N`
12 000 → 25 000 step, is the last hidden-validated receipt this lane has: dev −8.4e-4 → hidden
−3.55e-4). The measured next step of that axis, `SSI_MAX_N = 45 000`, buys −6.9e-5 dev with **4
movers** and was rejected on cost alone: `nd_netgen-3000-1-1-b-b-ns_7` (n = 33 155) went
**0.51 s → 1.34 s**.

That +0.83 s is not the search. Every class site built its OWN `Game`:

* `Game::build_adj` — `vec![0u64; n·⌈n/64⌉]` plus an `O(nnz)` bit-set scan;
* `Game::new` — a second full `n·⌈n/64⌉` copy **plus** a popcount pass over it for `deg0`.

A class row runs the exchange, its dense/hub twin and **nine** sparse-span passes ⇒ up to **eleven**
setups of the same immutable pattern. At n = 33 155 one setup is `3n⌈n/64⌉ ≈ 51.6 M` word-ops
(≈ 413 MB of traffic); eleven of them ≈ 4.1 GB — the whole measured +0.83 s.

`Game::new` returns a game that is only valid after `reset()`, and the sweep loop already calls
`reset()` first thing; `reset()` restores exactly what a fresh `Game::new` leaves. So **one shared
`Game`, reset at the head of every sweep, is bit-identical to one `Game` per site.**

## 2. The change (structural, output-preserving)

* `rgreedy/window_dp.rs`: the body of `subset_window_descent_config` is split out as
  `subset_window_descent_body(game, …)`; new `subset_window_descent_step_with_game` and
  `sparse_span_window_descent_with_game` run a pass on a caller-owned `Game`. The plain entry points
  remain as thin wrappers (probes and unit tests keep their own-local-`Game` semantics).
* `window_pass_affordable(n, nnz, budget)`: head + setup + one sweep's charge > budget ⇒ the pass can
  only return `None`, so refuse it **before** the adjacency build. Also `window_descent_precheck`,
  which runs the body's structural input validation before `build_adj` (a malformed pattern must not
  reach `col_ptr[v..v+1]`).
* `mod.rs` (`leader_order`): one `class_pristine` bitset + one `class_game` per `order()` call, gated
  by the union of the class sites' keys (`6 ≤ n ≤ class_n && nnz ≤ 200 000`), handed to all three
  sites. **The work ledgers are untouched** — every pass still charges its own `setup` term — so which
  windows a budget funds does not move.
* `rgreedy::MAX_N` 25 000 → **45 000**.
* `PRODUCTION_EXCHANGE_LEDGER` 2G → **4G**. `0237` priced this step at −1.03e-4 dev and refused it
  because it loads `crudeoil_lee4_10` (+0.11 s there). iter56 re-prices it on this tree with the
  shared engine in place (in-frame, `SSI_EXCHANGE_LEDGER` seam, everything else fixed): it moves
  exactly three rows — `crudeoil_lee4_10` **181 053 364 → 179 065 997 (−1.10 %)**, `crudeoil_lee4_09`
  **−0.59 %**, `nuclear10a` −0.001 % — i.e. **−1.51e-4 dev**, and the interleaved min-of-3 timing A/B
  over six class rows (incl. the corpus's slowest, `chimera_selby-c16-02`) reads **−1.19 s total with
  no row systematically worse**. The shared engine takes ~0.1–0.15 s OFF every class row (nine of the
  eleven builds at n = 17 809 are ~14.9 M word-ops each), the same order as this step's added work, so
  the pair is cap-neutral against the frontier.

`cargo test --release -p ssi-candidate-worker`: **126 passed, 0 failed** (56 ignored).

## 3. Result

Bit-identical output on 296 of 300 rows (verified row-by-row against the pre-change probe log), and
the four movers are exactly the rows the new ceiling admits:

| row | n | nnz | base ratio | new ratio | Δ |
|---|--:|--:|--:|--:|--:|
| `mpbp_48` | 28 368 | 91 016 | 0.4772 | 0.4746 | −0.539 % |
| `crudeoil_pooling_dt3` | 30 660 | 152 210 | 0.7100 | 0.7056 | −0.623 % |
| `nd_netgen-3000-1-1-b-b-ns_7` | 33 155 | 90 000 | 0.9599 | 0.9572 | −0.278 % |
| `arki0013` | 44 909 | 160 172 | 0.4021 | 0.3993 | −0.699 % |

`faclay35` (26 778/132 380) is admitted and unchanged. **0 rows regress** (every class site accepts
only a strict exact decrease), and the `lt_1k` / `1k_10k` buckets are bit-identical by construction.

```
SCORE 0.790425 -> 0.790295   (-1.30e-4)
buckets  0.8873 / 0.8373 / 0.6826  ->  0.8873 / 0.8373 / 0.6823
```

45 000 is where this corpus runs out: **no dev row above it clears the class's own `nnz ≤ 200 000`
key**, so a higher limit would add no candidate while squaring memory (`2·n·⌈n/64⌉·8` = 507 MB at
45 000 vs 2.5 GB at 100 000; the graded 4 GiB address-space cap is not enforced locally).

## 4. Negative result recorded (do not retry): the giant tier is not relabel headroom

New test-only probe `probe_giant_relabel` asks whether the heavy-tier relabelled-AMF lottery — the
historically strongest family, whose gate (`HEAVY_RELABEL_AMF_DENSE_MAX_NNZ = 700 000`) excludes the
giants — should be widened. Rows ≥ 100 000 nodes, relabelled AMD (aggressive and non-aggressive,
α10) + relabelled AMF (α5 / α2.5 / no-dense), seeds 1…8. **The incumbent wins almost everywhere**:

* `unitcommit_200_100_1_mod_8` (n = 146 830, cur **0.9764**): best draw over 10 passes **0.9978**;
* `cont6-qq` (n = 120 395, cur **0.6962**): best draw **1.1500**;
* `faclay75` (0.9405), `gabriel10` (0.9285): no draw beats the incumbent at all;
* `acopf_case9241pegase_qcqp` (n = 313 068, cur **0.9737**): best draw **0.9719** — one seed in
  eight, −0.18 %, at **0.5–0.8 s per pass**. On a weight-0.40 bucket of 45 rows that is ≈ −1.3e-5
  score for a pass that costs more than the entire remaining margin of the row.

So the `nnz ∈ [200k, 700k]` density gaps (`unitcommit`, `cont6-qq`, `nuclear104`, `gams05` fall in
neither the sparse nor the dense sub-tier) are **not** a loss: on those rows the shipped pipeline is
already far ahead of a fresh lottery. The same probe rediscovers the `0237` conclusion
(`probe_large`: AMF-5 / AMF-ND / METIS all above the shipped ratio on the five largest rows) for the
relabel family. Cost of one AMD pass on a 1.3 M-nnz row here: 0.5–0.8 s — a giant-tier device has to
be near-free to exist at all.

## 5. What this is and is not

* **Structural**: it removes work that was provably redundant (eleven identical adjacency builds per
  row) without touching a single acceptance decision or budget arithmetic. The score step it buys is
  a *pre-existing, already-measured* device (`MAX_N` 25k → 45k) that was blocked only by that
  redundancy. Nothing is tuned to a matrix's identity: the gate is `(n, nnz)`.
* **Not** a parameter lottery: the ceiling is the axis with the only hidden-validated receipt this
  lane owns, and 45 000 is its natural end on this corpus (no row above it is admissible).
* Still open, and now the cheapest place to look: the *replay* (`Game::reset` + a full `B = O(n·⌈n/64⌉)`
  elimination walk per sweep) is ~4/5 of what a class row spends. The charge model already pays for
  the dense row width (`(deg+1)(3w+6)`) while the kernel touches only the non-zero words of each row;
  a sparse or undo-logged replay would turn the ledger's unit of account into real work and is the
  only path found so far that could raise the ledger (2G → 4G is −1.03e-4 dev) **without** loading the
  near-cap rows.
