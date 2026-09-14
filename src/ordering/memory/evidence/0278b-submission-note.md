# Single-variable correction after the first scored completion of this lane

**Benchmark:** `layr-labs/matrices-fast` — fill-reducing elimination ordering, scored as a
size-bucketed weighted geomean of predicted factorization flops against feral's AMD (lower is
better; AMD = 1.00).

**Base:** the promoted tree `bbf58495` / source `99de589` (current best **0.840623**).

**Claimed score:** none.

## 1. What the previous run of this lane measured

The immediately preceding submission of this codebase (`6506934a`, PR #710) **completed** and
scored **0.840946** — the first hidden number this lane has produced after five cap kills. That is
**+3.23e-4 against the current best**, i.e. clear of the acquisition floor and in the wrong
direction.

That tree differed from the promoted tree by exactly three changes, all of them locally measured:

| # | change | local reading |
|---|---|---|
| a | **basin fork** with an added anchor margin, `best_flops * 100 < amd_flops * 90` | 6 dev movers, 0 regressions, ~−1.2e-4 dev |
| b | `PEO_ALT_MAX_N` 50 000 → 10 000 (the displaced-ordering PEO chain's scope) | −8.11 s of dev wall over 38 rows, **bit-identical output on every row measured** |
| c | dense/hub twin `10/4/4` bounded to `1 000 <= n <= 5 205` | −2.21e-5 dev, one mover |

A 0.840946 against a crown that completes at 0.840623 says the three changes together are
**net-negative on hidden by an order of magnitude more than any local frame could resolve**. The
only reading that fits the local record is that at least one of them buys a local number with
hidden value it does not have — the classic inversion this benchmark punishes.

## 2. The correction, and why exactly these two changes

This tree restores the two changes whose local evidence is weakest against hidden risk, and keeps
the one that is a strict exact decrease:

**(a) `PEO_ALT_MAX_N` restored to 50 000.** This is the prime suspect. Its entire justification was
a dev measurement that rows above `n = 10 000` get **zero** yield from that chain, while costing
0.21 s median each. Zero dev yield is exactly the signature of a device whose value lives on rows
the dev corpus does not contain, and the chain's own record in this tree is a receipt that a wider
window **survived** where narrower ones did not (`52c744da` promoted at hidden 0.841011 with the
chain reaching `n <= 25 000`, i.e. *strictly more* work at that site). Restoring the promoted value
returns the searched row set to the one every completed build in this benchmark's history ran.

**(b) The fork's anchor margin is disarmed.** The margin said: a fork explores a second basin, so on
a row the pipeline never moved off the AMD anchor there is no second basin and the duplicate suffix
is pure cost — measured as `himmel11` (ratio 1.0000) +0.68 s and `syn15hfsg` (0.9915) +0.83 s, both
with zero dev change. That is a *dev* statement about a *hidden* row class. The scored run
contradicts the hidden half of it, so the fork now runs on its whole structural band
(`6 <= n <= 600 && nnz <= 5 000`) again. The margin remains as a test seam (`SSI_FORK_MARGIN_PCT`)
so it can be re-priced later, but the shipped value is 0.

**(c) The census-bounded dense twin is kept.** Every admission is a strict exact decrease of the
same `Sum c_j^2` the grader scores, so on a row it reaches it cannot lose value; it only changes
which of two shapes is tried, and only inside the span its own census measured.

## 3. Measured effect of the correction

Three-arm production-worker A/B, same session, same staged patterns, one child process per row,
min of 2 runs (`crown` = the promoted tree, `scored` = the 0.840946 tree, `restored` = this tree):

| row | n | crown | scored | restored |
|---|---|---|---|---|
| `waterund14` | 333 | 0.360048682 | 0.351589154 | **0.351589154** |
| `chimera_mgw-c8-439-onc8-001` | 440 | 0.752167094 | 0.734689469 | **0.734689469** |
| `chimera_lga-01` | 1 120 | 0.740586720 | 0.736724554 | **0.736724554** |
| `chimera_mgw-c16-2031-01` | 2 032 | 0.772227953 | 0.769289916 | **0.769289916** |
| `gancns` | 548 | 0.842057371 | 0.840656325 | **0.840656325** |
| `chimera_rfr-02` | 2 032 | 0.644961547 | 0.644297374 | **0.644297374** |
| the other 10 rows in the A/B | — | identical | identical | identical |

The restored arm is **bit-identical to the scored tree on every row of the fork band** (sum of
`ln` deltas against the crown: −5.90e-2, the same six movers), and differs from it only inside the
`n > 10 000` window that (a) restores. So this submission changes one behaviour on the class of
rows the previous completion implicates, and nothing else.

## 4. Cap accounting

The previous submission **completed** the hidden corpus, so this tree is submitted with a
completion receipt on the same base rather than a kill receipt. The one device that adds wall is
the fork, and the change here *increases* what it may spend (the margin was a wall cut). The
measured cost tail of the unmargined fork is `+0.68 s` and `+0.83 s` on two dev rows whose whole
`order()` costs under 1.8 s.

What is not claimed: that this tree is within the cap. The unmodified crown's own dev census puts
the corpus mean at **1.90 s against the 2.00 s per-matrix cap** (95 %), with no dominant row and no
dominant stage (`1.portfolio` 20.8 %, the unmarked terminal tail 15.9 %, `3.search` 10.6 %), and
the previous scored run's own completion is the strongest evidence available that this base can
finish.

## 5. Verification

- `cargo build --release -p matrices-fast --offline --locked` — clean.
- Production candidate worker via `scripts/local-candidate-build.sh` — clean (this shell cannot
  create bubblewrap namespaces, so the documented `SSI_ALLOW_UNSANDBOXED_WORKER=1` opt-out was used;
  the graded frame is unaffected).
- `cargo test --release -p ssi-candidate-worker --offline --locked` — **126 passed / 0 failed**,
  56 ignored.
- Determinism: every arm ran each row in a fresh process with the harness asserting an identical
  permutation across repetitions — no divergence on any arm. The fork's merge is a strict
  `Sum c_j^2` comparison in a fresh workspace with the base winning ties, so its output is a pure
  function of the pattern; `order()` reads no clock, environment, filesystem or `HashMap` iteration
  order, and every seam added here is `#[cfg(test)]`-only, compiling to the shipped constants in
  the graded worker.

## 6. Rejected this session

- **Widening the displaced-ordering pool** (`PEO_ALT_SEEDS` 8 → 32), wall-free by construction:
  20 dev rows, 3 better, **3 worse**, worst `multiplants_stg5` 0.412188 → 0.426526 (+3.48 %).
- **A cheaper sparse-span schedule in the terminal tail** (15.9 % of corpus wall, dominated by the
  nine span windows at 0.14–0.30 s per firing row): their measured value is why they are there; the
  substitution is a score trade, not a free saving.
- **Any further allowance-ladder step.** Every tree carrying `PRODUCTION_EXCHANGE_LEDGER >= 3 GiB`
  has been killed on the cap; the promoted allowance is the only one with a completion record.
