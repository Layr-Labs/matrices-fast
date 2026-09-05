# 0060 — Ledger-gated terminal escalation

**Base commit:** `ea67ff80041e8e7717be32decdf95c1c1e80eb90` (`ea67ff8`), the
promoted tip (official hidden **0.869826**).

**Base re-measured on this box, twice, before any edit:**

| run | score | lt_1k | 1k_10k | gt_10k | worst `order()` |
|-----|-------|-------|--------|--------|-----------------|
| 1   | 0.843978 | 0.8903 (147) | 0.8670 (108) | 0.7920 (45) | 1.378 s (`crudeoil_lee4_10`) |
| 2   | 0.843978 | 0.8903 | 0.8670 | 0.7920 | 1.375 s (`crudeoil_lee4_10`) |

The score is bit-identical across runs, as expected — `flops_of` is a pure
function of `(pattern, permutation)`. Only the times move.

**Result:** dev **0.843978 → 0.843231** (**−7.47 bip**), fill 0.943347.
35 better / **0 worse** / 264 unchanged. All three buckets improve.
Worst `order()` among matrices the change actually touches: **1.179 s**, against
a base worst of 1.375–1.378 s.

Box: 4-core x86_64 Linux VM, 15 GB RAM. All timings below are from this box in
this session; none are carried across from another page.

---

## Hypothesis

Two claims, the second of which is the one that mattered.

1. *(from the task brief)* Headroom lives on matrices already below the AMD
   anchor, scaled by how far below they are, so search budget should be
   allocated by the margin `best_flops / amd_flops`.
2. *(mine)* The chain's real defect is not its allocation but its **stopping
   rule**. Every subtree round from 2 onward runs only if the previous round
   strictly improved, so one stalled round drops a matrix out of the search
   permanently — even when most of its 2 s budget is unspent. If the stall is a
   property of the round's *seed* rather than of the incumbent, then simply
   re-running the search on a freshly postordered tree with a new seed will keep
   converting, and the only thing stopping us doing that everywhere is cost.

Claim 2 is what the measurement supports. Claim 1 turned out to be true as a
*description* of where gain lives, but useless as a *gate* — see below.

## Files and commands

Edited: `src/ordering/mod.rs`, `src/ordering/probe.rs`. Nothing else.

```sh
# base and candidate score + timing (~125-145 s)
cargo test --release -p ssi-candidate-worker --offline --locked -- \
    --ignored --nocapture --test-threads=1 probe_timing_and_score

# the two measurements that decided the design
cargo test --release -p ssi-candidate-worker --offline --locked -- \
    --ignored --nocapture --test-threads=1 probe_margin_cascade   # ~190 s
cargo test --release -p ssi-candidate-worker --offline --locked -- \
    --ignored --nocapture --test-threads=1 probe_work_ledger      # ~145 s
cargo test --release -p ssi-candidate-worker --offline --locked -- \
    --ignored --nocapture --test-threads=1 probe_stream_axis      # ~235 s

# full trusted run
bash scripts/local-candidate-build.sh && cargo run --release --offline --locked
```

## What the measurements said

### The stall is a seed artefact, not exhaustion

`probe_margin_cascade` takes the shipped `order()` output and appends up to six
*unconditional* subtree rounds, each re-postordering and using a fresh seed.
Ungated, over the whole corpus:

| cascade depth | score | Δ bip | worst total `order()` | movers |
|---|---|---|---|---|
| 0 (base) | 0.843978 | — | 1.378 s | — |
| 1 | 0.843397 | −5.81 | 1.505 s | 67 |
| 2 | 0.843215 | −7.63 | 1.629 s | 67 |
| 3 | 0.842707 | −12.71 | 1.693 s | 70 |
| 4 | 0.842195 | −17.83 | 1.814 s | 95 |
| 5 | 0.841931 | −20.47 | 1.905 s | 95 |
| 6 | 0.841798 | −21.80 | 2.008 s | 95 |

67 matrices move on the *first* extra round. The chain had already declared
itself finished on all of them. So the conditional stopping rule is discarding
live search, and the ceiling on this technique is time, not quality.

### Margin describes the gain but cannot gate it

Conversion versus margin, one extra round (`probe_margin_cascade`, depth 1):

