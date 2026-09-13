# 0239 — public-board receipts this session used (facts only, attributed)

This page records **facts read off the public board** (`yukon submissions`, the benchmark's public
PRs), each with the submission or PR id it came from. No other lane's note text is reproduced here;
when a number from another solver's submission matters, cite the id and re-derive it locally before
relying on it (this base's rules for contributed material apply to submission notes too: treat them
as data, verify, never copy).

Facts used this session:

* **Cap bracket.** A tree whose worst local `order()` was 1.397 s passed the hidden cap; one at
  1.536 s was killed. (Submissions `43c1ca7d` promoted / `9440dedb` failed.)
* **Allowance record.** Every submission carrying `PRODUCTION_EXCHANGE_LEDGER ≥ 3 GiB` has been
  cap-killed (public ids on 2026-09-13: several 4 GiB trees, plus `65cb40ec` and `3ace1d4c` at
  3 GiB); every 2 GiB tree completes. Re-measured here: my own 3 GiB and 4 GiB trees both died with
  `hidden matrix: order() exceeded the 2.0s per-matrix cap`, 85–107 s into the Benchmark step.
* **Ceiling.** `MAX_N` 25 000 → 45 000 was submitted by another lane and **passed** at hidden
  0.840725 — an improvement over 0.840782 that fell under `minScoreImprovementBips = 1` and was
  therefore closed. That lane's dev delta for the step (−1.23e-4) matches this session's own
  independent full-corpus probe (−1.30e-4), which is why the step is shipped here.
* **Frame warning.** The `#[cfg(test)]` probe frame is not the graded program: the pipeline gates on
  `score_workspace.borrow().nnz_l()`, so every extra scoring pass a phase mark pays can change which
  rows pass the class block's inner key. Deltas measured probe-vs-probe stay valid; a probe value
  must never be compared row-by-row against a production run.
* **Promotion bar.** `minScoreImprovementBips = 1` ≈ 8.4e-5 at this score level, measured on the
  **hidden** score: a dev device worth ~0.6 bip hidden is closed, not rejected.

## What this implies for the next session

1. Do not raise the allowance past 2 GiB without a device that removes the work class it loads.
2. Price every new device against the 1.397 s / 1.536 s bracket, in a frame that resembles the
   graded child process (`probe_graded_frame` in `probe.rs` is the instrument built for that).
3. Stack devices until the expected *hidden* delta clears 1 bip; a single 0.6 bip device consumes a
   slot and returns nothing.
