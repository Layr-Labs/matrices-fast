# 0234 — the class-block exchange family on the current frontier: OFF / 8-4-3 / 12-4-5, and the `MAX_WIDTH = 14` trap

**Frame.** One binary, one process/session, 300 dev rows, production `#[cfg(test)]` probe
(`probe_timing_and_score`), `taskset -c 0-3`, repo's own sandboxed probe runner
(`target/probe-sandbox.sh`). Every arm below differs **only** by the three arguments of the
class-block call to `rgreedy::subset_window_descent_step` at the terminal chain
(`src/ordering/mod.rs`), exposed by a test-only `SSI_EXCHANGE_WIDTH` / `SSI_EXCHANGE_SWEEPS`
/ `SSI_EXCHANGE_STEP` seam; the shipped path hardcodes one shape.

**Tree.** The current frontier `78c434c` / `7df69b92` (fetched from
`refs/heads/submissions/78c434c0-8975-4dc8-b769-2a82e48c078e`), i.e. *not* this lane's tree:
`git diff -w 475be33 7df69b9 -- src/ordering` is the frontier author's own four code files
plus one new module, and the frontier's class block still calls `…, 8, 4, 3, …`.

| arm `(width, sweeps, offset_step)` | dev probe SCORE | worst `order()` | log |
|---|--:|--:|---|
| `(16, 4, 7)` — **rejected by the guard ⇒ exchange is OFF** | 0.791940 | 1.243 s | `0234-merge-x16-s4-t7-4cpu.log` |
| `(24, 4, 11)` — identical to the above, 300/300 rows | 0.791940 | 1.242 s | `0234-merge-x24-s4-t11-4cpu.log` |
| `(8, 4, 3)` — **what the frontier ships** | 0.791782 | 1.334 s | `0234-front-only-4cpu.log` |
| `(12, 4, 5)` — this lane's device (submitted as `a34c109a`) | 0.791694 | 1.391 s | `0234-merge-x12-4cpu.log` |
| `(12, 5, 5)` | **0.791678** | 1.366 s | `0234-merge-x12-s5-t5-4cpu.log` |
| `(12, 6, 5)` | 0.791677 | 1.370 s | `0234-merge-x12-s6-t5-4cpu.log` |
| `(12, 8, 5)` | 0.791677 | 1.347 s | `0234-merge-x12-s8-t5-4cpu.log` |
| `(13, 4, 6)` | 0.791738 | 1.393 s | `0234-merge-x13-s4-t6-4cpu.log` |
| `(14, 4, 6)` | 0.791766 | 1.379 s | `0234-merge-x14-s4-t6-4cpu.log` |

## Three facts this curve establishes

1. **The class-block exchange is worth −1.58e-4 dev as the frontier ships it** (OFF 0.791940
   vs `(8,4,3)` 0.791782) and **−2.46e-4 at `(12,4,5)`**. The "OFF" arm is new: it is the
   first measurement of what this device buys on this base, and it comes for free.
2. **Sweeps is the live axis, width is not.** 4 → 5 pays a further −1.6e-5 and the curve then
   saturates (5/6/8 differ by 1e-6 while the worst row moves only inside the ±0.03 s
   single-draw noise band). Widening the window with `offset_step = 6` *loses*
   (`(13,4,6)` 0.791738, `(14,4,6)` 0.791766 — both worse than `(12,4,5)`).
3. **A width above the internal ceiling silently disables the whole site.**
   `rgreedy/window_dp.rs` has `const MAX_WIDTH: usize = 14;` and
   `subset_window_descent_config` begins with
   `!(2..=max_span).contains(&width) || offset_step >= width || sweeps == 0 || budget <= 0 → None`.
   So `(16,4,7)` and `(24,4,11)` do not run a *wider* search — they run **no** search, and the
   two arms are bit-identical on all 300 dev rows. Reading that as "wider is worse" would have
   been the wrong conclusion, and the same silent `None` is available to any future parameter
   probe that crosses 14.

## Consequence for the next bat

The prepped next candidate is `(12, 5, 5)` on the same frontier base: −1.6e-5 dev better than
the in-flight `(12,4,5)` and no worse on the worst row. It is deliberately **not** submitted
yet: the same-family step is 0.16 bip, i.e. below the practical acceptance boundary measured
from the public ledger (see `0234-acceptance-boundary-ledger.txt`), so the value of the next
submission has to come from a *different* device stacked on this base, not from this
parameter.