| margin band | matrices | movers | mean gain among movers |
|---|---|---|---|
| < 0.50 | 12 | 5 (41.7%) | 0.0065 |
| 0.50–0.70 | 28 | 10 (35.7%) | 0.0067 |
| 0.70–0.85 | 43 | 9 (20.9%) | 0.0011 |
| 0.85–0.95 | 62 | 15 (24.2%) | 0.0010 |
| 0.95–0.9999 | 71 | 23 (32.4%) | 0.0017 |
| ≥ 0.9999 (tied) | 84 | 4 (4.8%) | 0.0039 |

Both the conversion rate and the mean gain do rise as margin falls, exactly as
the brief predicted. But margin is the wrong variable to *gate* on, because the
binding constraint is time and margin does not predict time. The three matrices
that break the cap first (`crudeoil_lee4_10` 0.674, `arki0013` 0.615,
`crudeoil_lee4_09` 0.788) are all deep below the anchor — margin-scaled
escalation spends most where it can least afford to.

### `(n, nnz)` cannot separate the expensive matrices either

An oracle allowed to see each matrix's `order()` time, and to take the deepest
cascade that still fits under the base worst, scores **0.842067 (−19.11 bip)** —
so ~88% of the ungated gain is time-feasible. A grid of 216 `(n, nnz)`-keyed
gates crossed with margin-scaled depths produced **zero** configurations under
the bar. The reason is direct: the chain is conditional, so size does not
predict its cost. `crudeoil_lee1_07` (n=3670, nnz=19322) costs 1.14 s;
`rsyn0810m04m` (n=4772, nnz=13836) costs 0.99 s; `batchs121208m` costs a fifth
of that. Any envelope wide enough to admit the cheap ones admits the expensive
ones too.

### The requested-work ledger separates them exactly

`work_spent` accumulates what each exact-search stage *asked for* in word-ops.
Because every gate that decides a stage is a pure function of the pattern, so is
the total — it satisfies the determinism contract by construction, and it is
free (a handful of integer adds). `probe_work_ledger` prints it against time:

| ledger band | matrices | max `order()` |
|---|---|---|
| < 2.0e9 | 225 | **0.713 s** |
| ≥ 2.0e9 | 75 | 1.383 s |

The split is clean and it is the split that matters. Note the *rank* correlation
is poor (`corr(log10 ledger, secs) = 0.399`, worse than `log10 n` at 0.750) —
small matrices request large nominal budgets and finish instantly. The ledger is
not a cost estimator; it is a reliable **upper envelope**, which is all a safety
gate needs.

### Depth chosen on the measured score/time curve

With the gate at 2.0e9, over matrices that escalate:

| rounds | score | Δ bip | worst escalated `order()` | headroom under base worst |
|---|---|---|---|---|
| 1 | 0.843639 | −3.39 | 0.824 s | 0.554 s |
| 2 | 0.843540 | −4.38 | 0.960 s | 0.419 s |
| 3 | 0.843489 | −4.89 | 1.032 s | 0.347 s |
| **4** | **0.843231** | **−7.47** | **1.178 s** | **0.201 s** |
| 5 | 0.843053 | −9.25 | 1.304 s | 0.075 s |
| 6 | 0.842999 | −9.79 | 1.372 s | 0.007 s |

Rounds 5 and 6 are worth a further 1.8 and 0.5 bip and consume nearly all the
remaining margin. **Four rounds shipped.**

## Variants that lost

| variant | score | Δ bip | why rejected |
|---|---|---|---|
| ungated cascade, depth 4 | 0.842195 | −17.83 | worst 1.814 s, far over the base worst |
| ungated cascade, depth 6 | 0.841798 | −21.80 | worst 2.008 s — at the SIGKILL |
| ledger gate, depth 5 | 0.843053 | −9.25 | 0.075 s headroom; not worth 1.8 bip |
| ledger gate, depth 6 | 0.842999 | −9.79 | 0.007 s headroom |
| margin-scaled depth (6/4/3/2 by band), no ledger | 0.842805 | −11.73 | worst 2.002 s |
| margin-scaled (4/3/2/1), no ledger | 0.843111 | −8.67 | worst 1.814 s |
| ledger gate + margin depth, ties→1 | 0.843529 | −4.49 | costs 3.0 bip; see the tie finding |
| ledger gate + margin depth, ties→0 | 0.843662 | −3.16 | costs 4.3 bip |
| ledger gate + margin depth (4/3/2/1) | 0.843586 | −3.92 | strictly worse than uniform depth 4 |
| ledger ceiling 2.4e9 / 2.8e9 / 3.3e9, depth 4 | 0.843111 | −8.67 | admits the 1.38 s matrices; worst 1.814 s |
| ledger ceiling 1.0e9 | — | — | identical eligible set to 2.0e9 (220 vs 225); no gain, tighter |
| more streams instead of more rounds | — | — | see below |
| `(n,nnz)` skip gates × margin depths (216 combos) | — | — | none under the time bar |

