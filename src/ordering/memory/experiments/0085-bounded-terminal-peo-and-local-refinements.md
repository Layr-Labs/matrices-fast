# Bounded terminal PEO descent and exact local refinement

2026-09-07. The final composition improves the public score from 0.8267840479210953 to **0.8267316877823669**, with **14 wins, zero losses and 286 ties**. It passed 87 active tests and the full trusted 300-case run. The gain is modest; the official outcome is unknown at upload.

Sequence: **complete leader -> PEO (2M) -> optimized five (8M) -> high-degree interval (8M)**. Each stage freshly scores its input and returns only a complete strict gain. Five/interval share wrapper code, not allowances. The heterogeneous 2M+8M+8M ledgers are neither pooled nor an instruction/runtime sum. Every inherited stage is preserved.

## Final artifact and result fields

- Baseline: `4d8641400a4050795b9674ca429f5a72430c427f`.
- Candidate commit: `348c51650c99a6396d11d38f9f27eac60b88118a`.
- Public score: **0.8267316877823669**; trusted fill ratio **0.937351** versus baseline 0.937368.
- Final active suite: **87 passed, zero failed, 20 ignored**. Tests and probe used identical hashes, unchanged during each run.
- Changed Rust files: `src/ordering/mod.rs`, `peo_tie.rs`, `probe.rs`, `rgreedy.rs`, `rgreedy/chain_interleave.rs` and `rgreedy/chain_interleave/tests.rs` (all under `src/ordering/`).
- Trusted candidate: **PASS, 300/300**, 167.558 s total, including purity/license, bijection, repeated determinism and 2-second watchdog. Exact counts match the probe; all twelve recursive Rust hashes match tests/probe/trusted/current. The local harness does not apply the official 4-GiB address-space cap.
- Official execution outcome and hidden score: unknown at upload.

## Baseline and objective

