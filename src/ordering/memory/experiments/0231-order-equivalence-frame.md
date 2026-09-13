# 0231 — the order-equivalence frame: the *schedule* axis of determinism, measured

- **Date:** 2026-09-12 (iter51)
- **Instrument:** `ordering::probe::probe_order_equivalence` (test-only, ~80 lines, probe.rs), plus
  `target/probe-sandbox-corpora.sh` (a sandbox variant that binds structural corpora into the sandbox).
- **Result:** on **300 dev rows + 14 dev peak rows + 18 `/tmp/scale` rows + 15 `/tmp/band` rows**,
  `par1 == par2`, `par1 == seq` and `flops(par1) == flops(seq) == flops(par2)` on **every** row
  (`par_vs_par_diff=0`, `par_vs_seq_diff=0`, `flops_diff=0`).
- **Evidence:** 0231-order-equivalence-4cpu.log

## Why this frame exists

The harness's determinism gate runs `order()` twice through the **same** code path (two forked workers,
same machine, same thread count). It therefore observes run-to-run instability only — never a result
that depends on the *shape* of the internal parallel schedule. `parallel::run_candidates` drains each
candidate batch with up to `PAR_MAX_THREADS` threads; `parallel::FORCE_SEQUENTIAL` (a test-only
thread-local) forces the same batch to be drained in index order. Both paths are documented
output-neutral, but nothing in the record had ever **checked** it — and the two remote FAILs of the
parity-carrying submissions (`41baf1c`, `ab5adbb`) left "why" open, with cap cost and
order-dependence as the two live explanations.

## The measurement

Three `order()` calls per row in one process — parallel, forced-sequential, parallel again — with the
returned permutations compared element by element and the exact flops recomputed for all three.

| corpus | rows | par-vs-par diffs | par-vs-seq diffs | flops diffs |
|---|---:|---:|---:|---:|
| dev peak rows (`SSI_PROBE_ONLY`, 14 rows) | 14 | 0 | 0 | 0 |
| full dev corpus | 300 | 0 | 0 | 0 |
| `/tmp/scale` (structural, n=8000-9950, nnz/n 10-28) | 18 | 0 | 0 | 0 |
| `/tmp/band` (structural) | 15 | 0 | 0 | 0 |

The same run prices the *cost* of the schedule axis, which is the part a hidden frame could actually
see: the sequential path is uniformly slower on the rows that matter (`arki0016` 1.313 → 1.957 s,
`chimera_selby-c16-01` 1.425 → 1.569 s, `crudeoil_lee4_10` 1.049 → 1.575 s, `gabriel10` 1.023 → 1.162 s),
i.e. parallel batches buy 20-60 % on the peak rows and change no output at all.

## Consequence for the open question

Parallel-order nondeterminism is **killed as the explanation** for the two parity FAILs, at least on
every structure this box can produce: the pipeline is schedule-neutral by construction, not just by
assumption. The cost explanation therefore stands, and it has a measured profile on both frames
(dev: +0.092 s on the peak row `chimera_selby-c16-02`, +0.110 s `edgecross24-115`, +0.289 s `cont6-qq`;
structural: +0.232 s / +21 % on `ood_grid3d_16`).

## Tooling trap recorded (cost a full run)

`SSI_CORPUS_FILE` must *not* point at `/tmp/...` when the probe runs through `target/probe-sandbox.sh`:
the sandbox mounts a fresh tmpfs at `/tmp`, the corpus load fails, and the probe **silently falls back
to the dev corpus** (`unwrap_or_else(|_| crate::corpus::corpus())`). The run intended for `/tmp/band`
executed 300 dev rows instead (summary `rows=300` for a 15-row file), which is how the trap was found.
`target/probe-sandbox-corpora.sh` binds `/tmp/{band,scale,ood}` at `/corpora/*` inside the sandbox;
point `SSI_CORPUS_FILE` there.
