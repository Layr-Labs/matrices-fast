# 0260 — exact-content memo of the class block's pristine bitset (and the `nnz` key reverted)

Model: **deepseek-v4-flash
display label `deepseek-v4-pro` is stale and is *not* the attribution).
Harness: **angelX
(`scripts/local-candidate-build.sh && cargo run --release`), plus in-frame A/B in
one binary.

## What changed

Two hunks, both inside `src/ordering` (the only editable path):

1. `src/ordering/rgreedy.rs` + `src/ordering/rgreedy/window_dp.rs` — a new
   **exact-content memo of the pristine adjacency image**. `Game::build_adj` is a
   pure function of `(n, col_ptr, row_idx)`, but the terminal class block is
   entered **12–14 times per row** on the large-`n` rows, and each entry rebuilt
   the same `n·⌈n/64⌉` bitset from scratch (one zeroed allocation plus one
   scattered read-modify-write per off-diagonal nonzero) and then re-derived the
   degree vector by a full popcount pass. The memo keeps at most two
   `Rc<Pristine>` images per thread, keyed on the **full CSR content compared
   element-wise** (never on a pointer, never on a hash), and hands back the same
   bits, so a hit is bit-identical to a rebuild: same `Game`, same sweeps, same
   ordering. Producing path is always the memo (no seam); a test-only
   `SSI_NO_PRISTINE_MEMO` prices the rebuild arm in the same binary. Peak memory
   does **not** rise — the memo replaces a per-call owned bitset with a shared
   one, so the live set per `Game` is unchanged and an `n = 25 000` row holds
   ≤ 2 × 78 MB of cache instead of a fresh 78 MB per entry.
2. `src/ordering/mod.rs` — the class block's `nnz` admission key is reverted
   `300_000 → 200_000` (the value the promoted frontier ships). Measured below:
   the raise armed exactly one dev row for −1.4e-5 and cost that row +0.44 s.

Neither hunk changes any *decision*: every adoption in the class block is still a
strict flop decrease against the same incumbent, and the memo is a pure function
memo. No matrix identity, no lookup table, no fingerprinting; the memo key is the
row's own CSR content.

## Why — the device was chosen from a measurement, not from a knob

This lane has just lost **three consecutive remote bats** (`9440dedb` 4 G + 6
sweeps, `edd49e95` 4 G + 5 sweeps, `75117ca9` 4 G + 5 + `nnz ≤ 300 000`) while
the only clean pass (`43c1ca7d`, 2 G + 5 sweeps, hidden **0.840782**) is the
*smaller* work budget. So the binding question is not "which knob" but "can any
wall be removed without moving the ordering". This submission answers that with
the first *free* (output-identical) time device found in this lane.

**Per-call phase attribution of the class block** (new instrument,
`SSI_XCH_TIME`, one 4-vCPU session, 300 rows, 3 501 valid exchange calls over 259
distinct CSR keys = 93 % of calls are repeats of an identical key):

| phase | corpus wall | share |
|---|---|---|
| window refine (the charged search itself) | 24.4 s | 80.3 % |
| `Game::new` (incl. the `n·w` working copy + popcount pass) | 3.1 s | 10.3 % |
| `build_adj` (zeroed bitset + scattered writes) | 1.9 s | 6.2 % |
| `Game::reset()` (per sweep, full-bitset memcpy + bucket rebuild) | 1.0 s | 3.4 % |
| prefix eliminations | 0.0 s | 0.0 % |

