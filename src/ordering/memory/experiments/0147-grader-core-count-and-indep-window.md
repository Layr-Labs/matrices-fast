# 0147: measure the cap at the grader's CORE COUNT, and the `indep_first` window priced

Base: `7c12f0a` (descendant of promoted frontier `62654a5`), dev **0.792452**,
16-vCPU worst `order()` **1.303 s**.

Two results, one of which changes how every timing decision in this tree should
be made.

## 1. The headline: a 16-vCPU wall clock is the WRONG instrument for the cap

`7c12f0a` was submitted unmodified as a deliberate two-way measurement. It
**FAILED** (submission `96e4593`) — despite reading **1.303 s** worst and 13
rows over 1.0 s on a 16-vCPU box, i.e. *safer on both statistics* than the
frontier `62654a5` (1.479 s, 17 rows) that had passed days earlier.

The comparative rule "at or below a revision known to have passed" therefore
**does not hold** on 16-vCPU readings. The reason is `PAR_MAX_THREADS = 4`
against a ~2-vCPU grader: on a 16-vCPU box each of the 4 workers gets its own
core, so wall clock ≈ critical path; on the grader they share ~2 cores, so wall
clock ≈ total work / 2. **How much a row inflates depends on how parallel it
is**, and that varies by more than 1.6× across the tail.

Emulate the grader directly instead — pin the probe to two cores:

```sh
SSI_PROBE_ONLY="<rows>" SSI_PROBE_REPEAT=1 taskset -c 0,1 \
  cargo test --release -p ssi-candidate-worker --offline --locked -- \
  --ignored --nocapture --test-threads=1 probe_timing_and_score
```

### Measured inflation, frontier `62654a5` (the tree that PASSED)

| row | 16 vCPU | 2 vCPU | inflation |
|---|---:|---:|---:|
| `crudeoil_lee4_10` | 1.479 | **1.939** | 1.31× |
| `crudeoil_lee4_09` | 1.364 | **1.868** | 1.37× |
| `arki0016` | ~1.11 | **1.863** | **1.68×** |
| `crudeoil_lee4_06` | 1.266 | 1.732 | 1.37× |
| `arki0013` | 1.192 | 1.615 | 1.35× |
| `gams05` | 1.187 | 1.598 | 1.35× |
| `ringpack_30_2` | 1.187 | 1.560 | 1.31× |
| `nuclear104` | 1.455 | 1.560 | 1.07× |
| `mpbp_48` | 1.240 | 1.515 | 1.22× |
| `pooling_sppc3pq` | 1.193 | 1.251 | 1.05× |
| `gabriel10` | 1.190 | 1.215 | 1.02× |
| `acopf_case9241pegase_qcqp` | 1.206 | 1.069 | 0.89× |
| `faclay75` | ~1.02 | 1.033 | 1.01× |

`7c12f0a` (the tree that FAILED) on the same instrument: worst **1.841 s**
(`crudeoil_lee4_10`), then lee4_09 1.693, gams05 1.595, lee4_06 1.538,
ringpack_30_2 1.409, nuclear104 1.388, arki0013 1.351, mpbp_48 1.316; `arki0016`
dropped below 1.10 s (stage-13 retirement helped it most).

### Three consequences

1. **The frontier passed at 1.939 s against a 2.0 s SIGKILL.** Our 1.841 s tree
   failed. At these levels pass/fail is **grader noise, not margin** — which is
   why `jonathan308` shows ~9 `failed` for each pass, and why the public list is
   mostly `failed`. Nobody near 1.8 s is submitting; they are gambling.
2. **The 16-vCPU ranking of the tail is wrong, and it is wrong in the expensive
   direction.** The giants (`gabriel10`, `faclay75`, `acopf_case9241pegase_qcqp`)
   look dangerous locally at ~1.0–1.2 s and are in fact the **safest rows on the
   grader** (inflation 0.89–1.02×: they are memory-bound and barely parallel).
   The real cap risk is the mid-size, well-parallelised band — `arki0016`
   inflates **1.68×**, from a locally unremarkable ~1.11 s to 1.863 s.
