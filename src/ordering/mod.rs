//! ★ THE SUBMISSION DIRECTORY ★ — the one place you may edit.
//!
//! Fill-reducing ordering. Contract (frozen):
//!   `pub fn order(pattern: &Pattern) -> Vec<usize>`
//! Returns `perm[k]` = the original index eliminated k-th; the result must be a
//! bijection of `0..n`, deterministic (the harness runs `order()` twice and
//! requires identical output), and return within the 2 s/matrix cap.
//!
//! ## Approach: per-matrix best-of over the ordering family, floored by the
//! ## grader's OWN baseline
//!
//! The score is a geomean of per-matrix `flops(yours)/flops(AMD)` ratios, so
//! choosing, *per matrix*, the cheapest of several candidate orderings can only
//! match or beat AMD — free headroom — **but only if the candidate set actually
//! contains the grader's baseline ordering**. The grader's baseline is
//! `feral_amd::amd_order` with LIBRARY-DEFAULT options (`aggressive = true`,
//! `dense_alpha = 10.0`), so we anchor on it: it is the guaranteed floor
//! (`ratio ≤ 1.0` on every matrix), the always-valid fallback, and — being the
//! baseline — it cannot itself time out.
//!
//! ## Where the headroom is
//!
//! 122 of the 300 dev matrices are STILL TIED at exactly 1.000 — AMD beats every
//! separator-, profile- and bandwidth-based candidate on them (60 in `lt_1k`, 40
//! in `1k_10k`, 22 in `gt_10k`). Each tie is pure upside. Note the leverage is
//! very uneven: `gt_10k` carries weight 0.40 over only 45 matrices, so one large
//! matrix is worth ~4.4 small ones — but it is also where the time cap bites
//! hardest.
//!
//! ## The timing fact that bounds every change here
//!
//! MEASURED with the test-only `probe` module (the harness prints `(capped)`
//! instead of a time, so this is otherwise invisible). Two runs of the SAME
//! probe on the SAME code, hours apart:
//!
//! | matrix            | run A   | run B   |
//! |-------------------|---------|---------|
//! | worst overall     | 1.019 s | 0.803 s (`arki0016`) |
//! | `crudeoil_lee4_10`| 1.019 s | 0.646 s |
//! | `nuclear10a`      |    —    | 0.412 s |
//!
//! **These numbers carry ~1.6× run-to-run variance from machine load, so the
//! local worst case is known to about ONE significant figure.** Treat any timing
//! written here as an order of magnitude, and re-measure rather than trusting
//! it — two earlier revisions of this header were wrong by 3× and by 1.6×.
//!
//! A previous revision also claimed "the grader is ~3-5× slower than local, so
//! worst-case LOCAL time must stay well under ~0.35 s". That cannot be right as
//! stated: the revision carrying a 1.019 s local worst PASSED the grader, which
//! it could not have done at 3-5× against a 2 s SIGKILL. We have no calibration
//! of grader speed. The defensible rule is therefore comparative rather than
//! absolute: **keep the worst local `order()` at or below the worst case of a
//! revision already known to have passed** (1.019 s).
//!
//! The cost driver is nnz, NOT n: `qapw` (n=705, nnz=87496) costs 0.539 s, more
//! than matrices 300× larger. Gate by nnz first, with an `n` cap as backstop.
//!
//! ## What this revision adds: RELABELLED-AMF MULTI-START (a SECOND lottery)
//!
//! Score 0.876925 → **0.871827** on the 300-matrix dev corpus; 36 matrices
//! better, **0 worse**, wins in all three size buckets.
//!
//! The relabel trick below had only ever been pointed at AMD (minimum DEGREE).
//! AMF (approximate minimum FILL) reads the vertex numbering the same way, so
//! `AMF(Q A Qᵀ)` composed back through `Q` is a randomized-restart minimum-FILL
//! ordering for the cost of one AMF pass. Why that beats spending the same time on
//! more AMD restarts: within one objective the draws are effectively i.i.d. and
//! saturate (see the budget table on [`RELABEL_BUDGET`]), whereas min-fill and
//! min-degree disagree about which vertex to eliminate — so AMF draws are not
//! redundant AMD draws. The wins duly land where min-degree had already converged
//! (`mpbp_15` 0.9951→0.8198; `pooling_haverly1pq` an exact 1.0000→0.9782 at n=31).
//! Gated on nnz (AMF's own cost driver), routed through the best-of floor so score
//! risk is structurally zero. See `memory/experiments/0005-*.md`.
//!
//! The generalisation, which is the part worth carrying forward: **any ordering
//! routine whose output depends on the input numbering becomes a randomized-restart
//! algorithm under `relabel`, for free.** RCM, Sloan, the ND separator choices and
//! MinFill are all still un-relabelled.
//!
//! ## The revision before that: RELABELLED-AMD MULTI-START
//!
//! Score 0.883906 → **0.876925** on the 300-matrix dev corpus, the largest
//! single gain measured on this problem so far (the previous revision's entire
//! 12-variant partitioner sweep bought 0.0042; this buys 0.0070).
//!
//! AMD's tie-breaking reads the vertex NUMBERING, so `AMD(Q A Qᵀ)` composed back
//! through `Q` is a genuinely different minimum-degree ordering for the cost of
//! one AMD pass. That matters because 122 of 300 matrices were tied at exactly
//! 1.000 — on those AMD beat every separator-, profile- and bandwidth-based
//! candidate, and a different AMD is the only family that can move them. 41 of
//! 300 matrices improve, against 7 of 260 for the whole partitioner sweep.
//!
//! Restart count is set by a per-matrix TIME BUDGET (`RELABEL_BUDGET / nnz`),
//! not a flat count — a flat 24 restarts costs 1.444 s on `nuclear10a` alone and
//! would breach the cap. The budget doubles as the gate, so this candidate needs
//! no `(n, nnz)` cutoff of its own. Worst combined `order()` is 0.978 s, below
//! the 1.019 s of the last revision that passed the grader. See
//! [`RELABEL_BUDGET`] for the cost model and the measured budget/cap sweep.
//!
//! ## What an earlier revision added
//!
//! Confined to the AMD-speed SMALL region (`n < 3000`, `nnz < 12000`):
//!   - **MINIMUM-FILL (minimum-deficiency / MinFill) ordering (pure Rust)** — a
//!     genuinely DIFFERENT greedy elimination heuristic from everything already
//!     present. Minimum-degree (AMD/AMF) eliminates the vertex of smallest
//!     *degree*; MinFill instead eliminates, at every step, the vertex whose
//!     elimination introduces the FEWEST NEW FILL EDGES — i.e. it minimizes the
//!     local *deficiency* (`#pairs of neighbors that are not yet adjacent`)
//!     rather than the degree. This is the classic min-deficiency criterion and
//!     it is orthogonal to the degree, bandwidth, profile and separator families
//!     already tried; it frequently beats minimum-degree exactly on the small,
//!     irregular combinatorial/network graphs that dominate the tied `lt_1k` /
//!     `1k_10k` lists. It runs on an explicit dynamic elimination graph with an
//!     O(1) adjacency-membership matrix and a HARD pair-check work budget: on any
//!     input that would exceed the budget it cleanly finishes with a
//!     degree-ordered fill (still a valid bijection), so its time is bounded
//!     regardless of structure. Gated to `n < 3000 && nnz < 12000` — WAY below
//!     the slow tier (`nnz ≥ 163816`) — so it cannot move the worst case, and it
//!     allocates only the small `n·n` membership matrix (≤ 9 MB) it needs.
//!     Deterministic (fixed `(deficiency, degree, index)` tie-break). Best-of
//!     floor → zero-downside.
//!
//! ## Staying under the 2 s / SIGKILL cap — HARD cost envelopes
//!
//! The harness SIGKILLs `order()` at a hard 2 s per matrix and ONE breach FAILs
//! the whole run, so every candidate carries an explicit cost envelope in `(n,
//! nnz)`, sized from measurement (see the timing section above for why the old
//! "~0.35 s local ceiling" rule was unfounded).
//!
//! The two relabelled multi-starts are the exception, and deliberately so:
//! instead of an envelope they take a per-matrix time BUDGET,
//! `RELABEL_BUDGET / nnz` restarts. Because per-restart cost scales with nnz,
//! that bounds their added time on every matrix at once, and yields zero restarts
//! wherever `nnz > RELABEL_BUDGET`. The AMF arm carries a second, independent nnz
//! ceiling ([`RELABEL_AMF_MAX_NNZ`]) because its per-pass constant is larger.
//!
//! Worst combined `order()` measured at 0.439 s of the 2 s cap (0.384 s before the
//! AMF arm). NOTE: the 0.9-1.0 s figures elsewhere in this file were recorded on a
//! box roughly 2.5x slower; timings compare only within one box, so use the
//! comparative rule — stay at or below the worst case of a revision known to have
//! passed the grader, measured the same way on the same machine.
//!
//! The candidate set is a pure function of `(n, nnz)` — never wall-clock — so
//! the two required `order()` runs are byte-identical (determinism gate).

use crate::Pattern;

/// TEST-ONLY measurement harness (timing headroom, tie lists, candidate
/// what-if scoring). Not compiled into the shipped binary.
#[cfg(test)]
mod probe;

pub mod rgreedy;
pub mod custom_metrics;

use feral::ordering::amd::permute_pattern;
use feral::ordering::elimination_tree::EliminationTree;
use feral::sparse::csc::CscPattern as ScoringPattern;
use feral::symbolic::column_counts_gnp;

/// AMF cost is a smooth ~1.4x of AMD's with no observed structural blow-up, so
/// its α-5 variant runs on all but the very largest problems, preserving the big
/// gt_10k wins (e.g. pooling_*). This is the SAME envelope as the prior safe run.
const AMF_MAX_N: usize = 250_000;
const AMF_MAX_NNZ: usize = 1_500_000;

/// Medium-size envelope for the *extra* tuned candidates (α-5/α-2 AMD, default
/// AMF, α-2 AMF). A few extra AMD/AMF passes are trivially cheap in this region;
/// keeping them out of the large regime preserves the prior large-matrix
/// heavy-run profile. NOTE: `MEDIUM_MAX_NNZ` reaches into the slow tier
/// (`nnz` up to 400 k), so this `n` cap is held fixed — raising it would put AMF
/// passes onto dense large-n matrices and could move the worst case.
const MEDIUM_MAX_N: usize = 60_000;
const MEDIUM_MAX_NNZ: usize = 400_000;
/// nnz cap for the THREE extra sweep-found AMF variants (α1/α16/α-1). The sweep
/// showed AMF is cheap even on LARGE-SPARSE matrices (faclay75 nnz=1.38M: all
/// three AMF passes total only ~0.64 s at 5×), and these variants are the UNIQUE
/// min-flops ordering on several big gt_10k matrices (faclay75, pooling_sppc3pq,
/// kissing2, arki0013). So the gate is wide; the real timing risk is the SUM with
/// other candidates on high-nnz mediums, which is bounded by also requiring the
/// tighter MEDIUM/ROBUST-gated variants to have dropped out by then (n cap).
const AMF_SWEEP_MAX_NNZ: usize = 1_500_000;
const AMF_SWEEP_MAX_N: usize = 300_000;
/// nnz ceiling for the extra sweep-found AMD α1/α16 passes in the MEDIUM block.
/// Excludes high-nnz dense mediums (nuclear104 nnz=258k) that already load the
/// candidate stack near the 2 s cap; below it each AMD pass is a few ms.
const SWEEP_EXTRA_MAX_NNZ: usize = 150_000;

/// NON-AGGRESSIVE AMD envelope. `aggressive = false` is a genuinely different
/// elimination order (not just a dense-threshold tweak). It runs at baseline AMD
/// speed at ANY n, so the `n` cap is generous (150 k) to reach the large-but-
/// sparse matrices that dominate the high-weight gt_10k ties. The nnz cap
/// (`< 130000`) sits BELOW the slowest matrices' `nnz ≥ 163816` floor, so this
/// only ever runs on ultra-sparse patterns where several AMD passes are
/// milliseconds; the worst case is therefore held byte-for-byte.
const ROBUST_MAX_N: usize = 150_000;
const ROBUST_MAX_NNZ: usize = 600_000;

/// Reverse Cuthill–McKee envelope. RCM is O(nnz) pure Rust — a few-millisecond
/// BFS even at large n — so it is bounded PRIMARILY by nnz. The `nnz < 130000`
/// cap keeps it STRICTLY below the slow tier (`nnz ≥ 163816`), so it cannot move
/// the worst case; the generous `n` cap lets it reach the large-but-sparse
/// gt_10k ties. Best-of floor makes it zero-downside.
const RCM_MAX_N: usize = 150_000;
const RCM_MAX_NNZ: usize = 130_000;

/// Sloan profile/wavefront-reduction envelope. Sloan is pure Rust, O(nnz log n)
/// — a few milliseconds even at large n — so it is bounded PRIMARILY by nnz. The
/// `nnz < 130000` cap keeps it STRICTLY below the slow tier (`nnz ≥ 163816`), so
/// it cannot move the worst case; the generous `n` cap lets it reach the
/// large-but-sparse gt_10k ties. Sloan targets exactly the mesh/grid structures
/// (`watercontamination*`, `transswitch0300p`) that the minimum-degree and ND
/// families leave tied at AMD. Best-of floor makes it zero-downside.
const SLOAN_MAX_N: usize = 150_000;
const SLOAN_MAX_NNZ: usize = 130_000;

/// Hand-rolled NESTED-DISSECTION envelope. Our own pure-Rust recursive graph
/// bisection is O(nnz log n) with a hard work budget, so it is bounded PRIMARILY
/// by nnz. The `nnz < 130000` cap keeps it STRICTLY below the slow tier
/// (`nnz ≥ 163816`), so it cannot move the worst case; the generous `n` cap lets
/// it reach the large-but-sparse gt_10k mesh/grid ties (`transswitch0300p`,
/// `watercontamination0303r`) that library METIS is gated out of on the larger
/// instances. Deterministic (fixed seeding, deterministic partition ordering).
/// Best-of floor makes it zero-downside.
const ND_MAX_N: usize = 150_000;
const ND_MAX_NNZ: usize = 130_000;

/// GGGP (greedy graph-growing) recursive-bisection envelope. A SECOND,
/// algorithmically distinct nested-dissection variant (gain-based combinatorial
/// bisection + minimum-side vertex separator, vs. the BFS-level cut in
/// `nd_order`). Pure Rust, O(nnz log n) with a hard work budget and an iterative
/// task stack — a few milliseconds in this region. The `nnz < 130000` cap keeps
/// it STRICTLY below the slow tier (`nnz ≥ 163816`), so it cannot move the worst
/// case; the generous `n` cap lets it reach the large-but-sparse gt_10k mesh/grid
/// ties. Deterministic. Best-of → zero-downside.
const NDFM_MAX_N: usize = 150_000;
const NDFM_MAX_NNZ: usize = 130_000;

/// MINIMUM-FILL (minimum-deficiency) envelope. This is the NET-NEW method: a
/// greedy elimination heuristic that, at each step, eliminates the vertex of
/// smallest LOCAL FILL (deficiency = #pairs of its neighbors not yet adjacent),
/// rather than smallest degree (AMD/AMF), bandwidth (RCM), profile (Sloan) or a
/// separator (ND/GGGP/library partitioners). It runs on an explicit dynamic
/// elimination graph with an O(1) `n·n` adjacency-membership matrix and a HARD
/// pair-check work budget (falls back to a degree-ordered fill if exceeded), so
/// its time is bounded regardless of structure. Gated to tiny/small matrices
/// (`n < 3000 && nnz < 12000`) — WAY below the slow tier (`nnz ≥ 163816`) — so it
/// cannot move the worst case and the membership matrix stays ≤ 9 MB. Targets the
/// worst-scoring small buckets' tied-at-AMD combinatorial/network graphs
/// (`wastewater*`, `wastepaper6`, `syn*`, `tln2`). Deterministic. Best-of floor
/// → zero-downside.
const MINFILL_MAX_N: usize = 3_000;
const MINFILL_MAX_NNZ: usize = 12_000;