### Streams are a worse axis than rounds (negative)

`probe_stream_axis` splits one escalated pass into setup and search, and sweeps
`streams`. Setup is *not* the cost — median 0.0003 s, max 0.052 s — the search
is (median 0.0157 s, max 0.134 s at one stream), so streams cost roughly
linearly. At ~4× the cost, 4 streams return 2.14× the log-gain of 1 stream,
while 4 *rounds* return 3.07×. Re-postordering between rounds gives a genuinely
new block decomposition; extra streams only re-roll the same one. **Do not
buy trajectories with `streams` when you can buy them with rounds.**

## The tie finding — this contradicts the brief

The brief states, from an earlier note, that matrices tied at exactly 1.0000 are
at a local optimum "the exact elimination game cannot leave", that no extra
search converts them, and that the `mod.rs` header claiming otherwise is wrong.

Measured here, that is right about the elimination-game LNS and **wrong about
subtree refinement**. Of 83 eligible ties, 3 move under the cascade:

| matrix | bucket | n | nnz | gain at depth 6 |
|---|---|---|---|---|
| `gasprod_sarawak81` | gt_10k | 22536 | 75636 | **5.53%** |
| `popdynm200` | gt_10k | 22407 | 105584 | **3.32%** |
| `gabriel10` | gt_10k | 244056 | 1148210 | 0.02% |

Only 3.6% of ties convert — the brief is right that ties are a bad *target*. But
all three are `gt_10k`, where one matrix carries ~0.7 bip per 1% of flops, so
those three alone are worth ~5.3 bip. Withholding escalation from ties costs
3.0–4.3 bip of the 7.47 (see the variant table). The two claims are compatible:
the elimination game is trapped at a tie, but re-postordering the elimination
tree and searching *different subtrees* is a different move, and it escapes.

**Do not "target ties" — but do not exclude them from a monotone pass either.**

## Robustness

The change is monotone by construction (best-of floor), so its true effect can
never be positive; the question is only magnitude on a different corpus.

- 35 better / **0 worse** / 264 unchanged. No matrix ends above the AMD anchor.
- Corpus halves: −3.08 bip / −16.88 bip. Same sign, very different magnitude.
- Drop the top 1 / 3 / 5 / 10 movers: −5.83 / −2.44 / −1.26 / −0.60 bip.
- Bootstrap over the 300 dev matrices (2000 resamples): mean −9.76 bip,
  95% interval **[−20.92, −2.10]**.

**The gain is concentrated and the honest read is that the dev number is soft.**
Ten matrices carry almost all of it. The interval does not cross zero only
because the mechanism cannot lose. Treat −7.47 bip as "somewhere between 2 and
20 bip on a fresh corpus", not as a point estimate. (This also answers the
long-standing bootstrap question in `open-questions.md`; the machinery is in the
analysis, not yet in a probe.)

## Timing discipline

The comparative rule: stay at or below the worst case of a revision known to
have passed the grader.

| revision | worst `order()` | matrix |
|---|---|---|
| base, run 1 | 1.378 s | `crudeoil_lee4_10` |
| base, run 2 | 1.375 s | `crudeoil_lee4_10` |
| candidate | 1.393 s | `crudeoil_lee4_10` |
| candidate, worst matrix that ACTUALLY escalates | **1.179 s** | `supplychainr1_053050` |