Consequences that killed two standing hypotheses and produced the shipped one:
`Game::reset()` is **3.4 %** of the block, so an undo-log reset (the "cheaper
unit of search" candidate) can buy at most ~0.014 s on the binding row — killed
by measurement. The repeats are real: the same CSR key is entered 12–14 times per
row, so the *rebuild* is avoidable while the *search* is not.

**In-frame A/B, one binary, one session, `taskset -c 0-3`, graded-closest frame
(`SSI_MARK_NOSCORE=1`), two order-reversed pairs:**

| arm | score | worst `order()` |
|---|---|---|
| memo ON (pair 1 / pair 2) | 0.790308 / 0.790308 | 1.448 s / 1.461 s |
| memo OFF (pair 1 / pair 2) | 0.790308 / 0.790308 | 1.486 s / 1.487 s |

**Every one of the 300 dev ratios is identical in both pairs** (`0 of 300` ratios
differ by more than 1e-12), i.e. the memo is semantically neutral, as claimed.
Reproducible per-row savings on the rows that enter the block 12–14 times
(delta ON−OFF, pair1 / pair2):

```
nd_netgen-2000-3-4-b-a-ns_7  n=22074   -0.258 / -0.257   (1.05 -> 0.79 s)
ringpack_30_2                n=17999   -0.041 / -0.248
emfl100_5_5                  n=21925   -0.269 / -0.182
gasprod_sarawak81            n=22536   -0.255 / -0.181
pinene200                    n=19995   -0.236 / -0.177
supplychainr1_053050         n=16640   -0.159 / -0.142
faclay30                     n=16678   -0.158 / -0.131
crudeoil_lee4_10 (binding)   n=17809   -0.038 / -0.026
```

Corpus `order()` wall: 148.5 → 147.8 s (pair 1) and 148.6 → 147.2 s (pair 2).
Mid-size rows swing ±0.1 s between runs in both directions (e.g. `arki0016`
+0.150 in pair 1, −0.078 in pair 2), so only the eight hit rows above carry the
claim; they reproduce with the sign and roughly the magnitude in both pairs.

## The `nnz` key revert — why 300 000 is gone

The 2 G → 4 G ledger step was priced row by row in the same frame (identical
output rows, `0260-ledger-step-diff.txt`): **137 rows move, 3 improve** —
`crudeoil_lee4_10` −0.0067, `crudeoil_lee4_09` −0.0036, `gams05` −0.0006 (that
is the whole −1.1e-4 of the ledger's value) — while the other 134 rows pay
+2.9 s of corpus wall **with their ratios unchanged**, led by `crudeoil_lee4_10`
+0.188 s, `arki0016` +0.167 s, `gams05` +0.166 s, `crudeoil_lee4_09` +0.156 s.
The `nnz ≤ 300 000` key is the same trade with no upside: it armed exactly one
dev row (`gams05`, nnz 252 910) for −1.4e-5 and cost it +0.44 s (0.895 → 1.331 s
measured here). On a cap-limited board that is 0.14 bip of value bought with a
new near-cap row class, so it is reverted; the ledger step is kept because the
2 G → 4 G axis is the one device of this lane with a measured transfer of ≈ 1.02
(the 1 G → 2 G step, hidden −2.29e-4 vs dev −2.24e-4).

## Measured result (real sandboxed benchmark)

```
bash scripts/local-candidate-build.sh && cargo run --release
→ 300/300 OK, score 0.790309 / fill 0.923267
  buckets lt_1k 0.887274  |  1k_10k 0.837322  |  gt_10k 0.682326
  worst row order() (probe frame, memo on) 1.448-1.461 s
```

That is **bit-identical to the same tree's own previous official receipt**
(`0.790309 / 0.923267`, `results.tsv:1789288143`, submission `edd49e95`), which
is exactly what a semantically neutral device must do — and it is the first time
this lane has a *free* wall reduction in hand: the 4 G + 5 sweep configuration
that failed remotely now runs the same ordering with 0.13–0.26 s less wall on the
eight rows that enter the class block most often, and ~0.03 s less on the binding
row. Against the promoted frontier tree's own official 0.790412 the value delta
is **−1.03e-4 in the graded frame**.

## Cost and risk (stated in full)

* The memo cannot change any row's ratio — verified, not assumed: 0 of 300 dev
  ratios differ in two order-reversed A/B pairs, and the official receipt is
  identical to the same tree without the memo at 6 significant digits.
* The memo's saving on the *binding* dev row is only −0.026…−0.038 s. It does not
  pay back the ledger step's own +0.188 s on that row. The rows it pays back
  most are the 12–14-entry rows (n ≈ 16 k–22 k), which are also the rows where
  the last three bats added their seconds.
* If the hidden failing row is a *single-entry*, fill-heavy row of the
  `gams05`/`crudeoil_lee4_*` shape, this submission will die the way `edd49e95`
  did; if it is one of the multi-entry rows, the memo's −0.18…−0.26 s is the
  difference. This lane cannot yet see the hidden row class, and says so.
* Cache residency is bounded: 2 images per thread, LRU, each freed on the third
  distinct key; the key copies are `Vec<usize>` of the CSR (≤ 2 MB on dev).

## Reproduce

```
cd <repo>
CARGO_BUILD_JOBS=2 bash -lc 'bash scripts/local-candidate-build.sh && cargo run --release'
# in-frame A/B, one binary, one session (order-reversed pairs):
taskset -c 0-3 env SSI_MARK_NOSCORE=1 bash target/probe-sandbox.sh run
taskset -c 0-3 env SSI_MARK_NOSCORE=1 SSI_NO_PRISTINE_MEMO=1 bash target/probe-sandbox.sh run
# phase attribution of the class block (test-only instrument, never in a release path):
taskset -c 0-3 env SSI_XCH_TIME=1 SSI_MARK_NOSCORE=1 bash target/probe-sandbox.sh run
```

Evidence in `src/ordering/memory/evidence/`: `0260-xchkey-4cpu.log` (call keys),
`0260-pristine-memo-{on,off}-4cpu.log` + `0260-pristine-memo-diff.txt` (pair 1),
`0260-pair-{on,off}-4cpu.log` + `0260-pristine-memo-pair2.txt` (pair 2),
`0260-ledger2G-memo-on-4cpu.log` + `0260-ledger-step-diff.txt` (ledger row
attribution), `0260-official-run-memo.log` + `results.tsv:1789291655` (the real
sandboxed run); ledger entry in `src/ordering/memory/log.md` (iter60).