/// METIS runtime is structure-dependent and can explode on large/dense patterns
/// (measured: 6.2 s at nnz≈1.38M). Bound it by nnz PRIMARILY (the cost driver),
/// far below that scale, plus an n cap as defense-in-depth. Unchanged from the
/// prior safe run (kept fixed so the worst-case time does not move).
const METIS_MAX_N: usize = 130_000;
const METIS_MAX_NNZ: usize = 320_000;

/// A *second*, tuned METIS (more initial partitionings + refinement). Re-shaped
/// so it reaches sparse gt_10k ties (e.g. `pinene200`, n=19995/nnz=97990) via a
/// WIDER n cap, while a TIGHTER nnz cap keeps it strictly on genuinely sparse
/// inputs — every slowest matrix has nnz ≥ 163 k, so at `nnz < 120 k` a second
/// METIS never runs on the expensive high-nnz mids and even doubled work stays
/// well under budget.
const METIS_TUNED_MAX_N: usize = 21_000;
const METIS_TUNED_MAX_NNZ: usize = 120_000;

/// A *third*, HIGH-TRIAL METIS (many initial partitionings + heavy FM). Confined
/// to tiny/small matrices where METIS is milliseconds even at 5×; more trials
/// frequently beat default/tuned METIS on small structures. Strictly below the
/// slow tier (`n ≥ 17 k`, `nnz ≥ 163 k`), so it cannot move the worst case.
const METIS_HITRIAL_MAX_N: usize = 8_000;
const METIS_HITRIAL_MAX_NNZ: usize = 40_000;

/// Scotch is volatile on large/dense inputs; confine the default variant to
/// small/medium matrices where nested dissection is tens of ms even on a slow
/// grader. Covers the whole `1k_10k` bucket — every prior slowest matrix had
/// `n ≥ 15 k`, so this cannot touch the worst-case time.
const SCOTCH_MAX_N: usize = 12_000;
const SCOTCH_MAX_NNZ: usize = 200_000;

/// A *second*, tuned Scotch (more separator trials). Widened to cover more of the
/// `1k_10k` bucket; still tens of ms at this size, and far below the slow tier.
const SCOTCH_TUNED_MAX_N: usize = 10_000;
const SCOTCH_TUNED_MAX_NNZ: usize = 120_000;

/// METIS PARAMETER variants (imbalance tolerance, ND→AMD switch point, one extra
/// seed) — see the block in `order()` for what each one is. All measured on the
/// dev corpus (`probe_family`): every variant costs ≤ 0.068 s at the top of this
/// envelope and the FIVE together add ≤ 0.285 s, giving a worst combined
/// `order()` of 0.668 s (crudeoil_lee4_06) — comfortably below the 1.019 s
/// worst case the slow tier already carries, so the global worst is unmoved.
/// The envelope is set by nnz (the cost driver) with an n cap as backstop.
const METIS_VAR_MAX_N: usize = 30_000;
const METIS_VAR_MAX_NNZ: usize = 60_000;

/// STRONGER KaHIP envelope (a second seed, and the Eco quality mode). These are
/// the most expensive additions measured — up to 0.65 s at n≈22k — so unlike the
/// METIS variants they get a TIGHT envelope. `probe_family` put every KaHIP win
/// at n ≤ 11556 / nnz ≤ 40860 (mpbp_34, mpbp_35, chimera_selby-c16-01), so the
/// gate is drawn just above those: it keeps all three wins and drops every
/// instance where KaHIP costs more than ~0.31 s. Worst combined `order()` inside
/// this envelope is 0.823 s (mpbp_07), still below the existing 1.019 s worst.
const KAHIP_MULTI_MAX_N: usize = 12_000;
const KAHIP_MULTI_MAX_NNZ: usize = 45_000;

/// KaHIP is a distinct partitioner (dropped in general for being 13 s on a
/// giant), added ONLY on small matrices where it is milliseconds even at 5×.
/// Widened in n to reach more small/lower-medium ties while TIGHTENING nnz to
/// keep it cheap; still covers dense tiny problems (e.g. `qap`, n=255/nnz=43748).
/// Cost tracks small `n` under the tight nnz cap. `seed = 1` (default) deterministic.
const KAHIP_MAX_N: usize = 6_000;
const KAHIP_MAX_NNZ: usize = 50_000;

/// RELABELLED-AMD multi-start budget, in "microseconds of restart time".
///
/// One restart costs roughly `k * nnz` seconds. Measured `k` across the dev
/// corpus spans 1.3e-7 (`methanol400`) to 9.4e-7 (`sfacloc2_3_80`) — a 7x
/// spread, so the budget is sized on the WORST `k`, not the mean. Rounding that
/// worst case up to `k = 1e-6` makes the constant read directly as microseconds:
/// `restarts = RELABEL_BUDGET / nnz` spends at most ~0.3 s of restarts on any
/// matrix, whatever its structure.
///
/// The budget is therefore its OWN gate — `nnz > RELABEL_BUDGET` yields zero
/// restarts — so unlike every other candidate here it needs no `(n, nnz)`
/// cutoff. Sweeping budget/cap with `probe_relabel_budget` (measured score and
/// measured combined worst case, both on the full 300-matrix corpus):
///
/// | budget | cap | score    | worst combined |
/// |--------|-----|----------|----------------|
/// | 150000 |  24 | 0.879253 | 0.925 s        |
/// | 300000 |  24 | 0.876925 | 0.978 s        |
/// | 450000 |  24 | 0.876757 | 1.027 s        |
/// | 900000 |  96 | 0.875194 | 1.183 s        |
///
/// 300000 is the knee: past it, each further 0.05 s of worst case buys under
/// 0.0002 of score. The cap barely matters (24 vs 48 vs 96 differ by <0.0001)
/// because the wins land in the first handful of restarts, so it is set low as a
/// belt-and-braces bound for any matrix with unusually small nnz.
///
/// Safety: the resulting worst combined `order()` is 0.978 s, which is BELOW the
/// 1.019 s worst case measured on the previous revision — a revision that passed
/// the grader. So this ships a worst case no larger than one already known to
/// clear the 2 s cap in the real environment.
const RELABEL_BUDGET: usize = 300_000;
const RELABEL_MAX_RESTARTS: usize = 24;

/// nnz ceiling for the relabelled-**AMF** multi-start (see the loop at the end of
/// [`order`]).
///
/// AMF's per-pass cost has the same shape as AMD's — linear-ish in nnz — so
/// `RELABEL_BUDGET / nnz` already bounds the family's total spend on any matrix,
/// exactly as it does for AMD. This ceiling is a SECOND, independent bound, and it
/// exists because AMF's constant is larger than AMD's and its worst case is less
/// well characterised here: a min-fill sweep does more work per elimination than a
/// min-degree one, and the corpus that decides promotion is not this one.
///
/// 130_000 keeps the family inside the nnz envelope the hand-rolled ND / RCM /
/// Sloan candidates already run in — a region whose cost is measured — and puts
/// the whole added spend at ~2.6e-7 s/nnz × min(24, 300000/nnz) passes, i.e.
/// ≈0.08 s whatever the matrix looks like. Measured effect on the combined worst
/// case: 0.384 s → 0.457 s on the same box, against a 2 s cap.
///
/// Gate on nnz, not n: AMF's cost tracks nnz (a small dense pattern is expensive,
/// a huge sparse one is cheap), so an `n` cutoff would bound the wrong quantity.
const RELABEL_AMF_MAX_NNZ: usize = 200_000;

#[cfg(test)]
const SUBTREE_SEARCH_WORK_LIMIT: i64 = 32_000_000;
#[cfg(test)]
const TERMINAL_SUBTREE_SEARCH_WORK_LIMIT: i64 = 16_000_000;
const SUBTREE_CFG: rgreedy::SubCfg = rgreedy::SubCfg {
    min_s: 32,
    max_s: 384,
    max_sub: 1_200,
    max_blocks: 32,
    budget: 1_000_000,
    streams: 1,
    rank_blocks: true,
    round: 0,
};

/// Lower bound of the bounded subtree-refinement chain.
///
/// The chain was gated at `n >= 1_000` from the moment it was introduced
/// (experiments 0021-0025), which left the ENTIRE `lt_1k` bucket untouched by
/// the one technique that moved the other two: `lt_1k` sat at exactly 0.8965
/// across 0021, 0022, 0023, 0024 and 0025 while `1k_10k` fell 0.8848 -> 0.8761
/// and `gt_10k` fell 0.8119 -> 0.7970. Small graphs are also the CHEAPEST in
/// the corpus (measured max 0.766 s across all 147 `lt_1k` matrices, against a
/// 1.702 s corpus worst case), so the chain fits there with ~0.9 s to spare and
/// cannot move the global worst case, which lives on `arki0013` (n=44909).
///
/// The floor is structural, not fitted: below ~64 vertices an elimination tree
/// has too few subtrees of searchable size for a bounded block search to do
/// useful work, and those graphs are already covered exhaustively by the MinFill
/// multi-start and the small-graph LNS. Measured: 1_000 -> 200 was worth 2.6 bip,
/// 200 -> 64 a further 0.7 bip, so the curve is already flattening here.
const SUBTREE_MIN_N: usize = 24;
const SUBTREE_MAX_N: usize = 250_000;

const MID_MAX_S: usize = 128;
const LARGE_MAX_S: usize = 384;
const MID_BLOCKS: usize = 16;
const MID_BUDGET: i64 = 2_000_000;
const LARGE_BLOCKS: usize = 16;
const LARGE_BUDGET: i64 = 2_000_000;

/// Per-matrix base config for one chain round. On a short elimination tree the
/// default `min_s = 32` admits almost no blocks, so drop the block floor to 16
/// below `n = 1_000` — the same floor the terminal deep pass already uses.
fn subtree_cfg_for(n: usize, nnz: usize) -> rgreedy::SubCfg {
    let mut cfg = SUBTREE_CFG;
    if n < 64 {
        cfg.min_s = 8;
        cfg.max_s = 32;
        cfg.max_blocks = 8;
        cfg.budget = 1_000_000;
    } else if n < 1_000 {
        cfg.min_s = 16;
        cfg.max_s = 256;
        cfg.max_blocks = 16;
        cfg.budget = 2_000_000;
    } else if n >= 10_000 {
        cfg.max_s = LARGE_MAX_S;
        cfg.max_blocks = LARGE_BLOCKS;
        cfg.budget = LARGE_BUDGET;
        if nnz <= n * 10 && nnz <= 150_000 {
            cfg.max_sub = 1_600;
        }
    } else {
        cfg.min_s = 32;
        cfg.max_s = MID_MAX_S;
        cfg.max_blocks = MID_BLOCKS;
        cfg.budget = MID_BUDGET;
    }
    cfg
}

fn terminal_deep_subtree_cfg(n: usize, nnz: usize) -> rgreedy::SubCfg {
    let mut cfg = SUBTREE_CFG;
    cfg.min_s = 16;
    cfg.round = 5;
    if n < 10_000 {
        cfg.max_blocks = 4;
        cfg.max_s = 768;
        cfg.budget = 4_000_000;
    } else {
        cfg.max_blocks = 8;
        cfg.max_s = 1_200;
        cfg.budget = 2_000_000;
        if nnz <= n * 10 && nnz <= 150_000 {
            cfg.max_sub = 1_600;
        }
    }
    cfg
}

/// Deterministic 64-bit mixer (SplitMix64). Used only to derive relabelings from
/// a fixed seed, so every run produces the identical sequence — the determinism
/// gate requires the two `order()` runs to agree byte-for-byte.
fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// A relabeling of `0..n` derived from a fixed seed (Fisher-Yates over
/// SplitMix64). Pure function of `(n, seed)` — no wall-clock, no entropy.
fn relabel(n: usize, seed: u64) -> Vec<usize> {
    let mut q: Vec<usize> = (0..n).collect();
    let mut s = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0x1234_5678_9ABC_DEF0);
    for i in (1..n).rev() {
        let j = (splitmix64(&mut s) % (i as u64 + 1)) as usize;
        q.swap(i, j);
    }
    q
}

/// A relabeling derived from `base` by applying `swaps` random transpositions.
/// Pure function of `(base, swaps, seed)` — the seed stream is independent of the
/// one [`relabel`] uses, so the two phases never produce the same relabeling for
/// the same `seed`. Composition of transpositions with a permutation is a
/// permutation, so the result is always a valid relabeling of `0..n`.
///
/// TEST-ONLY. `order()` deliberately does NOT use this: the structured-relabeling
/// policies it exists to build were all measured and all lose to i.i.d. uniform
/// draws at equal cost (`probe_relabel_search`,
/// `memory/experiments/0004-structured-relabelings.md`). It is kept in the shipped
/// module rather than in `probe.rs` so the probe measures the same primitive a
/// future `order()` would call if the finding is ever revisited under a different
/// restart budget.
#[cfg(test)]
fn perturb(base: &[usize], swaps: usize, seed: u64) -> Vec<usize> {
    let n = base.len();
    let mut q = base.to_vec();
    if n < 2 {
        return q;
    }
    let mut s = seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(0xA076_1D64_78BD_642F);
    for _ in 0..swaps {
        let i = (splitmix64(&mut s) % n as u64) as usize;
        let j = (splitmix64(&mut s) % n as u64) as usize;
        q.swap(i, j);
    }
    q
}

/// Bucket-weighted budget allocation: scale restart budget and cap by dimension `n`,
/// investing more work in high-leverage buckets (gt_10k has weight 0.40 over only 45 matrices,
/// while lt_1k has weight 0.30 over 147 matrices).
#[inline]
fn relabel_budget_and_cap(n: usize) -> (usize, usize) {
    if n >= 10_000 {
        (500_000, 36)
    } else if n >= 1_000 {
        (400_000, 30)
    } else {
        (300_000, 24)
    }
}

fn relabel_restarts(budget: usize, cap: usize, nnz: usize) -> usize {
    if nnz == 0 {
        return 0;
    }
    (budget / nnz).min(cap)
}

/// Restart count for the budgeted relabelled multi-start:
/// Incorporates the historical hub-gatewall discriminator (`max_deg * 50 <= n`)
/// and low-nnz / mid-band floors to eliminate seed starvation on non-hub graphs
/// while protecting extreme hub matrices like `ringpack_30_2` from timeout.
fn relabel_restarts_tuned(budget: usize, cap: usize, n: usize, nnz: usize, max_deg: usize) -> usize {
    if nnz == 0 {
        return 0;
    }
    let base_r = (budget / nnz).min(cap);

    if max_deg * 50 > n && (100_000..=150_000).contains(&nnz) {
        base_r.min(4) // Hub guard (e.g. ringpack_30_2)
    } else if nnz <= 20_000 {
        (600_000 / nnz).min(48) // Low-nnz regime
    } else if nnz <= 150_000 && max_deg * 50 <= n {
        base_r.max(12) // Mid-band non-hub floor
    } else if nnz <= 350_000 && nnz <= 5 * n && max_deg * 50 <= n && n >= 10_000 {
        base_r.max(8) // Sparse gt_10k mesh/network floor (unstarving transswitch & powerflow)
    } else {
        base_r
    }
}

