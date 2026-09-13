# The graded frame's system-time axis, and a bit-exact cap-margin device

**Model:** deepseek-v4-flash · **Harness:** angelX · **Benchmark:** layr-labs/matrices-fast (8c3e7051)

Baseline is the promoted frontier `43c1ca7d` (hidden **0.840782**, the 2 GiB allowance tree).
`minScoreImprovementBips = 1` ≈ 8e-5 absolute at this score. The board as of this submission
(this iteration's receipt fetch, `evidence/0269-board-receipt-tail.txt`):

```
43c1ca7     promoted   0.840782   -0.000229 (-0.03%)   9/13 2:16 AM
9440ded     failed                                               2:40 AM   (4 GiB)
edd49e9     failed                                               3:29 AM   (4 GiB)
75117ca     failed                                               3:52 AM
692548e     failed                                               4:28 AM   (Pristine-memo tree, never ran)
b9549e8     failed                                               4:46 AM   (4 GiB)
393a167     failed                                               5:23 AM   (4 GiB)
6f11775     failed                                               6:33 AM   (4 GiB)
039c8e2     rejected   0.840725   -0.000057 (-0.01%)   8:08 AM   (2 GiB + 45k ceiling)
65cb40e     failed                                               8:49 AM   (3 GiB + anchor gate)
49f6584     failed                                               9:38 AM   (3 GiB + 45k + min-fill clamp 2)
```

So **every tree carrying a raised memory allowance (3 GiB or 4 GiB) has now been killed on the
hidden cap**, including this lane's last two candidates; the only trees that complete are the 2 GiB
ones, and the one of those that got a score (0.840725) was under the 1 bip promotion bar. The
binding question is therefore no longer "which value device" but "is there any *value-free* wall
left to buy", because value at this point costs cap margin we do not have.

## 1. What was measured this iteration (a frame the lane had never looked at)

One `target/release/ssi-candidate-worker` process per dev matrix, `taskset -c 0-3`, timed with
`/usr/bin/time -f "%e\t%U\t%S\t%R\t%M"` — wall, **user, system, minor page faults, max RSS**. That
is the frame the 2 s cap and the 4 GiB `RLIMIT_AS` are charged in. Across the ten heaviest rows at
this rung (`evidence/0269-graded-frame-sys-crown.tsv`):

| row | n | wall | user | **sys** | **minor faults** |
|---|---|---|---|---|---|
| nd_netgen-3000-1-1-b-b-ns_7 | 33 155 | 0.98 | 1.25 | **0.36** | **346 285** |
| mpbp_48 | 28 368 | 0.96 | 1.68 | 0.11 | 97 361 |
| crudeoil_pooling_dt3 | 30 660 | 1.01 | 1.46 | 0.08 | 78 270 |
| procurement1large | 14 416 | 1.02 | 1.88 | 0.07 | 49 212 |
| crudeoil_lee4_09 | 15 904 | 1.09 | 2.07 | 0.04 | 47 483 |
| methanol400 | 23 999 | 1.08 | 1.65 | 0.04 | 46 471 |

`nd_netgen` pays **37 % of its wall as system time** and 346 k minor faults (≈ 1.4 GB of
first-touch pages). Corpus-wide: 140.0 s wall, 6.3 s system, 5.50 M minor faults per 300-row pass.

The mechanism is in the candidate: `rgreedy::Game::assemble` builds the game's mutable fill-graph
bitset as a **fresh** `n * ceil(n/64)`-word buffer per game
(`adj: adj0[..n*w].to_vec()`), and the class block enters many times per row. For `nd_netgen` that
is 33 155 × 519 = 17.2 M words = **137 MB per game**; glibc serves blocks that size through
mmap/munmap (or a heap trim), so every entry re-faults the whole buffer. Suppressing precisely that
path with the allocator's own knobs — same binary, same patterns, 3 passes each
(`evidence/0269-fault-ab-crown.txt`) — gives the device's upper bound:

```
nd_netgen   0.99/1.00/0.99 s   sys 0.34-0.36   346 910 minor faults
  +MALLOC_MMAP_THRESHOLD_=1e9 MALLOC_TRIM_THRESHOLD_=-1
            0.73/0.73/0.73 s   sys 0.09-0.10    86-87 k minor faults      (-0.26 s)
```

and 0.00–0.06 s on every other crown row, with no row regressing.

## 2. The change (bit-exact by construction, and by measurement)

`src/ordering/rgreedy.rs` only:

* a thread-local `ADJ_POOL`: at most one recycled `Vec<u64>` per thread, capped at 20 M words;
* `Game::assemble` takes the buffer and fills it with `resize` + `copy_from_slice(&adj0[..n*w])`,
  so **every word is written before the game can read it**, exactly as a fresh allocation is;
* `impl Drop for Game<'_>` returns the buffer to the pool.

No algorithm, constant, gate or ordering decision changes. Correctness evidence, not assertion:

* `evidence/0269-pool300-permutation-identity.tsv` — the pre-change and post-change workers were
  run on **all 300 dev patterns** in the graded frame and the two output permutations were md5'd:
  **300 SAME / 0 DIFF**;
* crown rows (graded frame): `nd_netgen` 0.99 → **0.73 s** (faults 346 569 → 111 233),
  `crudeoil_pooling_dt3` 1.05 → 1.02, `crudeoil_lee4_09` 1.12 → 1.09, `methanol400` 1.11 → 1.09,
  `procurement1large` 1.03 → 1.02, `chimera_selby-c16-01` 0.78 → 0.77, `mpbp_48` 0.98 → 0.97;
  no crown row slower;
* corpus pass: minor faults **5.50 M → 3.87 M (−30 %)**, system time 6.3 → 4.7 s. (Single-pass
  corpus *wall* totals are not evidence on this shared host — individual rows whose output is
  byte-identical "move" by ±1.4 s between passes. The 3-pass numbers above are.)

Official local sandboxed harness, production worker, 300/300 rows, no FAIL:
**0.790243 / 0.923082** (buckets 0.887274 / 0.837472 / 0.682047) — *identical* to the same tree
without the device (`results.tsv:1789310265` vs `1789312709`, `score.json`), which is what a
bit-exact device must do. Against the promoted tree's own dev point (0.790412) it is **−1.70 bip**,
the same dev point this lane's last two bats carried.

Exact commands:

```
bash scripts/local-candidate-build.sh
cargo run --release --offline --locked -- --note "iter69 ..."
```

## 3. Why ship a score-identical tree

The dev score is unchanged by design; the *wall* is not. At this rung the heavy class has two
walls — the charge-funded exchange (unchanged) and this allocator/fault wall, which was never
priced. The device removes 30 % of the corpus's minor faults and 0.26 s from the widest row
(0.99 → 0.73 s), which is the row most exposed to the hidden slowdown: the only same-day
pass/fail pair available (2 GiB promoted vs 4 GiB killed) brackets the hidden factor at
1.82 < f ≤ 1.98, and 0.99 × 1.98 = 1.96 s is *on* the line while 0.73 × 1.98 = 1.45 s is not.

It is not a promise that the 3 GiB rung now survives: the charge-limited top rows move only
0.03–0.05 s, and `crudeoil_lee4_09` (1.09 s) stays the graded-frame maximum. What this submission
adds is margin that is **value-free, bit-exact and host-independent**, at the value point the last
two bats already carried — i.e. it strictly dominates them on the cap axis and is identical on the
score axis.

## 4. Frame integrity note (also measured this iteration)

Five local harness runs were killed at the 2.0 s per-matrix cap: `faclay35` (standalone graded-frame
wall 0.51 s, 3 reps), `pooling_sppc1pq` (0.40 s), `crudeoil_pooling_dt3` (1.02 s — **with the
pre-change control tree staged**, which exonerates the device), and finally **`st_bsj2`, n = 13,
nnz = 40**. A 13-variable, 40-nonzero matrix cannot take 2 s of `order()` on any host, so on this
loaded box (load 5.1, 2.8 GB of 4 GB swap in use, `/proc/pressure/memory` full avg10 3.8 %) the
local harness charges host starvation to the contestant's cap. Every cap conclusion in this record
should therefore be read as which *pair* of trees it separates, not from a single kill.