The reference is [4d8641400a4050795b9674ca429f5a72430c427f](https://github.com/Layr-Labs/matrices-fast/tree/4d8641400a4050795b9674ca429f5a72430c427f), including eight-round incumbent PEO chains, wider-input/oversized-factor ledgers and alternate starting seeds sharing 4M. Every inherited ordering/reduction/core/local/completion stage finishes first; earlier candidates, seeds and watcher allocations remain intact. The research packets studied older PEO2 code.

The public baseline is **0.8267840479210953** on 300 patterns, bucket counts 147/108/45. The aggregate weights within-bucket geomean ratios to the same default AMD by 0.30/0.30/0.40. Public and official hidden scores are separate.

The input is full symmetric CSC with diagonal omitted; input nnz counts both directions. The exact objective is F=sum(c_j^2), with each symbolic column width c_j including the diagonal. For a chordal completion H and any PEO, F(H)=n+3|E(H)|+2T(H). Applying a PEO of H to the original graph yields a subcompletion of H. Stalling under two deterministic MCS choices does not certify minimality, and an inclusion-minimal completion need not minimize F among incomparable completions.

## What the returned Pro research established

Three owner-run GPT Pro returns informed the work; precise Pro variant was not recorded. Implementation/audits/native orchestration used GPT 6 Astra, ultra effort, Codex. Integrity and arguments were checked; selected witnesses were independently recalculated. No return ran the current full native benchmark, and reported external Python counts were not local reruns.

Topics 01 and 03 converge on interval interleaving, despite Topic 01 asking for fixed-completion selection. Their hand-seed gains establish kernel barriers, not native wins: Topic 01's 1111->966 and 291978->291403 are beaten by easy seeds scoring 581 and 30989. Topic 03's 9508->9062 compares against generic Python five/four, not optimized Rust.

Topic 02 supplies tied-PEO steering and neutral exchange/deletion. Its tie witness is below the gate and repairable earlier. The native exchange trial explicitly raised its restrictive 100K proposal to 1M. Swap alone is neutral; insertion invalidates old watcher witnesses, so deletion tests are fresh.

The fill-path proof uses [Cummings, Fahrbach and Fatehpuria (2020), Definition 2.1](https://arxiv.org/html/1907.12119v2). Nested chordal deletion accessibility follows [Kijima, Kiyomi, Okamoto and Uno (2007), Proposition 2.1/appendix](https://www.kurims.kyoto-u.ac.jp/preprint/file/RIMS1610.pdf). Neither predicts native efficacy or runtime.

## Terminal mechanisms and strict fallback

The PEO helper `src/ordering/peo_tie.rs` freshly analyzes each seed; inherited score/count caches may be stale. Gate: n16..30000, input nnz<=180000, Lnnz<=300000. Six rounds maximum exactly score both MCS alternatives. Strict gain precedes an unvisited equal candidate; two consecutive neutral steps maximum reset on gain. Forward wins consideration ties. Current seed, seven-entry full-permutation history and strict-best output are separate. Exact equality detects cycles. Refusal returns only the saved strict gain, otherwise input. Controls change neutral allowance, candidate order or new budget only.

Five and interval each gate on **16<=n<=4096 and 0<input nnz<=65536**, with separate all-inclusive 8M allowances. Their shared wrapper freshly scores input, creates bounded Game scratch, reserves final replay, and requires a bijective strict full-score gain equal to predicted deltas. It does not make setup free between stages.

Five runs the existing optimized FiveWindow kernel for one five-offset/stride-five cycle, charging resets/replay and retaining completed gains on refusal. Interval schedules at most sixteen disjoint tiles of at most 32 positions by fresh squared-width mass, with at most 512 distinct live boundary vertices. The revised selector releases the three highest-current-degree vertices anywhere in a tile, with position/ID ties. Only discovery changed from the flat later/lower-degree-neighbor selector.

Interval state `(chain_prefix, selected_subset)` determines eliminated set E. Next-pivot width is one plus the adjacency union of its component in residual E union {v}, minus that component. Boundary identities are deduplicated and cannot connect components; all intervening columns count. Maximum span has 240 states/592 transitions representing 29760 interleavings, fixed scratch and deterministic ties. Equal optimum retains input.

Each move eliminates the same whole tile after the same prefix, preserving the endpoint residual/suffix. Disjoint exact deltas telescope; scratch can replay original tiles. Fresh final replay validates bijection, predicted delta and strict gain; invariant failure preserves input. No extra post-interval watcher/PEO is added.

## Completed native controls

Every row uses the same 300 cases and exact baseline. These are sandboxed direct probes, not final trusted candidate passes. Single-run elapsed differences do not prove speedups.

| Arm | Weighted public score | Better / worse / same | Worst direct call |
|---|---:|---:|---:|
| Complete baseline | 0.8267840479210953 | 0 / 0 / 300 | 0.7404 s |
| Forward PEO, 1M | 0.8267554613857986 | 4 / 0 / 296 | 0.7393 s |
| Strict-only control, same 1M | 0.8267554952515961 | 3 / 0 / 297 | 0.7121 s |
| Reverse-first, same 1M | 0.8267554613857986 | 4 / 0 / 296 | 0.7181 s |
| Forward PEO, budget-only 2M | 0.826753909989036 | 4 / 0 / 296 | 0.7169 s |
| Original interval discovery, 8M | 0.8267840479210953 | 0 / 0 / 300 | 0.7472 s |
| High-current-degree selector, same 8M | 0.826777860929238 | 4 / 0 / 296 | 0.7074 s |
| Neutral exchange/deletion, 1M | 0.8267840479210953 | 0 / 0 / 300 | 0.7136 s |
| Optimized final five-pivot control, 8M | 0.8267664351796755 | 7 / 0 / 293 | 0.7134 s |
| Final sequential composition | 0.8267316877823669 | 14 / 0 / 286 | 0.7553 s |

Strict-only continuation reproduces the three principal 1M gains. Neutral steps add only **13 FLOPs on one case**, aggregate 3.3866e-8; most value is extra strict descent. Reverse-first matches every forward score. Fourteen 1M budget stops, including three winners, and no round-cap exhaustion motivated 2M. It saves another 3776 FLOPs on one existing winner, with no broader coverage: more allowed work for a small gain.

Corrected interval diagnostics include the first test-prefixed row: 284 rows, sixteen earlier returns. Original selection scans 1718 windows on 176 cases, selects on five and solves four kernels, with no gains. High-degree selection solves 1197/1278 windows on 176 cases for four final wins, spending more of the same 8M. Exchange executes thirty neutral swaps but no further deletion/final gain. These flat/redundant arms remain negative controls.

Five diagnostics: **284 rows, 177 cases with solves, 52141 windows, 52086 solves, ten local gains/seven final winners**. Both halves and drop-five improve (remaining delta -4.220804673549239e-7). Interval has three unique winners: waterund14, gancns, chimera_lga-01. Shared rsyn0840m scores baseline/interval/five =10645/10638/10631; five has six other unique winners. Familiar five continuation has more standalone aggregate value; interval adds distinct reach.

The final composition's bucket geomeans are 0.8897136145711372 / 0.8632349290607788 / 0.7521178117319802. Both halves improve (-3.3025076703441236e-5 and -6.803268166155618e-5); dropping five contributors still improves by -1.630192985946355e-5. It matches the independently computed minimum of standalone outputs on all 300 cases. That is an observed equality, not an unseen-input guarantee for sequential search. Worst direct time is 0.7553 s versus 0.7404 s; summed ordering times are 74.5542 s versus 73.8601 s. These single runs show modest added time, not a universal bound. Standalone tie/interval gains remain concentrated in four cases each.

## Work, storage and refusal accounting

The empirical PEO tariff pays n+input_nnz before symbolic preparation, then Lnnz before reconstruction, two MCS traversals and exact scoring. Initial validation/copies cost 4n; history work adds `(2*visited_count+2)*n per round`. Omitted validation work was corrected before the probe. This fitted law is not an instruction/time bound. Failed admission pays performed preparation; history is capped and scratch released per round.

Each native 8M allowance pays validation, allocation/build, fresh scoring, search, reset/replay, copies and final validation. Game pivots prepay `width*(3*words+6)+24` before mutation. Five uses its existing scalar/subset tariff; interval prepays whole DP/component/boundary/output work. Refusal cannot publish a partial move.

Before search, each local stage reserves final replay using `sum(widths)<=floor_sqrt(n*F0)` for accumulated exact improvements. Search cannot spend it. Final replay has no new degree gate: columns may grow while total F falls. Score mismatch or finishing-ticket exhaustion is an invariant failure preserving input. Rust tariffs differ from Python, so its numerical compulsory-refusal threshold is not asserted for this port.

Two dense Game adjacency buffers occupy 4 MiB at n=4096, plus bounded linear/fixed and inherited scratch. Ordinary Vec allocation does not implement the report's fallible-allocation promise. Bounds are not whole-process memory measurements or guarantees of the 2-second cap on other hardware; the official address-space cap is 4 GiB.

## Validation status and reproduction

Baseline trusted validation passed 300 cases in 177.397 s, exact counts matching its probe; the earlier contended failure remains recorded. The candidate separately passed in 167.558 s with exact score 0.8267316877823669, rounded 0.826732, fill 0.937351. Total-run elapsed differences are not a speedup claim.

The final **87-test suite** covers the corrected tie validation charge; its earlier 72-test run did not. Earlier suites passed baseline 68, interval 78/81, exchange 71 and five 83. Independent tests cover exact elimination/interleaving widths and suffixes, filled/shared boundaries, 512/513 refusal, bit/word boundaries, ties, commit/final tickets, cycles and completed gains surviving exhaustion.

Changed-source SHA-256 (relative to `src/ordering/`; identical across tests/probe/trusted):

```text
mod.rs 4ea52b1ab81bb0812ed4e308011f49ffe3ff76c0e7915435d6a7173023c59d70
peo_tie.rs 6e11648e5983aecd26206089cf41c76b1476397c1a5c6fdc2ef7fee681aba041
probe.rs 952c4fca139e68432f5bae190481d7d2f92c7b7fc363b81b39797f53ac6e24e2
rgreedy.rs c33dc684959100b6068543d5e35b7bc1f968009c50d1c2aec09d4c6b2260450d
rgreedy/chain_interleave.rs 592918db58e90ad99f9c8ac88944ae2f6076e30146ecf4ad5bd0e0afb59ccbd7
rgreedy/chain_interleave/tests.rs 013709b5cf93a6af68ef79e7ce0a95e135915cda3a7e9c10892ddaea0a22712f
```

Reproduce from separate prepared baseline/final checkouts with the real public corpus and pinned dependencies/manifests. Run from each root. This macOS test/probe wrapper matches the controller's Seatbelt profile; candidate execution stays sandboxed and uses that checkout's corpus.

```sh
export CARGO_BUILD_JOBS=2
export CARGO_TARGET_DIR="$PWD/target"
export SSI_CORPUS_FILE="$PWD/corpus/dev/patterns.jsonl"
python3 - <<'PYTEST'
import json, os, pathlib, subprocess, sys, tempfile
assert sys.platform == "darwin", "This test wrapper is for macOS Seatbelt."
root = pathlib.Path.cwd().resolve()
target = pathlib.Path(os.environ["CARGO_TARGET_DIR"]).resolve()
target.mkdir(parents=True, exist_ok=True)
cargo_cache = pathlib.Path(os.environ.get("CARGO_HOME", pathlib.Path.home()/".cargo")).resolve()
profile = """(version 1)
(deny default)
(allow process-fork)
(allow process-exec)
(allow sysctl-read)
(allow sysctl-write)
(allow mach-lookup)
(allow file-read*)
(allow file-ioctl)
(deny network*)
(allow file-write-data (literal "/dev/null"))
"""
for writable in (target, cargo_cache, pathlib.Path(tempfile.gettempdir()).resolve()):
    profile += "(allow file-write* (subpath " + json.dumps(str(writable)) + "))\n"
base = ["/usr/bin/sandbox-exec", "-p", profile, "cargo", "test",
        "--release", "-p", "ssi-candidate-worker", "--offline", "--locked"]
for label, args in [
    ("active-tests", ["--", "--test-threads=1"]),
    ("direct-probe", ["probe_timing_and_score", "--", "--ignored",
                       "--nocapture", "--test-threads=1"]),
]:
    with open(label + ".log", "w") as log:
        subprocess.run(base + args, cwd=root, check=True, timeout=600,
                       stdout=log, stderr=subprocess.STDOUT)
PYTEST
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "bounded terminal refinement reproduction"
```

Preserve source hashes and logs. Parse COUNTS anywhere in the line, including the test-prefixed first row; require 300 matching cases and identical n/nnz/AMD. Probe timing is not watchdog validation. The final command runs the trusted gates; keep exact counts because score.json rounds. Linux uses the audited repository bubblewrap boundary, never an unsandboxed override.

Production is deterministic and pattern-only, with structural gates and no instance lookup, clock, external data or hidden probing. Telemetry never feeds decisions. Only `src/ordering/` changes. Hidden generalization and official outcome are unknown; measurements do not prove other-hardware runtime or memory.
