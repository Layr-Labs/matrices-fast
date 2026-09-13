# Remove the deferred-lift parity pass; ship the terminal class's exchange window alone

## Initial context and goal

This is the `matrices-fast` competition: fill-reducing elimination ordering for
sparse symmetric indefinite matrices, scored as a size-bucketed weighted geomean
of predicted factorization flops against feral AMD (lower is better; buckets
`lt_1k` 0.30 / `1k_10k` 0.30 / `gt_10k` 0.40). Only `src/ordering` is editable.
Every matrix must return from `order()` inside a 2 s per-matrix wall-clock cap or
the whole submission is `failed`.

The promoted frontier at the start of this iteration is **our own** submission
`e07fe7ae` at hidden **0.841768** (a nine-entry sparse-span window schedule).
Its local dev receipt is `0.791586 / 0.924134`. The goal of this iteration is a
*distinct, remotely accepted* improvement — not a dev number.

## Environment / interface

- Workspace: the cloned benchmark repo; `yukon` CLI for `run` / `submit` /
  `submissions`; the official local harness is `yukon run`
  (`bash scripts/local-candidate-build.sh && cargo run --release`), which builds
  the untrusted candidate worker inside bubblewrap (network denied) and then runs
  all 300 dev matrices under the grader's own accounting.
- One setup/build writer at a time, `CARGO_BUILD_JOBS=2` (shared host).
- All timing numbers below are from `taskset -c 0-3` runs at host load ≤ 2, i.e.
  the graded frame's 4-vCPU width, not the 24-core dev width.

## Baseline and prior work (what the record already knew)

1. The nine-window sparse-span schedule is shipped and promoted (hidden
   `0.841768`, −0.09 bips on the board; dev −0.62 bips → first measured
   dev→hidden transfer ratio on this board ≈ 1.7×).
2. Two further devices were measured on dev in this line of work:
   - **Deferred-lift parity (0227).** Stage 4b compares the held stage-1b
     "independent-set-first lift" (scored *raw*, before any polish) against the
     portfolio incumbent (scored *after* `2.descent + 3.search + 4.subtree`).
     Parity gave the lift the identical polish and kept the better of the two
     *polished* pairs. In-frame dev value: `0.791586 → 0.791480` (−1.06e-4,
     −1.34 bips), 6 rows better / 1 worse / 293 identical.
   - **Exchange window shape (0227b).** The terminal class block's exchange pass
     (`subset_window_descent_step`) is a *set of trajectories*, not a monotone
     knob: `(8,4,3)` shipped → `(12,4,5)` −1.23 bips, `(12,4,3)` best value,
     `(16,4,3)` worse than shipped. Shipped shape: width 12 / sweeps 4 / step 5.
3. Remote receipts, fetched this iteration with `yukon submissions`:
   - `e07fe7a` **promoted** 0.841768 (the nine-window tree).
   - `41baf1c` **failed** — parity alone.
   - `ab5adbb` **failed** — parity + the `(12,4,5)` exchange shape (its strict
     superset).
   Both parity-carrying submissions died; the same tree *without* parity
   promoted. That is the single cleanest attribution available on this board:
   the only common delta is the extra polish pass.

## Hypothesis and mechanism

**Hypothesis:** the parity pass is a *value* device that is not *cap-safe*: it
re-runs the whole pre-terminal polish on the held lift, and that pass costs, on
exactly the rows where the lift wins, a large fraction of the row's budget.

**Mechanism, measured (not inferred):** the per-stage phase dump on the binding
row `crudeoil_lee4_10` (n = 17 809) reads
`1.portfolio=0.4830/0.6858 … 4.subtree=0.1598/0.6763 2.descent=0.1608/0.6512
3.search=0.0010/0.6512 4b.indep-accept=0.1623/0.6452 5.terminal=0.0074/0.6452`.
The second `2.descent` + `4.subtree` pair *is* the parity pass: **+0.32 s on a
1.235 s row** (26 % of budget) to take the incumbent's 0.6763 down to the lift's
0.6452. The dev corpus peak is 1.39 s, so dev cannot exhibit the kill; a hidden
row of the same shape nearer the cap (or a structurally worse lift, whose
descent is more expensive — the descent cost is permutation-dependent: the same
descent costs 0.0012 s on the incumbent and 0.1608 s on the unpolished lift)
dies. Two independent hidden receipts agree with this reading.

**Decision:** drop the parity pass from production (restore the raw rule) and
keep the exchange shape, whose added cost is bounded and measured *on the rows
where the cap binds*.

## Implementation / files changed

`src/ordering/mod.rs` only:

1. The test-only switch is inverted and re-defaulted so that **both frames ship
   the raw rule**: `indep_parity_on()` (was `indep_parity_off()`), test body
   reads `SSI_INDEP_PARITY=1` to *enable* the device, `#[cfg(not(test))]` body
   is `false`. This also removes a `#[cfg(test)]`/`#[cfg(not(test))]` *default*
   mismatch — a class of frame bug that had already produced a 2e-4 phantom
   difference earlier in this campaign (the graded build and the probe are the
   same program on this seam now).
2. The 4b site keeps the lift's *raw* score when it wins
   (`if !indep_parity_on() { best_flops = f; best_perm = cand; }`), i.e. exactly
   the pre-0227 rule; the parity branch is retained verbatim behind the switch
   for future pricing.
3. Nothing else changed: the class block remains at exchange width 12 / sweeps 4
   / step 5 (`SSI_EXCHANGE_WIDTH`/`SWEEPS`/`STEP` test seams re-point the same
   constants; the graded build compiles the shipped values).

## Experiments run this iteration

```bash
# official local harness on the changed tree (300 matrices, sandboxed build)
CARGO_BUILD_JOBS=2 yukon run
# crate suite after the change
CARGO_BUILD_JOBS=2 cargo test --release -p ssi-candidate-worker
# remote outcomes for every submission of this line
yukon submissions
```

An earlier attempt to run the harness as a background job was **killed by the
tool's process-group cleanup** after ~9 rows (the log stops mid-table at
21:35:57); long runs must be foreground. That partial log is kept as
`0229b-attempt1-bg-killed-by-tool-pgroup.log` rather than discarded.

## Measured results

- **Official local receipt on this exact tree:** `yukon run` → 300/300 matrices,
  **score 0.791498**, tiebreak `0.924102`, buckets `0.8873 / 0.8374 / 0.6852`
  (promoted tree: `0.791586 / 0.924134`). **−8.8e-5 = −1.11 bips on dev**, same
  host/session class, no cap failure.
- In-frame per-row comparison of this tree against the parity tree it replaces
  (one binary, one session): −8.7e-5, **22 rows better / 4 worse**, worst
  `order()` 1.387 s.
- Cost of the kept device on the *binding* rows (in-frame, one binary, one
  session, 12 peak rows): `crudeoil_lee4_10` 1.2355 s at step 5 vs 1.2342 s at
  step 3; in-gate peak rows improve (`chimera_selby-c16-01` ratio
  0.6695 → 0.6669 at +0.018 s). The "+0.33 s peak-row cost" that had justified
  the older, narrower shape was a **cross-session artifact** — the same tree
  reads 1.2288 / 1.3295 / 1.5615 s on that row in three sessions.
- Crate suite: **123 passed / 0 failed / 55 ignored** (26.82 s).

## Failures / course corrections recorded

- Parity's two remote `failed` receipts (`41baf1ce`, `ab5adbbd`) are the reason
  the value device is removed even though it was worth 1.34 bips on dev: on this
  board a `failed` submission costs more than a smaller gain.
- Two *local* harness FAILs on other iterations (`clay0204m` n=222,
  `graphpart_clique-70` n=280) were environmental: the harness's 2 s cap is wall
  clock from spawn, and the same rows measure 0.307 s / 0.321 s on the pinned
  probe; the re-run completed 300/300. Local FAILs are therefore only believed
  after a re-run at low host load — which is what this receipt is.
- Alternative considered and rejected: shipping the second value device
  (`(12,4,3)`) instead. It is 1.4e-5 better on dev but costs up to +0.018 s on
  the in-gate peak rows; with a hidden cap kill already paid for twice, the
  step-5 shape is the risk-adjusted choice.

## Caveats

- Dev numbers are dev; the board's score is hidden. The last measured transfer
  ratio on this board was ≈1.7× (hidden relative / dev relative), so −1.11 bips
  dev is the working estimate for a small positive hidden move, not a promise.
- A local `yukon run` is evidence for deciding whether to submit, never an
  acceptance.
- The lift's polish cost is per-row and permutation-dependent; the numbers above
  are one row's dump plus the corpus-wide peak.

## Next steps (in order)

1. Read the remote outcome of this submission; if it promotes, the frontier is
   ours again and the next bat is the class block's next width group
   (`13/4/6, 16/4/6, 5/4/3, 24/4/11`, already priced at −1.4e-5).
2. If it `failed` on the cap, the exchange shape is the suspect (its cost lands
   on n ≤ 12 000 rows), and the next candidate is the same tree at `(8,4,3)`
   with a *cost* receipt per touched row rather than a dev peak reading.
3. The parity device can be re-opened only in a cap-safe form: a cheap
   *estimate* of the lift's polish cost before paying it (the descent on the
   unpolished lift is 130× the descent on the incumbent on `crudeoil_lee4_10`,
   so the row's own lift-penalty is measurable from work already done), or a
   bounded reduced-budget polish.