3. **Therefore "save time on the giants" is close to worthless for cap safety,
   and "save time on `arki0016`/lee/`gams05`" is what buys a passing
   submission.** Note `9.reduce`'s single worst waste — 0.252 s — sits on
   `arki0016`, the most inflated row in the corpus.

To pass reliably the 2-vCPU worst row needs to reach ~1.2–1.4 s, which is
roughly a **16-vCPU worst of 0.85–1.00 s** — consistent with, and now the
explanation for, the empirical "~1.05 s local" bar.

## 2. The `indep_first` `n`-window was worth 0.03 bip, not 1.7

`indep_first.rs` routed `n ∈ 1800..=2500` into `run_sequential_180`, a complete
older copy of the function — an identity-keyed window under `RULES.md` and the
last one in the tree. Diffing the two paths, the 180 path had exactly two
capabilities `run` had dropped: an **AMF α=5** pass (`cn ≤ 4_000 && cnnz ≤
60_000`) and a **2-seed relabelled-AMF** pass (`cn ≤ 3_000 && cnnz ≤ 30_000`).

The hypothesis was that re-exposing both in `run`'s Phase 2 under **core-shape**
gates would be compliant *and* breadth-shaped, since the capability was reachable
only through a dev-fitted window. Three arms, whole-corpus scored runs:

| arm | dev score | fill |
|---|---|---|
| `7c12f0a` (window present) | 0.792452 | 0.924478 |
| window deleted **+ both passes** under core-shape gates | **0.792452** | 0.924478 |
| window deleted, **no replacement** | 0.792455 | 0.924479 |

**The two capabilities are worth 0.03 bip corpus-wide.** They restore the
window's value exactly and win nothing anywhere else — 17 dev rows fall inside
the window and per-row ratios are *identical* on 14 of them; the whole 0.03 bip
sits on 1–3 rows (`chimera_selby-c16-01/02`, `hydroenergy2`). Timing on the
window's own rows is unchanged (16-vCPU worst 0.867 → 0.854 s; 2-vCPU worst
1.253 s, far from the tail).

**Shipped: the window and `run_sequential_180` are deleted with no replacement**
(−190 lines). Adding two AMF passes on every competitive small core corpus-wide
to buy 0.03 bip is exactly the "spend budget unconditionally on hundreds of
rows" failure mode that killed the ledger deepening in 0146; the score is
indistinguishable and the code is simpler. Dev **0.792455 / 0.924479**, official
300-pattern run **OK**.

So the compliance debt cost **0.03 bip to clear**, not the ~1.7 bip the
stage-1b band removals cost. **`run` had already subsumed the 180 path** — the
window was preserving a rounding error.

## 3. What to do with this

- **Time every candidate under `taskset` at the grader's core count.** A
  16-vCPU reading cannot rank the tail and cannot support a comparative safety
  claim. This is the single highest-value change to method in this engagement.
- **Stop buying tail safety on the giants.** They are already safe on the
  grader. Re-price `1b.indep`'s 0.31 s of waste on `faclay75` /
  `acopf_case9241pegase_qcqp` at 2 vCPU before spending any more effort there:
  at 1.01–1.02× inflation those rows have ~0.9 s of headroom.
- **Target the 1.3–1.7× inflation band**: `arki0016`, `crudeoil_lee4_09/10`,
  `gams05`, `ringpack_30_2`, `arki0013`, `mpbp_48`. A second of 2-vCPU time
  removed there is worth more than any score change currently on the table,
  because it converts a coin flip into a submission.
- The dev→hidden overfit discount is **still unmeasured**: the informational
  submission never scored, so what the identity-window removals cost hidden
  remains open.