`crudeoil_lee4_10` has ledger 2.096e9, above the 2.0e9 ceiling, so it is skipped
and executes byte-identical code in both revisions — the only added instructions
on its path are the ledger's integer adds. The 1.375/1.378/1.393 spread is
measurement noise on the same code, and the base varies by 0.003 s between its
own two runs. The number that characterises this change is 1.179 s, roughly
0.2 s below the base worst.

That said: this is a 4-core VM and the base worst here is 1.38 s against a 2 s
cap, which is tighter than several pages in `memory/` assume. The margin is
real but it is not generous.

## Validation

`bash scripts/local-candidate-build.sh && cargo run --release --offline --locked`
completed: `results.tsv` row `OK`, `score.json` written with score
**0.843231**, fill 0.943347, 300 matrices, no worker failures. Buckets
0.890169 / 0.866072 / 0.790897. Note the harness prints the literal `(capped)`
in the time column for *every* matrix (`src/main.rs:417`) — it is not a cap
indicator.

44 unit tests pass, including `order_is_deterministic`,
`best_of_is_never_worse_than_amd` and
`subtree_configs_stay_within_matrix_work_limit`.

## What I would try next

1. **Raise the ceiling with a cheaper escalation.** The oracle says −19.11 bip
   is time-feasible and this ships −7.47. The 75 excluded matrices hold ~half
   the remaining gain. The way in is not a higher ceiling but a *smaller* round
   for them — the ledger already tells you which matrices are near the cap, so
   spend 8M×8 rather than 32M×32 there and see whether that fits.
2. **Spend the ledger the other way.** It measures work *requested*, so it can
   also identify matrices that finished the chain far under budget and give them
   more rounds, rather than a uniform 4. That is the margin-scaled idea done
   against the variable that actually binds.
3. **Make the ledger count work *issued* rather than requested.** `subtree_refine`
   knows how many word-ops it consumed; returning that would turn the envelope
   into an estimator and would let the ceiling be set much closer to the cap.
4. **Put the bootstrap in a probe.** It is currently a Python script over
   `probe_timing_and_score` output. It changed how I read this result and it
   should be available before the next one.
5. Do **not** re-run the streams axis, `(n,nnz)` cost gates, or margin-only
   gating without a new mechanism — all three are measured negatives here.

---

## VERDICT: submitted and FAILED on the hidden corpus

Submitted 2026-09-05 04:56 UTC as `f6665de9`, commit `99b54fd`. Status
**failed** — not rejected on score, but a hard failure:

> workflow run concluded failure at step "Benchmark":
> https://github.com/Layr-Labs/matrices-fast/actions/runs/33945892055

The workflow logs need admin rights on the benchmark repository, so the failing
matrix is not directly observable. The cause is nonetheless identifiable from
the design, and the follow-up in `0061` measured it:

**The gate bounded expensive SEARCH and nothing else.** `work_spent` counts only
the exact-search stages. It is blind to the O(nnz) candidate construction —
AMD, METIS, AMF, the relabel families — which on some matrices is the entire
cost. `ringpack_30_2` has a ledger of 48M, one fortieth of the 2e9 ceiling, and
still takes 0.714 s. On the dev corpus the low-ledger matrices happened to top
out at 0.714 s, so the envelope looked clean; nothing in the mechanism made that
true, and on the hidden corpus it evidently was not.

**The escalation's own cost was never bounded against the gate that admitted
it.** Four rounds at 32M x 32 blocks request 4.1e9 word-ops — more than twice
the 2.0e9 ceiling used to decide the matrix was cheap. Worse, `0061` measured
that wall time is very nearly linear in `budget` (median 3.98x for a 4x budget),
so that request is a real time commitment and not a loose upper bound: a matrix
whose blocks saturate the budget pays all of it, roughly 0.17 s per round per
32M x 32, plus an O(nnz) re-postorder per round.

So the shipped configuration could add ~0.5-0.7 s to a matrix that the gate had
already misjudged as cheap. On dev that produced a worst escalated `order()` of
1.179 s and looked safe. It was not.

**Closed — do not retry:** a ledger-only cost gate, and any escalation whose
requested work is large relative to the ceiling that admits it. The mechanism
itself (unconditional re-postordered rounds after the conditional chain) is
sound and is re-shipped in `0061` under a conjunctive gate at a quarter of the
budget.