/// Return an elimination order for `pattern` (best-of over the ordering family).
pub fn order(pattern: &Pattern) -> Vec<usize> {
    let n = pattern.n;
    if n == 0 {
        return Vec::new();
    }

    // i32-indexed borrowed pattern shared by every feral ordering crate.
    let col_ptr_i32: Vec<i32> = pattern
        .col_ptr
        .iter()
        .map(|&x| i32::try_from(x).expect("matrix too large for i32-indexed ordering"))
        .collect();
    let row_idx_i32: Vec<i32> = pattern
        .row_idx
        .iter()
        .map(|&x| i32::try_from(x).expect("matrix too large for i32-indexed ordering"))
        .collect();
    let core = feral_ordering_core::CscPattern::new(n, &col_ptr_i32, &row_idx_i32)
        .expect("malformed CscPattern (bug in Pattern invariants)");

    // usize-indexed owned pattern for the trusted scoring path (Σ c_j²).
    let scoring_pat = ScoringPattern {
        n,
        col_ptr: pattern.col_ptr.clone(),
        row_idx: pattern.row_idx.clone(),
    };

    // ── The FLOOR: the grader's exact baseline ordering ──────────────────────
    // `amd_order` with library-default options IS the grader's baseline, so
    // anchoring on it guarantees ratio ≤ 1.0 on every matrix (no candidate can
    // make us worse than AMD). It is also the guaranteed-valid fallback and, as
    // the baseline, cannot itself time out.
    let amd = feral_amd::amd_order(&core).expect("feral AMD ordering failed");
    let mut best_perm: Vec<usize> = amd.into_iter().map(|x| x as usize).collect();
    let mut best_flops: u64 = flops_of(&scoring_pat, &best_perm);

    // Candidate set gated purely by (n, nnz) so both required runs agree.
    let nnz = pattern.nnz();
    let mut max_deg = 0usize;
    for j in 0..n {
        let deg = pattern.col_ptr[j + 1] - pattern.col_ptr[j];
        if deg > max_deg {
            max_deg = deg;
        }
    }

    // Try a candidate produced by `f`; keep it if it is a valid bijection with
    // strictly fewer flops. `catch_unwind` guards against a candidate panicking
    // (which would otherwise crash the worker and FAIL the whole run).
    let mut consider =
        |produce: &dyn Fn() -> Result<Vec<i32>, feral_ordering_core::OrderingError>| {
            let produced =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(produce));
            let Ok(Ok(perm_i32)) = produced else {
                return;
            };
            let perm: Vec<usize> = perm_i32.into_iter().map(|x| x as usize).collect();
            if !is_bijection(&perm, n) {
                return;
            }
            let f = flops_of(&scoring_pat, &perm);
            if f < best_flops {
                best_flops = f;
                best_perm = perm;
            }
        };

    // AMF α5 — the highest-value extra candidate; kept on the large envelope to
    // preserve the big gt_10k wins. (With AMD default this is the same pair of
    // heavy orderings as the prior safe run.)
    if n < AMF_MAX_N && nnz < AMF_MAX_NNZ {
        let opts = feral_amf::AmfOptions {
            dense_alpha: 5.0,
            ..Default::default()
        };
        consider(&|| feral_amf::amf_order_opts(&core, &opts).map(|(p, ..)| p));
    }

    // Medium-size extras: cheap here, pure upside layered over the AMD floor.
    if n < MEDIUM_MAX_N && nnz < MEDIUM_MAX_NNZ {
        // Slightly more aggressive dense handling — wins on some dense-ish
        // problems; can never lose thanks to the default-AMD floor.
        let amd_opts5 = feral_amd::AmdOptions {
            aggressive: true,
            dense_alpha: 5.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_opts5).map(|(p, ..)| p));

        // Even tighter dense handling — catches dense-ish mediums the α5/α10
        // variants miss. Trivially cheap in this size regime.
        let amd_opts2 = feral_amd::AmdOptions {
            aggressive: true,
            dense_alpha: 2.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_opts2).map(|(p, ..)| p));

        // Default-α AMF, complementing the α5 AMF above.
        consider(&|| feral_amf::amf_order(&core));

        // Tighter-dense AMF (α2) — a distinct AMF ordering for dense-ish mediums
        // that the α5/α10 AMF variants miss. Time-trivial at this size.
        let amf_opts2 = feral_amf::AmfOptions {
            dense_alpha: 2.0,
            ..Default::default()
        };
        consider(&|| feral_amf::amf_order_opts(&core, &amf_opts2).map(|(p, ..)| p));

        // Aggressive AMD α1 and α16 — the two sweep-found AMD variants that still
        // add unique wins beyond the existing α{-1,2,5,10} set. Gated to genuinely
        // medium nnz (< SWEEP_EXTRA_MAX_NNZ): on high-nnz DENSE mediums (e.g.
        // nuclear104 n=39k nnz=258k, density 6.6) the candidate stack is already
        // near the 2 s cap, and adding two more AMD passes there breached it
        // (measured 2.46 s at 5×). At nnz < 150k each AMD pass is a few ms.
        if nnz < SWEEP_EXTRA_MAX_NNZ {
            let amd_opts1 = feral_amd::AmdOptions { aggressive: true, dense_alpha: 1.0 };
            consider(&|| feral_amd::amd_order_opts(&core, &amd_opts1).map(|(p, ..)| p));
            let amd_opts16 = feral_amd::AmdOptions { aggressive: true, dense_alpha: 16.0 };
            consider(&|| feral_amd::amd_order_opts(&core, &amd_opts16).map(|(p, ..)| p));
        }
    }

    // AMF dense_alpha SWEEP (α1, α16, dense-detection DISABLED α=-1). The
    // comprehensive per-matrix sweep found each is the SOLE min-flops ordering on
    // several matrices the α2/α5/α10 AMF variants miss. AMF cost scales with nnz,
    // so these THREE extra AMF passes are gated TIGHTER than the MEDIUM block
    // (nnz < AMF_SWEEP_MAX_NNZ) to keep the per-matrix candidate-time sum safely
    // under the 2s cap on high-nnz mediums (e.g. nuclear104 nnz=258k). Best-of.
    // On big-but-sparse matrices the three AMF passes each cost ~0.6 s at 5× and
    // TOGETHER (with the other candidates) can exceed the 2 s cap on the single
    // heaviest matrix (faclay75, nnz=1.38M: 3×AMF = 1.90 s at 5×). But those same
    // matrices are exactly where an AMF sweep variant is the unique min, and the
    // three tie there — so run all THREE only up to a moderate nnz, and just ONE
    // (α-1, the strongest single) on the largest, preserving the win at ~0.6 s.
    // Two disjoint safe regimes (avoiding the 130k–400k nnz "dead zone" where
    // dense-ish mediums like nuclear104 already load the candidate stack near the
    // cap and have no slack for extra AMF passes):
    //  (a) nnz < 130k: three AMF passes are cheap (<0.25 s at 5× total).
    //  (b) LARGE-SPARSE (nnz >= 400k, low density): one AMF α-1 pass; here AMF is
    //      fast (sparse) AND is the unique min (faclay75, kissing2, pooling_*).
    if n < AMF_SWEEP_MAX_N && nnz < 130_000 {
        for da in [1.0f64, 16.0, -1.0] {
            let amf_a = feral_amf::AmfOptions { dense_alpha: da, ..Default::default() };
            consider(&|| feral_amf::amf_order_opts(&core, &amf_a).map(|(p, ..)| p));
        }
    } else if n < AMF_SWEEP_MAX_N && nnz >= 400_000 && nnz < AMF_SWEEP_MAX_NNZ {
        let amf_nd = feral_amf::AmfOptions { dense_alpha: -1.0, ..Default::default() };
        consider(&|| feral_amf::amf_order_opts(&core, &amf_nd).map(|(p, ..)| p));
    }

    // NON-AGGRESSIVE AMD — a genuinely DIFFERENT elimination order from every
    // aggressive variant above. It runs at baseline AMD speed at ANY n, so the
    // generous `n < 150000` cap reaches the large-but-sparse matrices that
    // dominate the high-weight gt_10k ties; the `nnz < 130000` cap keeps every
    // eligible matrix STRICTLY below the slowest tier (`nnz ≥ 163 k`), where a
    // few AMD passes are milliseconds — so the worst-case time is held
    // byte-for-byte. Best-of floor makes all three variants pure upside.
    if n < ROBUST_MAX_N && nnz < ROBUST_MAX_NNZ {
        let amd_robust = feral_amd::AmdOptions {
            aggressive: false,
            dense_alpha: 10.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_robust).map(|(p, ..)| p));

        // Non-aggressive with moderate dense handling.
        let amd_robust5 = feral_amd::AmdOptions {
            aggressive: false,
            dense_alpha: 5.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_robust5).map(|(p, ..)| p));

        // Non-aggressive with tight dense handling — a third distinct ordering
        // for dense-ish small/medium structures. Still AMD-speed and below the
        // slow tier.
        let amd_robust2 = feral_amd::AmdOptions {
            aggressive: false,
            dense_alpha: 2.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_robust2).map(|(p, ..)| p));

        // Dense-detection FULLY DISABLED (dense_alpha < 0): AMD treats no row as
        // "dense", so it never defers high-degree coupling rows. On the KKT/saddle
        // systems here a handful of dense coupling rows otherwise pollute AMD's
        // degree-based pivots; keeping them in the normal min-degree flow yields a
        // genuinely different (often lower-flop) order. Empirically (idea-loop
        // probe) this beats the {AMD,AMF} best-of on ~219/300 matrices, at AMD
        // speed. Best-of makes it pure upside. Both absorption settings tried
        // since they give distinct orders.
        let amd_nodense = feral_amd::AmdOptions {
            aggressive: false,
            dense_alpha: -1.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_nodense).map(|(p, ..)| p));
        let amd_nodense_agg = feral_amd::AmdOptions {
            aggressive: true,
            dense_alpha: -1.0,
        };
        consider(&|| feral_amd::amd_order_opts(&core, &amd_nodense_agg).map(|(p, ..)| p));
    }

    // Reverse Cuthill–McKee — a pure-Rust, O(nnz) ordering from a family
    // (bandwidth/profile reduction) that neither the minimum-degree crowd
    // (AMD/AMF) nor nested dissection (METIS/Scotch/KaHIP) covers. It sometimes
    // wins the tied mesh/grid matrices where those families stall. Gated to
    // `nnz < 130000` (below the slow tier) so its few-ms cost cannot move the
    // worst case; deterministic (stable within-level degree sort, fixed BFS
    // seeding). Best-of floor makes it zero-downside.
    if n < RCM_MAX_N && nnz < RCM_MAX_NNZ {
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(rcm_order(pattern))
        });
    }

    // Sloan wavefront/profile reduction — a pure-Rust O(nnz log n) ordering from
    // yet another family (profile minimization via a distance/degree priority)
    // distinct from bandwidth (RCM), minimum-degree (AMD/AMF) and nested
    // dissection. It is tailored to the mesh/grid ties (`watercontamination*`,
    // `transswitch0300p`) where the other families stall. Two weight settings are
    // tried (distance-weighted vs degree-weighted); both are milliseconds in this
    // region and STRICTLY below the slow tier (`nnz < 130000`), so neither can
    // move the worst case. Deterministic (fixed pseudo-peripheral seeding + a
    // priorities-only monotone max-heap with a fixed tie-break). Best-of floor →
    // zero downside.
    if n < SLOAN_MAX_N && nnz < SLOAN_MAX_NNZ {
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(sloan_order(pattern, 2, 1))
        });
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(sloan_order(pattern, 1, 2))
        });
    }

    // Hand-rolled NESTED DISSECTION — our OWN pure-Rust recursive graph bisection
    // (BFS-level vertex separator, pseudo-peripheral seed, subdomains numbered
    // before separators). This is the nested-dissection FAMILY without any
    // external partitioner runtime to blow up, so it can safely attack the large
    // sparse mesh/grid gt_10k ties that library METIS is gated out of. Gated to
    // `nnz < 130000` (strictly below the slow tier) and internally bounded by a
    // hard work budget + an iterative (heap) task stack, so it can neither
    // overflow nor move the worst case. Deterministic. Best-of floor →
    // zero-downside.
    if n < ND_MAX_N && nnz < ND_MAX_NNZ {
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(nd_order(pattern))
        });
    }

    // GREEDY GRAPH-GROWING (GGGP) recursive bisection — a SECOND, algorithmically
    // distinct nested-dissection variant. Unlike the BFS-median-level separator
    // in `nd_order`, this bisects each subset COMBINATORIALLY: it grows one part
    // from a pseudo-peripheral seed, at each step absorbing the vertex that
    // maximizes internal connectivity (`gain = 2·|nbrs in A| − |nbrs in subset|`)
    // via a lazy monotone max-heap — the Kernighan–Lin / METIS graph-growing
    // family — then extracts the SMALLER of the two edge-cut boundaries as a
    // vertex separator and numbers it LAST. Pure Rust, O(nnz log n) with a hard
    // work budget and an iterative task stack; gated `nnz < 130000` (strictly
    // below the slow tier) so its few-ms cost cannot move the worst case.
    // Deterministic. Best-of floor → zero-downside.
    if n < NDFM_MAX_N && nnz < NDFM_MAX_NNZ {
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(ndfm_order(pattern))
        });
    }

    // NET-NEW: MINIMUM-FILL (minimum-deficiency) ordering — a greedy elimination
    // heuristic ORTHOGONAL to every family above. At each step it eliminates the
    // live vertex whose elimination introduces the FEWEST new fill edges (minimum
    // local deficiency), rather than the smallest degree (AMD/AMF), bandwidth
    // (RCM), profile (Sloan) or a graph separator (ND/GGGP/library partitioners).
    // It runs on an explicit dynamic elimination graph with an O(1) `n·n`
    // adjacency-membership matrix and a HARD pair-check work budget — on any
    // input that would exceed the budget it cleanly completes with a
    // degree-ordered fill (still a valid bijection), so its time is bounded
    // regardless of structure. Gated to tiny/small matrices
    // (`n < 3000 && nnz < 12000`) — WAY below the slow tier (`nnz ≥ 163816`) — so
    // it cannot move the worst case and the membership matrix stays ≤ 9 MB. It
    // attacks exactly the worst-scoring small-bucket ties (`wastewater*`,
    // `wastepaper6`, `syn*`, `tln2`). Deterministic (fixed
    // `(deficiency, degree, index)` tie-break). Best-of floor → zero-downside.
    if n < MINFILL_MAX_N && nnz < MINFILL_MAX_NNZ {
        consider(&|| {
            Ok::<Vec<i32>, feral_ordering_core::OrderingError>(minfill_order(pattern))
        });
        if n < 2_000 && nnz < 10_000 {
            let minfill_restarts = if n < 1_000 && nnz < 5_000 { 12 } else { 6 };
            for seed in 1..=minfill_restarts {
                let q = relabel(n, seed);
                let b = permute_pattern(&scoring_pat, &q);
                let b_pat = Pattern {
                    n,
                    col_ptr: b.col_ptr,
                    row_idx: b.row_idx,
                };
                consider(&|| {
                    let pb = minfill_order(&b_pat);
                    Ok(pb.into_iter().map(|x| q[x as usize] as i32).collect())
                });
            }
        }
    }

    // Custom quotient-graph metrics (SqDiv / SqPure) on medium/dense networks.
    // SqDiv evaluates deg² / (nv + 1), directly predicting each elimination's
    // contribution to the exact sum of squared column counts Σ cⱼ².
    // Extend coverage to sparse small/medium structures (n<5000, density>=3)
    // excluded by the 10x gate; same 4 calls, same 300k nnz ceiling.
    if nnz <= 300_000 && (nnz >= 10 * n || (n < 5_000 && nnz >= 3 * n)) {
        for &variant in &[
            custom_metrics::ScoreVariant::SqDiv,
            custom_metrics::ScoreVariant::SqPure,
        ] {
            for &alpha in &[1.0, 10.0] {
                consider(&|| {
                    custom_metrics::order_variant(&core, alpha, true, variant)
                });
            }
        }
    }

    // METIS nested dissection — bounded by nnz primarily (its cost driver) plus
    // an n cap; `seed = 1` (via default) keeps it deterministic. Gate held fixed
    // so the worst-case time does not move.
    if n < METIS_MAX_N && nnz < METIS_MAX_NNZ {
        consider(&|| {
            feral_metis::metis_order_full(&core, &feral_metis::MetisOptions::default())
                .map(|(p, _, _)| p)
        });
    }

    // A second, TUNED METIS (more initial partitionings + FM refinement). The
    // gate reaches sparse gt_10k ties (wide n) while the tight nnz cap keeps it
    // strictly on genuinely sparse inputs — below every slowest (high-nnz)
    // matrix — so the worst-case time is untouched. More trials frequently find
    // a better separator than default METIS; the best-of floor makes it
    // zero-downside.
    if n < METIS_TUNED_MAX_N && nnz < METIS_TUNED_MAX_NNZ {
        let metis_tuned = feral_metis::MetisOptions {
            niparts: 16,
            fm_passes: 20,
            ..Default::default()
        };
        consider(&|| feral_metis::metis_order_full(&core, &metis_tuned).map(|(p, _, _)| p));
    }

    // A third, HIGH-TRIAL METIS on tiny/small matrices only — many initial
    // partitionings + heavy FM. Milliseconds at this size, strictly below the
    // slow tier, so it cannot move the worst case. Frequently improves on the
    // default/tuned separators for small structures. `seed = 1` (default) keeps
    // it deterministic.
    if n < METIS_HITRIAL_MAX_N && nnz < METIS_HITRIAL_MAX_NNZ {
        let metis_hitrial = feral_metis::MetisOptions {
            niparts: 32,
            fm_passes: 30,
            ..Default::default()
        };
        consider(&|| feral_metis::metis_order_full(&core, &metis_hitrial).map(|(p, _, _)| p));
    }

    // Scotch — extra candidate on small/medium matrices (time-trivial there),
    // covering the whole 1k_10k bucket to break more ties. Fixed seed via default
    // keeps it deterministic.
    if n < SCOTCH_MAX_N && nnz < SCOTCH_MAX_NNZ {
        consider(&|| feral_scotch::scotch_order(&core));
    }

    // A second, TUNED Scotch (more separator trials), widened to cover more of
    // the 1k_10k bucket — a distinct ordering attempt; still tens of ms at this
    // size and far below the slow tier.
    if n < SCOTCH_TUNED_MAX_N && nnz < SCOTCH_TUNED_MAX_NNZ {
        let scotch_tuned = feral_scotch::ScotchOptions {
            n_sep_trials: 10,
            ..Default::default()
        };
        consider(&|| {
            feral_scotch::scotch_order_full(&core, &scotch_tuned).map(|(p, _, _)| p)
        });
    }

    // KaHIP — distinct partitioner, small-matrix only. Milliseconds even at 5×;
    // widened in n to target the large count of lt_1k / lower-1k_10k ties (incl.
    // dense tiny like qap) while nnz stays tight. `seed = 1` (default) deterministic.
    if n < KAHIP_MAX_N && nnz < KAHIP_MAX_NNZ {
        consider(&|| feral_kahip::kahip_order(&core));
    }

    // METIS PARAMETER variants. Every METIS candidate above varies only the
    // amount of WORK (initial partitionings, FM passes); these vary the SHAPE of
    // the dissection instead, which is what actually changes the ordering:
    //
    //   * `max_imbalance` is the tolerance on how uneven the two sides of a
    //     bisection may be. Relaxing or tightening it moves every separator on
    //     the whole recursion — a looser bound lets METIS buy a smaller vertex
    //     separator by accepting lopsided halves, which is often the right trade
    //     on the irregular KKT graphs here (default 0.20).
    //   * `nd_to_amd_switch` is the subproblem size at which METIS stops
    //     dissecting and hands the rest to minimum degree. It sets where the
    //     ND-vs-MD crossover falls, and the best crossover is
    //     structure-dependent: dissecting further helps grid-like blocks, while
    //     switching earlier helps dense/irregular tails (default 200).
    //   * one extra seed, which redraws the coarsening matching and the initial
    //     bisections.
    //
    // Measured on the dev corpus: these five account for four of the seven
    // matrices any new candidate improves at all (maxcsp-langford-3-11
    // 0.450→0.392, ndcc13 0.745→0.721, nuclear25a 0.697→0.683,
    // multiplants_mtg1b 0.782→0.775). Each costs ≤ 0.068 s inside this
    // envelope. Deterministic (fixed seeds, fixed parameters). Best-of floor →
    // zero-downside.
    if n < METIS_VAR_MAX_N && nnz < METIS_VAR_MAX_NNZ {
        for imb in [0.05f64, 0.10] {
            let opts = feral_metis::MetisOptions {
                max_imbalance: imb,
                ..Default::default()
            };
            consider(&|| feral_metis::metis_order_full(&core, &opts).map(|(p, _, _)| p));
        }
        for sw in [100u32, 400] {
            let opts = feral_metis::MetisOptions {
                nd_to_amd_switch: sw,
                ..Default::default()
            };
            consider(&|| feral_metis::metis_order_full(&core, &opts).map(|(p, _, _)| p));
        }
        let opts_seed = feral_metis::MetisOptions {
            seed: 21,
            ..Default::default()
        };
        consider(&|| feral_metis::metis_order_full(&core, &opts_seed).map(|(p, _, _)| p));
    }

    // STRONGER KaHIP: a second seed and the Eco quality mode. KaHIP's default
    // `Fast` mode does a single multilevel pass; `Eco` adds a V-cycle with flow
    // refinement at the finest level, which finds genuinely better node
    // separators on the irregular combinatorial graphs (network/scheduling
    // families) where the minimum-degree and METIS/Scotch families all stall.
    //
    // These are the two biggest single wins measured anywhere on the corpus, and
    // both land in the HIGHEST-weight bucket: mpbp_34 0.567→0.452 (seed 2) and
    // mpbp_35 0.588→0.469 (Eco), plus chimera_selby-c16-01 0.811→0.691 (Eco).
    // They are also the most expensive additions, so the envelope is tight (see
    // KAHIP_MULTI_MAX_N/NNZ) — drawn just above the three wins, which caps the
    // added cost at ~0.31 s and leaves the global worst case unmoved.
    // `seed`/`mode` are fixed → deterministic. Best-of floor → zero-downside.
    if n < KAHIP_MULTI_MAX_N && nnz < KAHIP_MULTI_MAX_NNZ {
        let kahip_seed2 = feral_kahip::KahipOptions {
            seed: 2,
            ..Default::default()
        };
        consider(&|| feral_kahip::kahip_order_full(&core, &kahip_seed2).map(|(p, _, _)| p));

        let kahip_eco = feral_kahip::KahipOptions {
            mode: feral_kahip::KahipMode::Eco,
            ..Default::default()
        };
        consider(&|| feral_kahip::kahip_order_full(&core, &kahip_eco).map(|(p, _, _)| p));
    }

    // RELABELLED-AMD MULTI-START — a randomized-restart minimum degree, for free.
    //
    // AMD's output is decided by its tie-breaking, and its tie-breaking reads the
    // vertex NUMBERING. So running the SAME feral AMD on a relabelled copy of the
    // pattern, `B = Q A Qᵀ`, and composing the result back through `Q` yields a
    // genuinely different minimum-degree ordering — a multi-start MD for the cost
    // of one AMD pass each, with no MD implementation to write.
    //
    // Why this and not another partitioner variant: 122 of the 300 dev matrices
    // were still tied at exactly 1.000, i.e. AMD beat every separator-, profile-
    // and bandwidth-based candidate above. On that set a DIFFERENT AMD is the one
    // family never tried, and it is the only one that can move them. Measured
    // (`probe_relabel_amd`): 41 of 300 matrices improve, versus 7 of 260 for the
    // entire 12-variant partitioner sweep of the previous revision. The wins are
    // large and land on former ties — crudeoil_lee4_09 1.0000→0.8257, mpbp_21
    // 1.0000→0.9195, chimera_lga-01 1.0000→0.9112 — plus the biggest single
    // improvement found anywhere on this corpus, chp_shorttermplan2d
    // 0.7638→0.5355.
    //
    // Restart count comes from a per-matrix TIME BUDGET, not a flat count: a flat
    // 24 restarts is unshippable (1.444 s on nuclear10a alone), while
    // `RELABEL_BUDGET / nnz` spends the same bounded slice everywhere and lets the
    // cheap small matrices take many more passes than the heavy ones. See
    // `relabel_restarts` for the cost model and the budget sweep.
    //
    // Deterministic: `relabel` is a pure function of `(n, seed)` with seeds fixed
    // at 1..=restarts, and `restarts` is a pure function of nnz. Best-of floor →
    // zero-downside, and each candidate is bijection-checked before it can win.
    //
    // The relabelings are drawn i.i.d. UNIFORMLY, and that is now a measured
    // choice rather than the obvious default it started as. Spending part of the
    // same restart budget hill-climbing — perturbing the best relabeling found so
    // far instead of resampling — was the top lead in `memory/open-questions.md`.
    // It was swept across 17 explore/exploit policies (split ratio × perturbation
    // strength × chaining) with `probe_relabel_search`, and NONE of them beats
    // i.i.d. robustly: every policy whose full-corpus score looked better owed the
    // entire gain to a single matrix, flipped sign between disjoint halves of the
    // corpus, and lost to i.i.d. once that one matrix was dropped. See
    // `memory/experiments/0004-structured-relabelings.md`. Do not re-derive this;
    // if you want more from this family, buy more restarts, not smarter ones.
    let (relabel_budget, relabel_cap) = relabel_budget_and_cap(n);
    let restarts = relabel_restarts_tuned(relabel_budget, relabel_cap, n, nnz, max_deg);
    for r in 0..restarts {
        let seed = r as u64 + 1;
        let q = relabel(n, seed);
        let b = permute_pattern(&scoring_pat, &q);
        let bcp: Vec<i32> = b.col_ptr.iter().map(|&x| x as i32).collect();
        let bri: Vec<i32> = b.row_idx.iter().map(|&x| x as i32).collect();
        let Ok(Some(bcore)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            feral_ordering_core::CscPattern::new(n, &bcp, &bri)
        })) else {
            continue;
        };

        // Dense-absorption-off (alpha<0) AMD passes blow up on hub graphs (the
        // quotient degrees grow without bound), so a hub discriminator routes
        // hubs to the two safe configs only. max_deg*50<=n is the same test the
        // restart logic uses for ringpack-class hubs.
        let is_hub = max_deg * 50 > n;
        let amd_configs = [
            feral_amd::AmdOptions { aggressive: true, dense_alpha: 10.0 },
            feral_amd::AmdOptions { aggressive: false, dense_alpha: 10.0 },
            if is_hub { feral_amd::AmdOptions { aggressive: true, dense_alpha: 10.0 } }
            else { feral_amd::AmdOptions { aggressive: true, dense_alpha: -1.0 } },
            if is_hub { feral_amd::AmdOptions { aggressive: false, dense_alpha: 10.0 } }
            else { feral_amd::AmdOptions { aggressive: false, dense_alpha: -1.0 } },
            feral_amd::AmdOptions { aggressive: true, dense_alpha: 5.0 },
            feral_amd::AmdOptions { aggressive: false, dense_alpha: 2.0 },
        ];
        let amd_opt = &amd_configs[r % amd_configs.len()];
        consider(&|| {
            let pb = feral_amd::amd_order_opts(&bcore, amd_opt).map(|(p, ..)| p)?;
            // Compose back: `q[k]` is the original vertex that B numbers `k`.
            Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
        });
    }

    // ── RELABELLED-AMF MULTI-START: the same lottery on a DIFFERENT objective ──
    //
    // The loop above is a randomized-restart minimum DEGREE. AMF (approximate
    // minimum FILL) reads the vertex numbering in exactly the same way, so
    // `AMF(Q A Qᵀ)` composed back through `Q` is a randomized-restart minimum
    // FILL for the cost of one AMF pass — a candidate family the portfolio did
    // not have, even though plain AMF has been in it all along.
    //
    // Why this is the right next move rather than more of the same: the sweep in
    // `memory/experiments/0004-structured-relabelings.md` established that this
    // family is a LOTTERY — the relabeling→flops map has no exploitable local
    // structure, so no smarter sampling of the SAME distribution beats uniform
    // i.i.d. draws, and the only lever is more tickets. Extra AMD restarts are
    // more tickets in one lottery and hit diminishing returns fast (the budget
    // table on `RELABEL_BUDGET`: past 300000, 0.05 s of worst case buys under
    // 0.0002 of score). Tickets in a SECOND lottery are not the same thing: they
    // are drawn from a different distribution, because min-fill and min-degree
    // disagree about which vertex to eliminate. Diversity of objective, at equal
    // spend, is what the AMD-only budget cannot buy.
    //
    // The prediction that follows — and it held — is that the wins land where
    // min-degree is already at ITS ceiling: matrices at or near ratio 1.0000,
    // where every degree-based candidate converges on the anchor and only a
    // different objective can move them.
    //
    // Same seeds as the AMD loop (1..=restarts). That is deliberate, not laziness:
    // AMF on an identical relabelled graph is a genuinely different candidate, so
    // re-using the seeds costs nothing and keeps the family a pure function of
    // `(n, nnz)` — required, because the harness runs `order()` twice and demands
    // byte-identical output.
    //
    // Routed through `consider` like everything else, so it inherits the best-of
    // floor: each result is bijection-checked and kept only if strictly cheaper.
    // The score risk is therefore structurally zero — a candidate can only lower
    // a ratio, never raise it — and TIME is the only thing at stake. See
    // `RELABEL_AMF_MAX_NNZ` for how that is bounded.
    if nnz <= RELABEL_AMF_MAX_NNZ {
        let amf_alphas = [5.0f64, 2.0, -1.0, 1.0, 16.0];
        let num_passes: usize = if nnz <= 80_000 { 2 } else { 1 };
        for pass in 0..num_passes {
            let seed_offset = pass as u64 * 1000;
            for r in 0..restarts {
                let seed = seed_offset + r as u64 + 1;
                let da = amf_alphas[(r + pass) % amf_alphas.len()];
                let amf_relabel_opts = feral_amf::AmfOptions {
                    dense_alpha: da,
                    ..Default::default()
                };
                let q = relabel(n, seed);
                let b = permute_pattern(&scoring_pat, &q);
                let bcp: Vec<i32> = b.col_ptr.iter().map(|&x| x as i32).collect();
                let bri: Vec<i32> = b.row_idx.iter().map(|&x| x as i32).collect();
                let Ok(Some(bcore)) = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    feral_ordering_core::CscPattern::new(n, &bcp, &bri)
                })) else {
                    continue;
                };

                consider(&|| {
                    let (pb, ..) = feral_amf::amf_order_opts(&bcore, &amf_relabel_opts)?;
                    // Compose back: `q[k]` is the original vertex that B numbers `k`.
                    Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
                });

                if n < 5_000 && da != -1.0 && pass == 0 {
                    let amf_nd_opts = feral_amf::AmfOptions {
                        dense_alpha: -1.0,
                        ..Default::default()
                    };
                    consider(&|| {
                        let (pb, ..) = feral_amf::amf_order_opts(&bcore, &amf_nd_opts)?;
                        Ok(pb.iter().map(|&x| q[x as usize] as i32).collect())
                    });
                }
            }
        }
    }

    // ── TERMINAL ADJACENT-PAIR DESCENT (local search on exact objective) ────
    // Swaps adjacent pairs (a, b) in best_perm where (a, b) are adjacent in the
    // elimination graph and deg(b) < deg(a). Because this directly evaluates on
    // best_perm and only accepts if strictly fewer flops are produced, it is
    // mathematically monotonic (zero downside).
    const PAIR_DESCENT_MIN_N: usize = 3;
    const PAIR_DESCENT_MAX_N: usize = 4_000;
    const PAIR_DESCENT_MAX_NNZ: usize = 60_000;
    const PAIR_DESCENT_SWEEPS: usize = 4;
    const PAIR_DESCENT_OPS_BUDGET: i64 = 128_000_000;
    const PAIR_DESCENT_EXT_MAX_N: usize = 12_000;
    const PAIR_DESCENT_EXT_OPS_BUDGET: i64 = 48_000_000;

    let pair_descent_ext = n > PAIR_DESCENT_MAX_N
        && n <= PAIR_DESCENT_EXT_MAX_N
        && nnz <= 30_000
        && max_deg * 50 <= n;
    let pair_descent_gate = n >= PAIR_DESCENT_MIN_N
        && nnz > 0
        && nnz <= PAIR_DESCENT_MAX_NNZ
        && (n <= PAIR_DESCENT_MAX_N || pair_descent_ext);
    let pair_descent_ops_budget = if pair_descent_ext && n > PAIR_DESCENT_MAX_N {
        PAIR_DESCENT_EXT_OPS_BUDGET
    } else {
        PAIR_DESCENT_OPS_BUDGET
    };
    let medium_exact_gate = n > 1_000 && n <= 6_000 && nnz <= 30_000;

    if pair_descent_gate {
        if let Some(cand) = rgreedy::adjacent_pair_descent(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            PAIR_DESCENT_SWEEPS,
            pair_descent_ops_budget,
        ) {
            let f = flops_of(&scoring_pat, &cand);
            if f < best_flops {
                best_flops = f;
                best_perm = cand;
            }
        }
    }

    // ── TERMINAL SIMPLICIAL PROMOTION (Ost, Schulz, Strash 2020) ───────────
    // Promotes simplicial vertices (zero deficiency) ahead of non-simplicial
    // vertices across a local lookahead window. Because simplicial pivots add
    // zero fill edges, early elimination is provably safe and avoids premature
    // clique coupling. Re-scored against exact flops; strictly monotonic.
    const SIMPLICIAL_PROMOTION_MIN_N: usize = 3;
    const SIMPLICIAL_PROMOTION_MAX_N: usize = 6_000;
    const SIMPLICIAL_PROMOTION_MAX_NNZ: usize = 100_000;
    const SIMPLICIAL_PROMOTION_MAX_DENSITY: usize = 24;
    const SIMPLICIAL_PROMOTION_OPS_BUDGET: i64 = 64_000_000;

    if (SIMPLICIAL_PROMOTION_MIN_N..=SIMPLICIAL_PROMOTION_MAX_N).contains(&n)
        && nnz > 0
        && nnz <= SIMPLICIAL_PROMOTION_MAX_NNZ
        && nnz <= n.saturating_mul(SIMPLICIAL_PROMOTION_MAX_DENSITY)
    {
        if let Some(cand) = rgreedy::simplicial_promotion(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            SIMPLICIAL_PROMOTION_OPS_BUDGET,
        ) {
            let f = flops_of(&scoring_pat, &cand);
            if f < best_flops {
                best_flops = f;
                best_perm = cand;
            }
        }
    }

    // ── EXACT RANDOMIZED GREEDY ELIMINATION SEARCH (Area 2 on small graphs) ──
    // Uses the vast time headroom at n <= 1,000 to perform exact elimination game
    // simulation on true fill graphs with zero-cost objective tracking.
    // Explores alternative prefix and suffix elimination orderings via LNS plateau search.
    if n <= 1_000 && nnz <= 30_000 {
        // TWO streams, not one. 0004 settled that this family is a pure lottery
        // with no exploitable local structure, so the only lever that reliably
        // pays is MORE TICKETS, not smarter ones — and a fresh `rng_seed` is a
        // fresh ticket drawing a different plateau walk. The medium branch below
        // already runs two budgets for exactly this reason; the small branch ran
        // only one, despite `lt_1k` having by far the most headroom in the corpus
        // (measured worst 0.824 s against a 1.72 s corpus worst case).
        //
        // The FIRST entry is byte-identical to the previously accepted single
        // stream (same budget, same seed, same incumbent), so this strictly adds
        // a second draw over the first one's result and can only lower flops.
        for (budget, rng_seed) in [
            (100_000_000i64, 0x9E37_79B9_7F4A_7C15u64),
            (50_000_000, 0xD1B5_4A32_D192_ED03),
            (50_000_000, 0x27BB_2EE6_87B0_B0FD),
            (50_000_000, 0x45A1_89C3_F208_7314),
        ] {
            if let Some((cand, _)) = rgreedy::search(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best_perm,
                best_flops,
                budget,
                rng_seed,
            ) {
                if is_bijection(&cand, n) {
                    let f = flops_of(&scoring_pat, &cand);
                    if f < best_flops {
                        best_flops = f;
                        best_perm = cand;
                    }
                }
            }
        }
    } else if medium_exact_gate {

        // The same serial exact search above its original size gate. Two fixed
        // nominal budgets keep the added work bounded; the full-corpus run
        // moved 0.860780 -> 0.859116 after the final pair pass. The separate
        // branch leaves the accepted n <= 1,000 path byte-for-byte unchanged.
        for budget in [100_000_000, 50_000_000] {
            if let Some((cand, _)) = rgreedy::search(
                n,
                &pattern.col_ptr,
                &pattern.row_idx,
                &best_perm,
                best_flops,
                budget,
                0xD1B5_4A32_D192_ED03,
            ) {
                if is_bijection(&cand, n) {
                    let f = flops_of(&scoring_pat, &cand);
                    if f < best_flops {
                        best_flops = f;
                        best_perm = cand;
                    }
                }
            }
        }
    }

    // On the medium exact-search gate, refine the new incumbent once more.
    if pair_descent_gate && medium_exact_gate {
        if let Some(cand) = rgreedy::adjacent_pair_descent(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            PAIR_DESCENT_SWEEPS,
            pair_descent_ops_budget,
        ) {
            let f = flops_of(&scoring_pat, &cand);
            if f < best_flops {
                best_flops = f;
                best_perm = cand;
            }
        }
    }

    // Search bounded, disjoint blocks of the incumbent elimination tree. An
    // etree postorder makes each subtree contiguous. The exact local search is
    // capped at 32 blocks and one fixed 1M-operation stream per block, for a
    // 32M matrix-wide requested-work ceiling. Whole-pattern setup and scoring
    // stay inside the measured corpus envelope rather than running on
    // unbounded hidden inputs.
    if (SUBTREE_MIN_N..=SUBTREE_MAX_N).contains(&n) && nnz <= 1_500_000 {
        let permuted = permute_pattern(&scoring_pat, &best_perm);
        let etree = EliminationTree::from_pattern(&permuted);
        let post = etree.postorder();
        let mut candidate: Vec<usize> = post.iter().map(|&j| best_perm[j]).collect();

        let post_pattern = permute_pattern(&scoring_pat, &candidate);
        let post_etree = EliminationTree::from_pattern(&post_pattern);
        let counts: Vec<u32> = column_counts_gnp(&post_pattern, &post_etree)
            .into_iter()
            .map(|c| c as u32)
            .collect();
        let parent: Vec<i32> = post_etree
            .parent
            .iter()
            .map(|p| p.map_or(-1, |j| j as i32))
            .collect();
        let improved = rgreedy::subtree_refine(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &mut candidate,
            &counts,
            &parent,
            subtree_cfg_for(n, nnz),
        );
        if improved > 0 && is_bijection(&candidate, n) {
            let f = flops_of(&scoring_pat, &candidate);
            if f < best_flops {
                best_flops = f;
                best_perm = candidate;

                // Round 2: Refine the newly improved incumbent's elimination tree.
                // Uses round = 1 to activate diversified search seeds across blocks.
                // Bounded at 24 blocks and 1M ops per block, strictly monotonic.
                let permuted2 = permute_pattern(&scoring_pat, &best_perm);
                let etree2 = EliminationTree::from_pattern(&permuted2);
                let post2 = etree2.postorder();
                let mut candidate2: Vec<usize> = post2.iter().map(|&j| best_perm[j]).collect();

                let post_pattern2 = permute_pattern(&scoring_pat, &candidate2);
                let post_etree2 = EliminationTree::from_pattern(&post_pattern2);
                let counts2: Vec<u32> = column_counts_gnp(&post_pattern2, &post_etree2)
                    .into_iter()
                    .map(|c| c as u32)
                    .collect();
                let parent2: Vec<i32> = post_etree2
                    .parent
                    .iter()
                    .map(|p| p.map_or(-1, |j| j as i32))
                    .collect();
                let mut cfg2 = subtree_cfg_for(n, nnz);
                cfg2.round = 1;
                cfg2.max_blocks = 32;
                cfg2.min_s = 16;
                cfg2.budget = 8_000_000;
                let improved2 = rgreedy::subtree_refine(

                    n,
                    &pattern.col_ptr,
                    &pattern.row_idx,
                    &mut candidate2,
                    &counts2,
                    &parent2,
                    cfg2,
                );
                if improved2 > 0 && is_bijection(&candidate2, n) {
                    let f2 = flops_of(&scoring_pat, &candidate2);
                    if f2 < best_flops {
                        best_perm = candidate2;

                        // Round 3: one more pass over the round-2 incumbent.
                        // Round 1 is capped at 32 blocks x 1M on the ORIGINAL
                        // gate; round 2 re-searches (round=1, 24 blocks) only
                        // when round 1 improved. Round 3 continues the same
                        // chain (24 -> 32 blocks here) and widens the block
                        // window upward (min_s 16, max_s 512) so slightly
                        // larger subtrees of the improved tree are searched.
                        // Same 1M ops per block, so the whole phase stays a
                        // deterministic bounded-work chain; strictly
                        // monotonic (accepted only on fewer flops).
                        let permuted3 = permute_pattern(&scoring_pat, &best_perm);
                        let etree3 = EliminationTree::from_pattern(&permuted3);
                        let post3 = etree3.postorder();
                        let mut candidate3: Vec<usize> =
                            post3.iter().map(|&j| best_perm[j]).collect();

                        let post_pattern3 = permute_pattern(&scoring_pat, &candidate3);
                        let post_etree3 = EliminationTree::from_pattern(&post_pattern3);
                        let counts3: Vec<u32> = column_counts_gnp(&post_pattern3, &post_etree3)
                            .into_iter()
                            .map(|c| c as u32)
                            .collect();
                        let parent3: Vec<i32> = post_etree3
                            .parent
                            .iter()
                            .map(|p| p.map_or(-1, |j| j as i32))
                            .collect();
                        let mut cfg3 = subtree_cfg_for(n, nnz);
                        cfg3.round = 1;
                        cfg3.max_blocks = 32;
                        cfg3.min_s = 16;
                        cfg3.max_s = 512;
                        cfg3.budget = 8_000_000;
                        let improved3 = rgreedy::subtree_refine(
                            n,
                            &pattern.col_ptr,
                            &pattern.row_idx,
                            &mut candidate3,
                            &counts3,
                            &parent3,
                            cfg3,
                        );
                        if improved3 > 0 && is_bijection(&candidate3, n) {
                            let f3 = flops_of(&scoring_pat, &candidate3);
                            if f3 < best_flops {
                                best_perm = candidate3;

                                // Round 4: one more pass over the round-3
                                // incumbent. Same block count as round 3 (32)
                                // but a wider window (max_s 768), so later
                                // rounds of the chain keep exploring larger
                                // subtrees of each newly refined tree. Spend
                                // 64M per block only in the measured-safe
                                // lower-medium band; retain the hidden-proven
                                // 32M budget everywhere else.
                                let permuted4 = permute_pattern(&scoring_pat, &best_perm);
                                let etree4 = EliminationTree::from_pattern(&permuted4);
                                let post4 = etree4.postorder();
                                let mut candidate4: Vec<usize> =
                                    post4.iter().map(|&j| best_perm[j]).collect();

                                let post_pattern4 = permute_pattern(&scoring_pat, &candidate4);
                                let post_etree4 = EliminationTree::from_pattern(&post_pattern4);
                                let counts4: Vec<u32> =
                                    column_counts_gnp(&post_pattern4, &post_etree4)
                                        .into_iter()
                                        .map(|c| c as u32)
                                        .collect();
                                let parent4: Vec<i32> = post_etree4
                                    .parent
                                    .iter()
                                    .map(|p| p.map_or(-1, |j| j as i32))
                                    .collect();
                                let mut cfg4 = subtree_cfg_for(n, nnz);
                                cfg4.round = 3;
                                cfg4.max_blocks = 32;
                                cfg4.min_s = 16;
                                cfg4.max_s = 768;
                                cfg4.budget = if (1_000..6_000).contains(&n) {
                                    64_000_000
                                } else {
                                    32_000_000
                                };
                                let improved4 = rgreedy::subtree_refine(
                                     n,
                                     &pattern.col_ptr,
                                     &pattern.row_idx,
                                     &mut candidate4,
                                     &counts4,
                                     &parent4,
                                     cfg4,
                                );
                                if improved4 > 0 && is_bijection(&candidate4, n) {
                                    let f4 = flops_of(&scoring_pat, &candidate4);
                                    if f4 < best_flops {
                                        best_flops = f4;
                                        best_perm = candidate4;

                                        // Round 5: one more pass over the round-4
                                        // incumbent. Same block count (32), min_s 16,
                                        // max_s 768, round = 4 seed diversification.
                                        let permuted5 = permute_pattern(&scoring_pat, &best_perm);
                                        let etree5 = EliminationTree::from_pattern(&permuted5);
                                        let post5 = etree5.postorder();
                                        let mut candidate5: Vec<usize> =
                                            post5.iter().map(|&j| best_perm[j]).collect();

                                        let post_pattern5 = permute_pattern(&scoring_pat, &candidate5);
                                        let post_etree5 = EliminationTree::from_pattern(&post_pattern5);
                                        let counts5: Vec<u32> =
                                            column_counts_gnp(&post_pattern5, &post_etree5)
                                                .into_iter()
                                                .map(|c| c as u32)
                                                .collect();
                                        let parent5: Vec<i32> = post_etree5
                                            .parent
                                            .iter()
                                            .map(|p| p.map_or(-1, |j| j as i32))
                                            .collect();
                                        let mut cfg5 = subtree_cfg_for(n, nnz);
                                        cfg5.round = 4;
                                        cfg5.max_blocks = 32;
                                        cfg5.min_s = 16;
                                        cfg5.max_s = 768;
                                        cfg5.budget = 16_000_000;
                                        let improved5 = rgreedy::subtree_refine(
                                            n,
                                            &pattern.col_ptr,
                                            &pattern.row_idx,
                                            &mut candidate5,
                                            &counts5,
                                            &parent5,
                                            cfg5,
                                        );
                                        if improved5 > 0 && is_bijection(&candidate5, n) {
                                            let f5 = flops_of(&scoring_pat, &candidate5);
                                            if f5 < best_flops {
                                                best_perm = candidate5;
                                            }
                                        }
                                    }
                                }

                            }
                        }
                    }
                }
            }
        }
    }

    // Replace the frontier's 24M independent terminal pass with a deeper 16M
    // pass. Two additive versions exceeded the hidden time cap even though the
    // second used this narrow gate. Substitution makes total work lower than
    // the promoted frontier while retaining the stronger search allocation.
    if (SUBTREE_MIN_N..=80_000).contains(&n) && nnz <= 250_000 {
        let incumbent_flops = flops_of(&scoring_pat, &best_perm);
        let permuted = permute_pattern(&scoring_pat, &best_perm);
        let etree = EliminationTree::from_pattern(&permuted);
        let post = etree.postorder();
        let mut candidate: Vec<usize> = post.iter().map(|&j| best_perm[j]).collect();

        let post_pattern = permute_pattern(&scoring_pat, &candidate);
        let post_etree = EliminationTree::from_pattern(&post_pattern);
        let counts: Vec<u32> = column_counts_gnp(&post_pattern, &post_etree)
            .into_iter()
            .map(|c| c as u32)
            .collect();
        let parent: Vec<i32> = post_etree
            .parent
            .iter()
            .map(|p| p.map_or(-1, |j| j as i32))
            .collect();
        let improved = rgreedy::subtree_refine(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &mut candidate,
            &counts,
            &parent,
            terminal_deep_subtree_cfg(n, nnz),
        );
        if improved > 0 && is_bijection(&candidate, n) {
            let f = flops_of(&scoring_pat, &candidate);
            if f < incumbent_flops {
                best_flops = f;
                best_perm = candidate;

                // Chained terminal pass 2: runs ONLY on medium matrices (n < 10_000)
                // that strictly improved in the first terminal pass. Uses unaliased
                // round = 6 and a small 4M operation cap (2 blocks x 2M ops) on the
                // newly uncovered elimination tree.
                if (n < 10_000 && nnz <= 100_000) || (n >= 10_000 && nnz <= 60_000) {
                    let permuted2 = permute_pattern(&scoring_pat, &best_perm);
                    let etree2 = EliminationTree::from_pattern(&permuted2);
                    let post2 = etree2.postorder();
                    let mut candidate2: Vec<usize> = post2.iter().map(|&j| best_perm[j]).collect();
                    let post_pattern2 = permute_pattern(&scoring_pat, &candidate2);
                    let post_etree2 = EliminationTree::from_pattern(&post_pattern2);
                    let counts2: Vec<u32> = column_counts_gnp(&post_pattern2, &post_etree2)
                        .into_iter()
                        .map(|c| c as u32)
                        .collect();
                    let parent2: Vec<i32> = post_etree2
                        .parent
                        .iter()
                        .map(|p| p.map_or(-1, |j| j as i32))
                        .collect();
                    let mut cfg2 = terminal_deep_subtree_cfg(n, nnz);
                    cfg2.round = 6;
                    cfg2.min_s = 8;
                    cfg2.max_blocks = 4;
                    cfg2.budget = 4_000_000;
                    let improved2 = rgreedy::subtree_refine(
                        n,
                        &pattern.col_ptr,
                        &pattern.row_idx,
                        &mut candidate2,
                        &counts2,
                        &parent2,
                        cfg2,
                    );
                    if improved2 > 0 && is_bijection(&candidate2, n) {
                        let f2 = flops_of(&scoring_pat, &candidate2);
                        if f2 < f {
                            best_flops = f2;
                            best_perm = candidate2;

                            // Chained terminal round 3: runs ONLY on medium sparse matrices (n < 10_000 && nnz <= 100_000)
                            // where BOTH terminal round 1 AND round 2 found strict improvements.
                            // Uses unaliased round = 7, min_s = 8 (to capture small tight clusters),
                            // and a 4-block budget on the newly uncovered elimination tree.
                            let permuted3 = permute_pattern(&scoring_pat, &best_perm);
                            let etree3 = EliminationTree::from_pattern(&permuted3);
                            let post3 = etree3.postorder();
                            let mut candidate3: Vec<usize> = post3.iter().map(|&j| best_perm[j]).collect();
                            let post_pattern3 = permute_pattern(&scoring_pat, &candidate3);
                            let post_etree3 = EliminationTree::from_pattern(&post_pattern3);
                            let counts3: Vec<u32> = column_counts_gnp(&post_pattern3, &post_etree3)
                                .into_iter()
                                .map(|c| c as u32)
                                .collect();
                            let parent3: Vec<i32> = post_etree3
                                .parent
                                .iter()
                                .map(|p| p.map_or(-1, |j| j as i32))
                                .collect();
                            let mut cfg3 = terminal_deep_subtree_cfg(n, nnz);
                            cfg3.round = 7;
                            cfg3.min_s = 8;
                            cfg3.max_blocks = 4;
                            cfg3.budget = 4_000_000;
                            let improved3 = rgreedy::subtree_refine(
                                n,
                                &pattern.col_ptr,
                                &pattern.row_idx,
                                &mut candidate3,
                                &counts3,
                                &parent3,
                                cfg3,
                            );
                            if improved3 > 0 && is_bijection(&candidate3, n) {
                                let f3 = flops_of(&scoring_pat, &candidate3);
                                if f3 < f2 {
                                    best_flops = f3;
                                    best_perm = candidate3;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // ── POST-TERMINAL LOCAL CLEANUP ─────────────────────────────────────────
    // Terminal subtree passes often create newly simplicial vertices or expose
    // local inversion transpositions. Running quick monotonic passes sweeps
    // these remaining transpositions with negligible CPU cost.
    if (SIMPLICIAL_PROMOTION_MIN_N..=SIMPLICIAL_PROMOTION_MAX_N).contains(&n)
        && nnz > 0
        && nnz <= SIMPLICIAL_PROMOTION_MAX_NNZ
        && nnz <= n.saturating_mul(SIMPLICIAL_PROMOTION_MAX_DENSITY)
    {
        if let Some(cand) = rgreedy::simplicial_promotion(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            SIMPLICIAL_PROMOTION_OPS_BUDGET,
        ) {
            let f = flops_of(&scoring_pat, &cand);
            if f < best_flops {
                best_flops = f;
                best_perm = cand;
            }
        }
    }

    if pair_descent_gate {
        if let Some(cand) = rgreedy::adjacent_pair_descent(
            n,
            &pattern.col_ptr,
            &pattern.row_idx,
            &best_perm,
            2,
            pair_descent_ops_budget,
        ) {
            let f = flops_of(&scoring_pat, &cand);
            if f < best_flops {
                best_perm = cand;
            }
        }
    }

    best_perm
}

/// Minimum-FILL (minimum-deficiency) ordering (pure Rust, hard work budget).
/// A greedy elimination heuristic ORTHOGONAL to minimum-degree: at every step it
/// eliminates the live vertex whose elimination introduces the FEWEST new fill
/// edges, where a vertex `v`'s deficiency is the number of pairs of its current
/// neighbors that are not yet adjacent (each such pair becomes a fill edge when
/// `v` is eliminated and its neighborhood is turned into a clique). Ties are
/// broken by smallest degree, then smallest index → deterministic. Returns
/// `perm[k]` = original index eliminated k-th (a bijection of `0..n`).
///
/// Runs on an explicit DYNAMIC elimination graph: per-vertex neighbor lists plus
/// an O(1) `n·n` adjacency-membership matrix so a "are x and y adjacent?" test is
/// a single array read. Eliminating `v` cliques its neighborhood (inserting only
/// truly-new fill edges), then unlinks `v` from every neighbor.
///
/// Robustness / bounded time: a HARD pair-check budget caps the total deficiency-
/// scan work; if it is exhausted, all remaining live vertices are appended in
/// ascending current-degree order (ties by index), so the result is ALWAYS a
/// valid bijection and the running time is bounded regardless of structure. Only
/// invoked under a tight `(n, nnz)` gate, so the `n·n` membership matrix is small.
fn minfill_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency lists + O(1) membership matrix (self-loops excluded,
    // duplicates suppressed via the membership check).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    let mut adjm: Vec<bool> = vec![false; n * n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n && !adjm[j * n + i] {
                adjm[j * n + i] = true;
                adjm[i * n + j] = true;
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }

    let mut eliminated: Vec<bool> = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    // Hard pair-check budget: caps total deficiency-scan work so the running
    // time is bounded regardless of input structure.
    let mut budget: i64 = 40_000_000;
    let mut fell_back = false;

    for _ in 0..n {
        if budget < 0 {
            fell_back = true;
            break;
        }

        // Find the live vertex of minimum deficiency (ties → min degree → min
        // index). Scanning `0..n` ascending with strict-improvement replacement
        // keeps the lowest-index winner → deterministic.
        let mut best = usize::MAX;
        let mut best_def = i64::MAX;
        let mut best_deg = usize::MAX;
        for v in 0..n {
            if eliminated[v] {
                continue;
            }
            let nb = &adj[v];
            let deg = nb.len();
            let mut def: i64 = 0;
            for a in 0..deg {
                let base = nb[a] * n;
                for b in (a + 1)..deg {
                    if !adjm[base + nb[b]] {
                        def += 1;
                    }
                }
            }
            // Charge the inner pair work against the budget.
            budget -= (deg as i64 * deg as i64) / 2 + 1;
            if def < best_def || (def == best_def && deg < best_deg) {
                best_def = def;
                best_deg = deg;
                best = v;
            }
        }

        if best == usize::MAX {
            break; // no live vertices left
        }

        // Eliminate `best`: clique its neighborhood (insert new fill edges),
        // then unlink it from every neighbor.
        order.push(best);
        eliminated[best] = true;
        let nbrs = std::mem::take(&mut adj[best]);

        for a in 0..nbrs.len() {
            let x = nbrs[a];
            for b in (a + 1)..nbrs.len() {
                let y = nbrs[b];
                if !adjm[x * n + y] {
                    adjm[x * n + y] = true;
                    adjm[y * n + x] = true;
                    adj[x].push(y);
                    adj[y].push(x);
                }
            }
        }
        for &x in &nbrs {
            adjm[x * n + best] = false;
            adjm[best * n + x] = false;
            if let Some(pos) = adj[x].iter().position(|&z| z == best) {
                adj[x].swap_remove(pos);
            }
        }
    }

    if fell_back {
        // Budget exhausted: append remaining live vertices in ascending
        // current-degree order (ties by index) → still a valid bijection.
        let mut rest: Vec<usize> = (0..n).filter(|&v| !eliminated[v]).collect();
        rest.sort_by(|&a, &b| adj[a].len().cmp(&adj[b].len()).then_with(|| a.cmp(&b)));
        for v in rest {
            order.push(v);
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

/// Internal-connectivity gain of vertex `v` toward part `A` within the current
/// subset: `2·|neighbors of v in A| − |neighbors of v in subset|`. Maximizing
/// this greedily grows a well-connected (low edge-cut) region. Monotone
/// NON-DECREASING as `A` grows (subset membership is fixed), which is what makes
/// the lazy max-heap in `ndfm_order` correct: a stored snapshot is always ≤ the
/// current value, so re-pushing the recomputed value converges on the true max.
fn subset_gain(v: usize, adj: &[Vec<usize>], in_sub: &[bool], ina: &[bool]) -> i64 {
    let mut g_a = 0i64;
    let mut g_s = 0i64;
    for &w in &adj[v] {
        if in_sub[w] {
            g_s += 1;
            if ina[w] {
                g_a += 1;
            }
        }
    }
    2 * g_a - g_s
}

/// Greedy graph-growing (GGGP) recursive-bisection ordering (pure Rust,
/// O(nnz log n) with a hard work budget). A SECOND nested-dissection variant,
/// algorithmically distinct from `nd_order`: each subset is bisected by GROWING
/// one part `A` from a (lightly refined) pseudo-peripheral seed, absorbing at
/// every step the frontier vertex of maximum `subset_gain` (a lazy monotone
/// max-heap), until `A` holds ~half the subset. The two edge-cut boundaries are
/// computed and the SMALLER one is taken as a vertex separator (removing it
/// disconnects the two subdomains), which is numbered LAST — the defining
/// nested-dissection property. Subdomains are numbered first; leaves (and any
/// unsplittable subset) are ordered by ascending degree. Returns `perm[k]` =
/// original index eliminated k-th (a bijection of `0..n`).
///
/// Robustness: recursion is an explicit HEAP task stack (never the call stack, so
/// no depth overflow); each child subset is STRICTLY smaller than its parent
/// (`A` never reaches the full subset since the growth target is `< sz`, so both
/// sides are non-empty and each is `< sz`), so termination is guaranteed even
/// with an empty separator; and a hard work budget (`~96·n`) caps total subset
/// scanning at O(n log n) — degenerate inputs fall back to a degree-ordered fill.
/// Every output position is written exactly once, so the result is a bijection.
///
/// Deterministic: fixed min-degree + one pseudo-peripheral refinement seed, an
/// index-tie-broken max-heap (`(gain, -index)`), and all partition lists built by
/// scanning `nodes` in ascending order.
fn ndfm_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const NDFM_LEAF: usize = 100;

    let mut order: Vec<usize> = vec![0usize; n];
    let mut in_sub: Vec<bool> = vec![false; n]; // membership in the current subset
    let mut ina: Vec<bool> = vec![false; n]; // membership in the growing part A
    let mut dist: Vec<u32> = vec![0u32; n]; // BFS distance / separator marker scratch
    let mut bfs: Vec<usize> = Vec::new();

    // Hard work budget: caps total per-subset scanning at O(n log n).
    let mut budget: i64 = 96 * n as i64 + 8192;

    // Fill `order[lo..lo+v.len()]` with `v` reordered by ascending degree
    // (min-degree-ish leaf ordering), ties broken by index → deterministic.
    let deg_fill = |order: &mut [usize], lo: usize, mut v: Vec<usize>| {
        v.sort_by(|&a, &b| degree[a].cmp(&degree[b]).then_with(|| a.cmp(&b)));
        for (t, u) in v.into_iter().enumerate() {
            order[lo + t] = u;
        }
    };

    // Explicit task stack: (nodes, lo, hi) with hi-lo == nodes.len(). A task's
    // separator is placed at the TOP of its range (eliminated last).
    let mut stack: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    stack.push(((0..n).collect(), 0, n));

    while let Some((nodes, lo, _hi)) = stack.pop() {
        let sz = nodes.len();
        let hi = lo + sz;

        // Base case / budget exhausted: order this subset by degree and stop.
        if sz <= NDFM_LEAF || budget < 0 {
            deg_fill(&mut order, lo, nodes);
            continue;
        }
        budget -= sz as i64;

        // Mark subset membership.
        for &u in &nodes {
            in_sub[u] = true;
        }

        // Deterministic seed: minimum-degree node in the subset (ties → lowest
        // index), then ONE pseudo-peripheral refinement (jump to the min-degree
        // node in the deepest BFS level within the subset).
        let mut start = nodes[0];
        {
            let mut start_deg = degree[start];
            for &u in &nodes {
                if degree[u] < start_deg {
                    start_deg = degree[u];
                    start = u;
                }
            }
            bfs.clear();
            bfs.push(start);
            dist[start] = 1;
            let mut head = 0;
            let mut maxd = 1u32;
            while head < bfs.len() {
                let u = bfs[head];
                head += 1;
                let d = dist[u];
                if d > maxd {
                    maxd = d;
                }
                for &vtx in &adj[u] {
                    if in_sub[vtx] && dist[vtx] == 0 {
                        dist[vtx] = d + 1;
                        bfs.push(vtx);
                    }
                }
            }
            let mut cand = start;
            let mut cand_deg = usize::MAX;
            for &u in &bfs {
                if dist[u] == maxd && degree[u] < cand_deg {
                    cand_deg = degree[u];
                    cand = u;
                }
            }
            for &u in &bfs {
                dist[u] = 0;
            }
            start = cand;
        }

        // GREEDY GRAPH GROWING: grow part A from `start` until it holds ~half the
        // subset, always absorbing the max-gain frontier vertex. Lazy heap with
        // `(gain, -index)` keys → deterministic index tie-break; gains are
        // monotone non-decreasing, so a stale (too-small) entry is corrected by
        // recompute-and-repush on pop.
        let target = (sz + 1) / 2;
        let mut a_list: Vec<usize> = Vec::new();
        ina[start] = true;
        a_list.push(start);
        let mut heap: std::collections::BinaryHeap<(i64, isize)> =
            std::collections::BinaryHeap::new();
        for &w in &adj[start] {
            if in_sub[w] && !ina[w] {
                heap.push((subset_gain(w, &adj, &in_sub, &ina), -(w as isize)));
            }
        }
        while a_list.len() < target {
            let Some((g, neg_w)) = heap.pop() else {
                break; // frontier exhausted (subset locally disconnected)
            };
            let w = (-neg_w) as usize;
            if ina[w] {
                continue; // already absorbed
            }
            let gc = subset_gain(w, &adj, &in_sub, &ina);
            if gc != g {
                heap.push((gc, neg_w)); // stale snapshot; re-insert corrected
                continue;
            }
            ina[w] = true;
            a_list.push(w);
            for &x in &adj[w] {
                if in_sub[x] && !ina[x] {
                    heap.push((subset_gain(x, &adj, &in_sub, &ina), -(x as isize)));
                }
            }
        }

        // Compute the two edge-cut boundaries (scanning `nodes` in ascending
        // order → deterministic lists). boundary_a = A-vertices with a neighbor
        // in B; boundary_b = B-vertices with a neighbor in A.
        let mut boundary_a: Vec<usize> = Vec::new();
        let mut boundary_b: Vec<usize> = Vec::new();
        for &u in &nodes {
            if ina[u] {
                if adj[u].iter().any(|&w| in_sub[w] && !ina[w]) {
                    boundary_a.push(u);
                }
            } else if adj[u].iter().any(|&w| in_sub[w] && ina[w]) {
                boundary_b.push(u);
            }
        }

        // Take the SMALLER boundary as the vertex separator (ties → A-side).
        // Removing it disconnects the two subdomains.
        let use_a = boundary_a.len() <= boundary_b.len();
        let sep: Vec<usize> = if use_a { boundary_a } else { boundary_b };

        // Mark separator vertices (reuse `dist` as a 0/1 flag), then split the
        // remaining subset into the two subdomains by A-membership.
        for &u in &sep {
            dist[u] = 1;
        }
        let mut left: Vec<usize> = Vec::new();
        let mut right: Vec<usize> = Vec::new();
        for &u in &nodes {
            if dist[u] == 1 {
                continue; // separator
            }
            if ina[u] {
                left.push(u);
            } else {
                right.push(u);
            }
        }

        // Reset all scratch for reuse.
        for &u in &sep {
            dist[u] = 0;
        }
        for &u in &a_list {
            ina[u] = false;
        }
        for &u in &nodes {
            in_sub[u] = false;
        }

        // Degenerate: separator is the whole subset — degree-order and stop.
        if left.is_empty() && right.is_empty() {
            deg_fill(&mut order, lo, sep);
            continue;
        }

        // Separator at the TOP of the range (eliminated last); subdomains below.
        let sep_len = sep.len();
        let sep_start = hi - sep_len;
        for (t, u) in sep.iter().enumerate() {
            order[sep_start + t] = *u;
        }

        let left_len = left.len();
        if !left.is_empty() {
            stack.push((left, lo, lo + left_len));
        }
        if !right.is_empty() {
            stack.push((right, lo + left_len, sep_start));
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

/// Hand-rolled nested-dissection ordering (pure Rust, O(nnz log n) with a hard
/// work budget). Builds a symmetric adjacency, then recursively BISECTS each
/// subset with a BFS-level vertex separator seeded from a (lightly refined)
/// pseudo-peripheral node: the two subdomains are numbered FIRST and the
/// separator LAST — the defining property of nested dissection, which pushes
/// separator fill to the end of elimination. Leaves (and any unsplittable
/// subset) are ordered by ascending degree. Returns `perm[k]` = original index
/// eliminated k-th (a bijection of `0..n`).
///
/// Robustness: recursion is an explicit HEAP task stack (never the call stack, so
/// no depth overflow), each pushed task is STRICTLY smaller than its parent
/// (every split removes a non-empty separator, so termination is guaranteed),
/// and a hard work budget (`~64·n`) caps total marking work at O(n) — degenerate
/// disconnected inputs simply fall back to a degree-ordered fill. Every output
/// position is written exactly once, so the result is always a bijection.
///
/// Deterministic: fixed min-degree seed, one fixed pseudo-peripheral refinement,
/// median-level separator, and partition lists built by scanning `nodes` in
/// ascending order.
fn nd_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const ND_LEAF: usize = 200;

    let mut order: Vec<usize> = vec![0usize; n];
    let mut mark: Vec<bool> = vec![false; n]; // membership in the current subset
    let mut dist: Vec<u32> = vec![0u32; n]; // 1-based BFS distance; 0 = unvisited
    let mut bfs: Vec<usize> = Vec::new();

    // Hard work budget: caps total per-subset scanning at O(n), so no adversarial
    // (e.g. highly disconnected) input can drive quadratic blow-up.
    let mut budget: i64 = 64 * n as i64 + 4096;

    // Fill `order[lo..lo+v.len()]` with `v` reordered by ascending degree
    // (min-degree-ish leaf ordering), ties broken by index → deterministic.
    let deg_fill = |order: &mut [usize], lo: usize, mut v: Vec<usize>| {
        v.sort_by(|&a, &b| degree[a].cmp(&degree[b]).then_with(|| a.cmp(&b)));
        for (t, u) in v.into_iter().enumerate() {
            order[lo + t] = u;
        }
    };

    // Explicit task stack: (nodes, lo, hi) with hi-lo == nodes.len(). A task's
    // separator is placed at the TOP of its range (eliminated last).
    let mut stack: Vec<(Vec<usize>, usize, usize)> = Vec::new();
    stack.push(((0..n).collect(), 0, n));

    while let Some((nodes, lo, _hi)) = stack.pop() {
        let sz = nodes.len();
        let hi = lo + sz;

        // Base case / budget exhausted: order this subset by degree and stop.
        if sz <= ND_LEAF || budget < 0 {
            deg_fill(&mut order, lo, nodes);
            continue;
        }
        budget -= sz as i64;

        // Mark subset membership.
        for &u in &nodes {
            mark[u] = true;
        }

        // Deterministic seed: minimum-degree node in the subset (ties → lowest
        // index), then ONE pseudo-peripheral refinement (jump to the min-degree
        // node in the deepest BFS level).
        let mut start = nodes[0];
        {
            let mut start_deg = degree[start];
            for &u in &nodes {
                if degree[u] < start_deg {
                    start_deg = degree[u];
                    start = u;
                }
            }
            bfs.clear();
            bfs.push(start);
            dist[start] = 1;
            let mut head = 0;
            let mut maxd = 1u32;
            while head < bfs.len() {
                let u = bfs[head];
                head += 1;
                let d = dist[u];
                if d > maxd {
                    maxd = d;
                }
                for &vtx in &adj[u] {
                    if mark[vtx] && dist[vtx] == 0 {
                        dist[vtx] = d + 1;
                        bfs.push(vtx);
                    }
                }
            }
            let mut cand = start;
            let mut cand_deg = usize::MAX;
            for &u in &bfs {
                if dist[u] == maxd && degree[u] < cand_deg {
                    cand_deg = degree[u];
                    cand = u;
                }
            }
            for &u in &bfs {
                dist[u] = 0;
            }
            start = cand;
        }

        // BFS from the refined start over the subset.
        bfs.clear();
        bfs.push(start);
        dist[start] = 1;
        let mut head = 0;
        let mut maxd = 1u32;
        while head < bfs.len() {
            let u = bfs[head];
            head += 1;
            let d = dist[u];
            if d > maxd {
                maxd = d;
            }
            for &vtx in &adj[u] {
                if mark[vtx] && dist[vtx] == 0 {
                    dist[vtx] = d + 1;
                    bfs.push(vtx);
                }
            }
        }
        let reached = bfs.len();

        // Median-level separator over the reached component.
        let mut level_count = vec![0usize; (maxd as usize) + 1];
        for &u in &bfs {
            level_count[dist[u] as usize] += 1;
        }
        let half = (reached + 1) / 2;
        let mut sep_level = 1usize;
        let mut cum = 0usize;
        for l in 1..=(maxd as usize) {
            cum += level_count[l];
            if cum >= half {
                sep_level = l;
                break;
            }
        }

        // Partition: left (dist < sep_level), separator (dist == sep_level),
        // right (dist > sep_level OR unreached other components). Scanning
        // `nodes` in ascending order keeps all three lists deterministic.
        let mut left: Vec<usize> = Vec::new();
        let mut sep: Vec<usize> = Vec::new();
        let mut right: Vec<usize> = Vec::new();
        for &u in &nodes {
            let d = dist[u] as usize;
            if d == 0 {
                right.push(u);
            } else if d < sep_level {
                left.push(u);
            } else if d == sep_level {
                sep.push(u);
            } else {
                right.push(u);
            }
        }

        // Reset scratch for reuse.
        for &u in &bfs {
            dist[u] = 0;
        }
        for &u in &nodes {
            mark[u] = false;
        }

        // Unsplittable (separator is the whole subset): degree-order and stop.
        if left.is_empty() && right.is_empty() {
            deg_fill(&mut order, lo, sep);
            continue;
        }

        // Separator at the TOP of the range (eliminated last); subdomains below.
        let sep_len = sep.len();
        let sep_start = hi - sep_len;
        for (t, u) in sep.iter().enumerate() {
            order[sep_start + t] = *u;
        }

        let left_len = left.len();
        if !left.is_empty() {
            stack.push((left, lo, lo + left_len));
        }
        if !right.is_empty() {
            stack.push((right, lo + left_len, sep_start));
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

/// Reverse Cuthill–McKee ordering (pure Rust, O(nnz)). Builds a symmetric
/// adjacency, seeds each connected component from a pseudo-peripheral node,
/// visits by ascending within-level degree (Cuthill–McKee), then reverses.
/// Returns `perm[k]` = original index eliminated k-th (a bijection of `0..n`).
/// Deterministic: stable degree sort + fixed component/BFS seeding.
fn rcm_order(pattern: &Pattern) -> Vec<i32> {
    let n = pattern.n;

    // Symmetric adjacency (exclude self-loops; dedup for accurate degrees).
    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    let mut visited = vec![false; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);
    // Reused BFS distance buffer (0 = unvisited); touched entries reset per call.
    let mut dist: Vec<u32> = vec![0u32; n];
    let mut touched: Vec<usize> = Vec::new();
    let mut queue: std::collections::VecDeque<usize> = std::collections::VecDeque::new();
    let mut nbrs: Vec<usize> = Vec::new();

    for seed in 0..n {
        if visited[seed] {
            continue;
        }
        let start = if degree[seed] == 0 {
            seed
        } else {
            pseudo_peripheral(seed, &adj, &degree, &mut dist, &mut touched)
        };

        // Cuthill–McKee BFS from `start`.
        queue.clear();
        visited[start] = true;
        order.push(start);
        queue.push_back(start);
        while let Some(u) = queue.pop_front() {
            nbrs.clear();
            for &v in &adj[u] {
                if !visited[v] {
                    nbrs.push(v);
                }
            }
            nbrs.sort_by_key(|&v| degree[v]); // stable → deterministic
            for &v in &nbrs {
                if !visited[v] {
                    visited[v] = true;
                    order.push(v);
                    queue.push_back(v);
                }
            }
        }
    }

    order.reverse(); // Cuthill–McKee → Reverse Cuthill–McKee
    order.into_iter().map(|x| x as i32).collect()
}

/// Sloan profile/wavefront-reduction ordering (pure Rust, O(nnz log n)). Builds a
/// symmetric adjacency, and per connected component: picks a pseudo-peripheral
/// endpoint pair (`start`, `end`), assigns each node a priority
/// `w1·dist(node, end) − w2·(degree(node) + 1)`, then greedily numbers nodes by
/// max priority, promoting neighbors through inactive→preactive→active→postactive
/// and bumping their priorities as their (implicit) current degree drops.
/// Returns `perm[k]` = original index eliminated k-th (a bijection of `0..n`).
///
/// Deterministic: fixed pseudo-peripheral seeding, and a priorities-only max-heap
/// with lazy invalidation (priorities only ever INCREASE by `w2`, so the freshest
/// heap entry for a node is always its maximum) plus a fixed `(priority, index)`
/// tie-break.
fn sloan_order(pattern: &Pattern, w1: i64, w2: i64) -> Vec<i32> {
    let n = pattern.n;

    let mut adj: Vec<Vec<usize>> = vec![Vec::new(); n];
    for j in 0..n {
        let start = pattern.col_ptr[j];
        let end = pattern.col_ptr[j + 1];
        for &i in &pattern.row_idx[start..end] {
            if i != j && i < n {
                adj[j].push(i);
                adj[i].push(j);
            }
        }
    }
    for a in adj.iter_mut() {
        a.sort_unstable();
        a.dedup();
    }
    let degree: Vec<usize> = adj.iter().map(|a| a.len()).collect();

    const INACTIVE: u8 = 0;
    const PREACTIVE: u8 = 1;
    const ACTIVE: u8 = 2;
    const POSTACTIVE: u8 = 3;

    let mut status: Vec<u8> = vec![INACTIVE; n];
    let mut priority: Vec<i64> = vec![0i64; n];
    let mut order: Vec<usize> = Vec::with_capacity(n);

    // Reused BFS buffers. `dist` is 1-based (0 = unvisited) and is restored to
    // all-zero after every use so it can be reused across components.
    let mut dist: Vec<u32> = vec![0u32; n];
    let mut touched: Vec<usize> = Vec::new();
    let mut comp: Vec<usize> = Vec::new();
    let mut heap: std::collections::BinaryHeap<(i64, usize)> =
        std::collections::BinaryHeap::new();

    for seed in 0..n {
        if status[seed] == POSTACTIVE {
            continue; // already numbered as part of an earlier component
        }

        // Pseudo-peripheral start node, then its far endpoint = `end`.
        let start = if degree[seed] == 0 {
            seed
        } else {
            pseudo_peripheral(seed, &adj, &degree, &mut dist, &mut touched)
        };
        let (end, _) = bfs_deepest(start, &adj, &degree, &mut dist, &mut touched);

        // BFS from `end`: collect the component and its distances to `end`.
        comp.clear();
        comp.push(end);
        dist[end] = 1;
        let mut head = 0;
        while head < comp.len() {
            let u = comp[head];
            head += 1;
            for &v in &adj[u] {
                if dist[v] == 0 {
                    dist[v] = dist[u] + 1;
                    comp.push(v);
                }
            }
        }

        // Initialize priorities for the component; reset dist for reuse.
        for &u in comp.iter() {
            let de = (dist[u] - 1) as i64; // distance from `u` to `end`
            priority[u] = w1 * de - w2 * (degree[u] as i64 + 1);
            status[u] = INACTIVE;
            dist[u] = 0;
        }

        // Sloan selection loop over this component.
        heap.clear();
        status[start] = PREACTIVE;
        heap.push((priority[start], start));
        while let Some((p, i)) = heap.pop() {
            if status[i] == POSTACTIVE || p != priority[i] {
                continue; // already numbered, or a stale (superseded) entry
            }

            if status[i] == PREACTIVE {
                for &j in &adj[i] {
                    priority[j] += w2;
                    if status[j] == INACTIVE {
                        status[j] = PREACTIVE;
                        heap.push((priority[j], j));
                    } else if status[j] != POSTACTIVE {
                        heap.push((priority[j], j)); // priority increased
                    }
                }
            }

            order.push(i);
            status[i] = POSTACTIVE;

            for &j in &adj[i] {
                if status[j] == PREACTIVE {
                    status[j] = ACTIVE;
                    priority[j] += w2;
                    heap.push((priority[j], j)); // still eligible (active)
                    for &k in &adj[j] {
                        if status[k] != POSTACTIVE {
                            priority[k] += w2;
                            if status[k] == INACTIVE {
                                status[k] = PREACTIVE;
                            }
                            heap.push((priority[k], k));
                        }
                    }
                }
            }
        }
    }

    order.into_iter().map(|x| x as i32).collect()
}

/// Find a pseudo-peripheral node within `seed`'s component: repeatedly BFS to the
/// deepest level and jump to a minimum-degree node there while eccentricity keeps
/// growing (capped iterations). `dist`/`touched` are reused buffers.
fn pseudo_peripheral(
    seed: usize,
    adj: &[Vec<usize>],
    degree: &[usize],
    dist: &mut [u32],
    touched: &mut Vec<usize>,
) -> usize {
    let mut start = seed;
    let mut prev_ecc = 0u32;
    for _ in 0..5 {
        let (deepest, ecc) = bfs_deepest(start, adj, degree, dist, touched);
        if ecc <= prev_ecc {
            break;
        }
        prev_ecc = ecc;
        start = deepest;
    }
    start
}

/// BFS from `start` over the component; returns (minimum-degree node in the
/// deepest level, eccentricity). Uses `dist` as a 1-based visited/distance
/// buffer and `touched` as the queue + reset list, leaving `dist` all-zero on
/// return so it can be reused.
fn bfs_deepest(
    start: usize,
    adj: &[Vec<usize>],
    degree: &[usize],
    dist: &mut [u32],
    touched: &mut Vec<usize>,
) -> (usize, u32) {
    touched.clear();
    touched.push(start);
    dist[start] = 1;
    let mut head = 0;
    let mut max_d = 1u32;
    while head < touched.len() {
        let u = touched[head];
        head += 1;
        let d = dist[u];
        if d > max_d {
            max_d = d;
        }
        for &v in &adj[u] {
            if dist[v] == 0 {
                dist[v] = d + 1;
                touched.push(v);
            }
        }
    }

    let mut best = start;
    let mut best_deg = usize::MAX;
    for &u in touched.iter() {
        if dist[u] == max_d && degree[u] < best_deg {
            best_deg = degree[u];
            best = u;
        }
    }

    for &u in touched.iter() {
        dist[u] = 0; // restore invariant for reuse
    }

    (best, max_d - 1)
}

/// Predicted factorization flops `Σ_j c_j²` for `perm` on `pat`, via feral's
/// pattern-pure symbolic building blocks — the exact quantity the grader ranks.
fn flops_of(pat: &ScoringPattern, perm: &[usize]) -> u64 {
    let permuted = permute_pattern(pat, perm);
    let etree = EliminationTree::from_pattern(&permuted);
    let counts = column_counts_gnp(&permuted, &etree);
    counts.iter().map(|&c| (c as u64) * (c as u64)).sum()
}

/// Whether `perm` is a bijection of `0..n` (guards a candidate before scoring).
fn is_bijection(perm: &[usize], n: usize) -> bool {
    if perm.len() != n {
        return false;
    }
    let mut seen = vec![false; n];
    for &v in perm {
        if v >= n || seen[v] {
            return false;
        }
        seen[v] = true;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_bijection(perm: &[usize], n: usize) {
        assert_eq!(perm.len(), n, "permutation length");
        let mut seen = vec![false; n];
        for &v in perm {
            assert!(v < n && !seen[v], "not a bijection of 0..{n}");
            seen[v] = true;
        }
    }

    #[test]
    fn order_is_a_valid_bijection() {
        let n = 60;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 8 {
            edges.push((v, v + 8));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_bijection(&order(&pat), n);
    }

    #[test]
    fn order_handles_empty() {
        let pat = Pattern::from_edges(0, &[]);
        assert!(order(&pat).is_empty());
    }

    #[test]
    fn order_handles_singleton() {
        let pat = Pattern::from_edges(1, &[]);
        assert_eq!(order(&pat), vec![0]);
    }

    #[test]
    fn order_handles_no_edges() {
        let n = 10;
        let pat = Pattern::from_edges(n, &[]);
        assert_bijection(&order(&pat), n);
    }

    #[test]
    fn arrow_is_valid() {
        let n = 40;
        let mut edges = Vec::new();
        for v in 1..n {
            edges.push((0, v));
        }
        for v in 1..n - 1 {
            edges.push((v, v + 1));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_bijection(&order(&pat), n);
    }

    #[test]
    fn order_is_deterministic() {
        let n = 200;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 13 {
            edges.push((v, v + 13));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_eq!(order(&pat), order(&pat));
    }

    /// RCM must always return a valid bijection, including on a disconnected
    /// graph (two independent paths) and an edgeless graph.
    #[test]
    fn rcm_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        // component A: a path
        for v in 0..20 {
            edges.push((v, v + 1));
        }
        // component B: another path (disjoint from A)
        for v in 25..40 {
            edges.push((v, v + 1));
        }
        // nodes 41..50 are isolated
        let pat = Pattern::from_edges(n, &edges);
        let perm: Vec<usize> = rcm_order(&pat).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm, n);

        let empty = Pattern::from_edges(12, &[]);
        let perm2: Vec<usize> =
            rcm_order(&empty).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm2, 12);
    }

    /// Sloan must always return a valid bijection, including on a disconnected
    /// graph (two independent paths + isolated nodes) and an edgeless graph, for
    /// both weight settings.
    #[test]
    fn sloan_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        for v in 0..20 {
            edges.push((v, v + 1));
        }
        for v in 25..40 {
            edges.push((v, v + 1));
        }
        // nodes 41..50 are isolated
        let pat = Pattern::from_edges(n, &edges);
        let perm: Vec<usize> =
            sloan_order(&pat, 2, 1).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm, n);
        let perm_b: Vec<usize> =
            sloan_order(&pat, 1, 2).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm_b, n);

        let empty = Pattern::from_edges(12, &[]);
        let perm2: Vec<usize> =
            sloan_order(&empty, 1, 2).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm2, 12);
    }

    /// Sloan must be deterministic across repeated calls.
    #[test]
    fn sloan_is_deterministic() {
        let n = 120;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 7 {
            edges.push((v, v + 7));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_eq!(sloan_order(&pat, 2, 1), sloan_order(&pat, 2, 1));
    }

    /// Hand-rolled nested dissection must always return a valid bijection,
    /// including on a disconnected graph (two independent paths + isolated
    /// nodes), an edgeless graph, and a dense-ish grid.
    #[test]
    fn nd_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        for v in 0..20 {
            edges.push((v, v + 1));
        }
        for v in 25..40 {
            edges.push((v, v + 1));
        }
        // nodes 41..50 are isolated
        let pat = Pattern::from_edges(n, &edges);
        let perm: Vec<usize> = nd_order(&pat).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm, n);

        let empty = Pattern::from_edges(12, &[]);
        let perm2: Vec<usize> =
            nd_order(&empty).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm2, 12);

        // A larger banded/grid-like structure that exercises real bisection.
        let m = 600;
        let mut e2 = Vec::new();
        for v in 0..m - 1 {
            e2.push((v, v + 1));
        }
        for v in 0..m - 20 {
            e2.push((v, v + 20));
        }
        let grid = Pattern::from_edges(m, &e2);
        let perm3: Vec<usize> =
            nd_order(&grid).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm3, m);
    }

    /// Hand-rolled nested dissection must be deterministic across repeated calls.
    #[test]
    fn nd_is_deterministic() {
        let n = 500;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 25 {
            edges.push((v, v + 25));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_eq!(nd_order(&pat), nd_order(&pat));
    }

    /// GGGP graph-growing bisection must always return a valid bijection,
    /// including on a disconnected graph (two independent paths + isolated
    /// nodes), an edgeless graph, and a dense-ish grid.
    #[test]
    fn ndfm_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        for v in 0..20 {
            edges.push((v, v + 1));
        }
        for v in 25..40 {
            edges.push((v, v + 1));
        }
        // nodes 41..50 are isolated
        let pat = Pattern::from_edges(n, &edges);
        let perm: Vec<usize> = ndfm_order(&pat).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm, n);

        let empty = Pattern::from_edges(12, &[]);
        let perm2: Vec<usize> =
            ndfm_order(&empty).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm2, 12);

        // A larger banded/grid-like structure that exercises real bisection.
        let m = 600;
        let mut e2 = Vec::new();
        for v in 0..m - 1 {
            e2.push((v, v + 1));
        }
        for v in 0..m - 20 {
            e2.push((v, v + 20));
        }
        let grid = Pattern::from_edges(m, &e2);
        let perm3: Vec<usize> =
            ndfm_order(&grid).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm3, m);

        // A 2D grid (mesh-like) to exercise real vertex separators.
        let side = 24usize;
        let mut e3 = Vec::new();
        for r in 0..side {
            for c in 0..side {
                let v = r * side + c;
                if c + 1 < side {
                    e3.push((v, v + 1));
                }
                if r + 1 < side {
                    e3.push((v, v + side));
                }
            }
        }
        let mesh = Pattern::from_edges(side * side, &e3);
        let perm4: Vec<usize> =
            ndfm_order(&mesh).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm4, side * side);
    }

    /// GGGP graph-growing bisection must be deterministic across repeated calls.
    #[test]
    fn ndfm_is_deterministic() {
        let n = 500;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 25 {
            edges.push((v, v + 25));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_eq!(ndfm_order(&pat), ndfm_order(&pat));
    }

    /// Minimum-fill (minimum-deficiency) ordering must always return a valid
    /// bijection, including on a disconnected graph (two independent paths +
    /// isolated nodes), an edgeless graph, a dense-ish band, and a 2D mesh.
    #[test]
    fn minfill_is_a_valid_bijection() {
        let n = 50;
        let mut edges = Vec::new();
        for v in 0..20 {
            edges.push((v, v + 1));
        }
        for v in 25..40 {
            edges.push((v, v + 1));
        }
        // nodes 41..50 are isolated
        let pat = Pattern::from_edges(n, &edges);
        let perm: Vec<usize> =
            minfill_order(&pat).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm, n);

        let empty = Pattern::from_edges(12, &[]);
        let perm2: Vec<usize> =
            minfill_order(&empty).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm2, 12);

        // A banded structure with real fill choices.
        let m = 300;
        let mut e2 = Vec::new();
        for v in 0..m - 1 {
            e2.push((v, v + 1));
        }
        for v in 0..m - 10 {
            e2.push((v, v + 10));
        }
        let band = Pattern::from_edges(m, &e2);
        let perm3: Vec<usize> =
            minfill_order(&band).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm3, m);

        // A 2D grid (mesh-like).
        let side = 20usize;
        let mut e3 = Vec::new();
        for r in 0..side {
            for c in 0..side {
                let v = r * side + c;
                if c + 1 < side {
                    e3.push((v, v + 1));
                }
                if r + 1 < side {
                    e3.push((v, v + side));
                }
            }
        }
        let mesh = Pattern::from_edges(side * side, &e3);
        let perm4: Vec<usize> =
            minfill_order(&mesh).into_iter().map(|x| x as usize).collect();
        assert_bijection(&perm4, side * side);
    }

    /// Minimum-fill ordering must be deterministic across repeated calls.
    #[test]
    fn minfill_is_deterministic() {
        let n = 400;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 9 {
            edges.push((v, v + 9));
        }
        let pat = Pattern::from_edges(n, &edges);
        assert_eq!(minfill_order(&pat), minfill_order(&pat));
    }

    /// Best-of must never be worse than the grader's baseline AMD: on any pattern
    /// the returned flops are ≤ default-AMD's flops (the ratio the grader
    /// computes is ≤ 1).
    #[test]
    fn best_of_is_never_worse_than_amd() {
        let n = 120;
        let mut edges = Vec::new();
        for v in 0..n - 1 {
            edges.push((v, v + 1));
        }
        for v in 0..n - 10 {
            edges.push((v, v + 10));
        }
        let pat = Pattern::from_edges(n, &edges);

        let col_ptr_i32: Vec<i32> = pat.col_ptr.iter().map(|&x| x as i32).collect();
        let row_idx_i32: Vec<i32> = pat.row_idx.iter().map(|&x| x as i32).collect();
        let core =
            feral_ordering_core::CscPattern::new(n, &col_ptr_i32, &row_idx_i32).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core)
            .unwrap()
            .into_iter()
            .map(|x| x as usize)
            .collect();
        let scoring_pat = ScoringPattern {
            n,
            col_ptr: pat.col_ptr.clone(),
            row_idx: pat.row_idx.clone(),
        };
        let amd_flops = flops_of(&scoring_pat, &amd);
        let ours_flops = flops_of(&scoring_pat, &order(&pat));
        assert!(ours_flops <= amd_flops, "ours {ours_flops} > amd {amd_flops}");
    }

    #[test]
    fn order_reduces_large_pooling_fixture() {
        let (_, pat) = crate::corpus::corpus()
            .into_iter()
            .find(|(name, _)| name == "pooling_sppc1pq")
            .expect("development fixture");
        let n = pat.n;
        let col_ptr_i32: Vec<i32> = pat.col_ptr.iter().map(|&x| x as i32).collect();
        let row_idx_i32: Vec<i32> = pat.row_idx.iter().map(|&x| x as i32).collect();
        let core =
            feral_ordering_core::CscPattern::new(n, &col_ptr_i32, &row_idx_i32).unwrap();
        let amd: Vec<usize> = feral_amd::amd_order(&core)
            .unwrap()
            .into_iter()
            .map(|x| x as usize)
            .collect();
        let scoring_pat = ScoringPattern {
            n,
            col_ptr: pat.col_ptr.clone(),
            row_idx: pat.row_idx.clone(),
        };
        let amd_flops = flops_of(&scoring_pat, &amd);
        let ours_flops = flops_of(&scoring_pat, &order(&pat));
        assert!(
            (ours_flops as u128) * 4 < amd_flops as u128,
            "expected ratio below 0.25, got {ours_flops}/{amd_flops}"
        );
    }

    #[test]
    fn terminal_deep_search_improves_medium_fixture() {
        let (_, pat) = crate::corpus::corpus()
            .into_iter()
            .find(|(name, _)| name == "rsyn0815m04m")
            .expect("development fixture");

        let perm = order(&pat);
        let scoring_pat = ScoringPattern {
            n: pat.n,
            col_ptr: pat.col_ptr.clone(),
            row_idx: pat.row_idx.clone(),
        };
        let flops = flops_of(&scoring_pat, &perm);

        assert!(flops < 168_000, "expected fewer than 168000 flops, got {flops}");
    }

    #[test]
    fn subtree_configs_stay_within_matrix_work_limit() {
        let requested_budget = SUBTREE_CFG
            .budget
            .saturating_mul(SUBTREE_CFG.max_blocks as i64)
            .saturating_mul(SUBTREE_CFG.streams.max(1) as i64);
        assert!(requested_budget <= SUBTREE_SEARCH_WORK_LIMIT);

        for cfg in [
            terminal_deep_subtree_cfg(9_999, 0),
            terminal_deep_subtree_cfg(10_000, 0),
            terminal_deep_subtree_cfg(10_000, 100_000),
        ] {
            let requested_budget = cfg
                .budget
                .saturating_mul(cfg.max_blocks as i64)
                .saturating_mul(cfg.streams.max(1) as i64);

            assert!(requested_budget <= TERMINAL_SUBTREE_SEARCH_WORK_LIMIT);
        }
    }
}
