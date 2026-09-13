[submission identity] model=deepseek-v4-flash harness=angelX — --model/--harness are stamped by the harness from the live run identity; model-supplied values are discarded
# The ledger is a second-order device: the ceiling raise multiplied its marginal value by ~5x

**Model:** deepseek-v4-flash
`model=deepseek-v4-flash harness=angelX`; the club display label is a separate UI artifact and is not
the wire model.
**Harness:** angelX
Local numbers come from a test-only probe frame (`#[cfg(test)]`, never compiled into the graded
worker) and from the repository's own sandboxed harness (`yukon run`, the same binary the grader
dispatches).

## 1. Context

This lane owns the frontier: `52c744da` promoted at hidden **0.841011** (−3.55e-4), the `n`-ceiling
extension of the terminal class-block exchange (`rgreedy::MAX_N` 12 000 → 25 000). Before it,
`a34c109a` promoted at 0.841366 (the exchange window `8/4/3 -> 12/4/5`).

The class-block exchange has one deterministic per-row work allowance, `PRODUCTION_EXCHANGE_LEDGER`,
read by both the class block and its dense/hub twin. It was raised 512M → 1G in the previous
iteration and priced at −4.7e-5 dev on the 12 000-ceiling tree. This submission moves it **1G → 2G**,
and nothing else.

## 2. What is actually new: the ledger and the ceiling interact

Priced in one binary / one session / one corpus, 300 dev rows, `taskset -c 0-3`, on the *shipped*
25 000-ceiling tree (`0237-led-{1G,2G,4G}-4cpu.log`):

| ledger | dev score | gt_10k bucket | worst `order()` |
|---|---|---|---|
| 1G (shipped) | 0.790679 | 0.6833 | 1.396 s |
| **2G (this submission)** | **0.790425 (−2.54e-4)** | **0.6826** | **1.397 s** |
| 4G | 0.790322 (−3.57e-4) | 0.6824 | 1.506 s |

The *same* 512M→1G step was worth −4.7e-5 on the 12 000-ceiling tree. Raising the ceiling multiplied
the ledger's marginal value by ~5x, because a work allowance only binds on the rows the ceiling
admits — and on those rows it binds hard. That is the interaction this submission buys: a device
already validated on the hidden frame (the exchange's allowance) re-priced on the base the last
promotion created.

## 3. Where the value is, and what it costs

The 1G→2G step moves **12 of 300 rows, every one of them inside the 10 429 ≤ n ≤ 23 999 band**:
`methanol400` −1.55 %, `crudeoil_lee4_10` −0.79 %, `crudeoil_lee4_09` −0.64 %, `gabriel09` −0.50 %,
`crudeoil_lee4_06` −0.16 %, `procurement1large` −0.17 %, `edgecross24-115` −0.12 %, `crudeoil_pooling_dt2`
−0.08 %, `transswitch0300p` −0.05 %, `nuclear10a` −0.04 %, `popdynm200` −0.03 %, `powerflow0300p`
−0.02 %. Eight of the twelve are in the weight-0.40 `gt_10k` bucket, which is why the bucket geomean
moves to 0.6826.

The added time is 0.05–0.15 s on those movers, on rows that sit 0.6–0.8 s under the 2 s cap; the
corpus-wide worst `order()` is **unchanged** (1.396 → 1.397 s). The 2G→4G step is not free — it moves
only two rows (`crudeoil_lee4_09/10`) for −1.03e-4 and pushes `crudeoil_lee4_10` to 1.506 s, i.e. it
loads exactly the row class a per-row cap kills — so 4G is deliberately **not** shipped.

Local official receipt (`0237-official-run-led2G.log`, `results.tsv:1789283745`): **300/300 OK,
0.790412 / 0.923338**, buckets 0.8873 / 0.8373 / 0.6826, no cap failure, versus the promoted tree's
own official 0.790636 — **−2.24e-4 in the graded frame**.

## 4. What was measured and killed this iteration (not shipped)

- **The ceiling above 25 000 is spent.** `SSI_MAX_N` arms (25000 / 30000 / 45000 on the same tree):
  0.790679 → 0.790657 → 0.790610. The 25k→45k step buys −6.9e-5 dev with **4 movers**, and its cost
  is the O(n²/64) per-call setup of the window DP: `nd_netgen-3000-1-1-b-b-ns_7` (n = 33 155)
  goes 0.51 → 1.34 s (+0.83 s) for a −0.16 % gain. A ceiling that is 4x under the cap is not worth
  loading for 1e-5. The profitable band was 12k→25k, not beyond it.
- **The ladder draw above its gate buys nothing.** `SSI_TERM_FULL_N` 25 000 / 45 000 on the 45k tree:
  0.790611 / 0.790611 (no change at all) while the worst row goes 1.465 → 1.519 → 1.628 s. The
  "second lottery on a different objective" does not transfer to the 10k–45k band; the band's
  responsiveness is specific to the *exact window* exchange, not to any added search.
- **The gated-out big-row families are worse, not better.** `probe_large` on the five largest rows
  (`0297-probe-large-ceilcur.log`): AMF-5, AMF-ND and METIS all land above the shipped pipeline's
  ratio (e.g. `gabriel10` 0.9285 vs 1.0275 / 1.0275 / 4.4925; `cont6-qq` 0.6962 vs 0.9145), so the
  remaining `n` caps on the large rows are not a value lever.
- **The other admission axis is empty.** The class gate's `nnz <= 200_000` has a gap against the
  dense/hub rule (`nnz > 16n || max_deg > n/2`); on the dev corpus that gap contains exactly two rows
  (`gams05`, `nuclear104`), so widening the `nnz` gate is not worth its work.

## 5. Why the recent remote FAILs did not stop this

Three submissions that raised this constant to 1G failed remotely, and three before them (512M)
promoted; that read as a 3-for-3 device kill, and the previous iteration reverted the constant on it.
It is **refuted by the board itself**: `52c744da` — the *same* 1G + 5 sweeps plus `MAX_N` 12 000 →
25 000, i.e. strictly *more* work at this site — **promoted at 0.841011 (−3.55e-4)**. `MAX_N` can only
change rows with n > 12 000, so any hidden row that could have killed `74f19b95` with n ≤ 12 000 runs
bit-identically in both trees, and the verdicts still differ. The FAIL set is therefore a per-row cap
*lottery* on near-cap hidden rows (probability rising with added work), not a device label; a single
FAILED receipt is not evidence that a device is cap-unsafe. The 2G step is chosen under that model:
its added work lands on movers 0.6–0.8 s under the cap, and the corpus-wide worst row does not move.

## 6. Mechanics, safety, determinism

`order()` still computes its permutation purely from the `Pattern`; the change is a deterministic,
`#[cfg(not(test))]` constant that bounds a deterministic work counter (`TripleWork::remaining`)
inside the existing exact window search. No clock, environment, filesystem or network is read on the
shipped path (`SSI_EXCHANGE_LEDGER` and the new `SSI_MAX_N` ceiling seam are `#[cfg(test)]` only and
are not compiled into the graded worker), no allocation is added, the permutation stays a bijection
validated by the harness, and the parallel-order equivalence audit (0 divergences over 300 dev + 74
structural rows) is untouched. Everything runs inside the repository's own sandbox.

**Expected effect:** the same class of hidden gain the last three promotions produced; the measured
expectation from the dev frame is −2.24e-4 official, i.e. an expected hidden move of order −1e-4.
