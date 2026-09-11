//! Bounded separator search replaces the longer legacy route on fill-heavy graphs.
//! Whole-graph and residual proposals share a seed and run independently; exact
//! region recombination merges compatible gains before final symbolic acceptance.
use super::*;

pub(super) fn admits(pattern: &Pattern, amd_cost: u64) -> bool {
    (64..=320_000).contains(&pattern.n)
        && pattern.nnz() <= 1_500_000
        && amd_cost >= 32 * pattern.n as u64
}

pub(super) fn refine_small(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    if !(64..10_000).contains(&pattern.n) || pattern.nnz() > 150_000
        || best.cost < 32 * pattern.n as u64 { return; }
    let mut cfg = recovery_separator::RecoveryCfg::fast(pattern.nnz() <= 5 * pattern.n);
    cfg.roots = 2;
    cfg.caps = vec![256, 96];
    cfg.skeleton_modes = vec![0, 5];
    cfg.modes = vec![0, 3, 6];
    cfg.metis_tasks = 1;
    cfg.atoms = true;
    cfg.threads = 3;
    cfg.max_interface = 1024;
    cfg.greedy_work = 150_000;
    cfg.rotation_work = 100_000;
    cfg.rotation_depth = 32;
    cfg.max_atoms = 256;
    cfg.dense_modes = 0;
    cfg.improve_rounds = 0;
    cfg.restarts = 0;
    cfg.pool_sources = false;
    cfg.unions = 0;
    best.consider(scoring, recovery_separator::recovered(pattern, &best.perm, &cfg));
}

#[cfg(test)]
fn config(pattern: &Pattern) -> recovery_separator::RecoveryCfg {
    let mut cfg = recovery_separator::RecoveryCfg::fast(pattern.nnz() <= 5 * pattern.n);
    cfg.roots = 12;
    cfg.caps = if pattern.nnz() <= 5 * pattern.n {
        vec![2048, 512, 128]
    } else {
        vec![512, 128]
    };
    cfg.skeleton_modes = vec![0, 5, 9];
    cfg.modes = vec![0, 3, 6];
    cfg.greedy_work = 1_200_000;
    cfg.rotation_work = 1_000_000;
    cfg.rotation_depth = 64;
    cfg.metis_tasks = 1;
    cfg.atoms = true;
    cfg.max_atoms = 600;
    cfg.max_interface = 4096;
    cfg.dense_modes = 2;
    cfg.improve_rounds = 1;
    cfg.restarts = 2;
    cfg.pool_sources = true;
    cfg.unions = 4;
    cfg.threads = 3;
    cfg
}

pub(super) struct Seeds {
    pub(super) core: Option<ladder::Core>,
    pub(super) donors: Vec<Vec<usize>>,
}

#[cfg(test)]
pub(super) fn seed(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) -> Seeds {
    let mut donors = vec![best.perm.clone()];
    let core = core_seeds(pattern, scoring, best, &mut donors, false);
    Seeds { core, donors }
}

pub(super) fn bounded(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate, seeds: Seeds) {
    #[cfg(test)] let mut mark = stage_timer("BOUNDED_STAGE");
    let Seeds { core, mut donors } = seeds;
    donors.push(best.perm.clone());
    // Reuse the existing core on large graphs. Rebuilding the native donor
    // portfolio and repeatedly combining whole orders costs more than the
    // bounded patch search itself at this scale.
    let combine_donors = pattern.n <= 50_000;
    let mut pool = decomposition::WholePool::new(pattern);
    best.consider(scoring, promotions::refine(pattern, &best.perm));
    advancements::complete(pattern, scoring, &mut best.perm, &mut best.cost);
    best.consider(scoring, recovery_regions::refine(pattern, &best.perm));
    best.consider(scoring, recovery_separator::refine(pattern, &best.perm));
    #[cfg(test)] mark("prepare");
    let mut cfg = recovery_separator::RecoveryCfg::fast(pattern.nnz() <= 5 * pattern.n);
    cfg.roots = 18;
    cfg.big_cap_roots = 4;
    cfg.skeleton_modes = vec![0, 1, 2, 5, 7, 9, 11];
    cfg.root_select = 4;
    cfg.modes = vec![0, 3, 6];
    cfg.greedy_work = 800_000;
    cfg.rotation_work = 2_000_000;
    cfg.metis_tasks = 4;
    cfg.full_tasks = 4;
    cfg.dense_modes = 2;
    cfg.improve_rounds = 1;
    cfg.restarts = 2;
    cfg.unions = 4;
    cfg.atoms = true;
    cfg.max_atoms = 600;
    cfg.max_interface = 4096;
    cfg.threads = 3;
    for directed_pass in [false, true] {
        let before = best.cost;
        let seed = best.perm.clone();
        // First establish an inexpensive completion; then spend the stronger
        // search on the largest remaining gaps in that completion.
        let pass = if !directed_pass {
            let mut coarse = recovery_separator::RecoveryCfg::fast(false);
            coarse.roots = 6;
            coarse.caps = vec![2048, 512, 128];
            coarse.skeleton_modes = vec![0, 5, 9];
            coarse.modes = vec![0, 3, 6];
            coarse.root_select = 0;
            coarse.greedy_work = 200_000;
            coarse.atoms = true;
            coarse.rotation_work = 300_000;
            coarse.rotation_depth = 48;
            coarse.metis_tasks = 1;
            coarse.natives = 1;
            coarse.threads = 3;
            coarse
        } else {
            let mut directed = cfg.clone();
            directed.threads = 4;
            directed
        };
        let (whole, residual) = std::thread::scope(|scope| {
            let residual = scope.spawn(|| core.as_ref().and_then(|core| {
                residual_proposal(pattern, &seed, before, core, &pass)
            }));
            let whole = recovery_separator::recovered(pattern, &seed, &pass);
            (whole, residual.join().unwrap())
        });
        for order in [whole, residual].into_iter().flatten() {
            donors.push(order.clone());
            best.consider(scoring, Some(order));
        }
        if combine_donors {
            let mut references: Vec<_> = donors.iter().rev().take(3).map(Vec::as_slice).collect();
            references.push(donors[0].as_slice());
            best.consider(scoring, pool.combine(&references));
            donors.push(best.perm.clone());
        }
    }
    #[cfg(test)] mark("patches");
    donors.push(best.perm.clone());
    if combine_donors {
        let mut references = vec![donors.last().unwrap().as_slice(), donors[0].as_slice()];
        references.extend(donors.iter().rev().skip(1).take(2).map(Vec::as_slice));
        best.consider(scoring, pool.combine(&references));
        best.consider(scoring, decomposition::prefix_pool(pattern, &references));
    }
    #[cfg(test)] mark("pool");
}

#[cfg(test)]
pub(super) fn refine(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    #[cfg(test)]
    let mut mark = stage_timer("QUALITY_STAGE");
    let mut donors = vec![best.perm.clone()];
    let core = core_seeds(pattern, scoring, best, &mut donors, false);
    #[cfg(test)] mark("core_seeds");
    best.consider(scoring, promotions::refine(pattern, &best.perm));
    #[cfg(test)] mark("promotions");
    advancements::complete(pattern, scoring, &mut best.perm, &mut best.cost);
    #[cfg(test)] mark("completion");
    best.consider(scoring, recovery_regions::refine(pattern, &best.perm));
    #[cfg(test)] mark("regions");
    best.consider(scoring, recovery_separator::refine(pattern, &best.perm));
    #[cfg(test)] mark("cheap_separator");
    search_rounds(pattern, scoring, best, core.as_ref(), donors);
    completion_orbit(pattern, scoring, best);
    let mut cfg = config(pattern);
    cfg.bag_search = true;
    for round in 0..3 {
        cfg.root_select = round;
        let before = best.cost;
        best.consider(scoring, recovery_separator::recovered(pattern, &best.perm, &cfg));
        if best.cost == before { break; }
    }
    broad_portfolio(pattern, scoring, best, core.as_ref(), Vec::new());
}

/// Run additions only after the complete existing route, including its small
/// cleanup. Better intermediate states must not erase a later baseline win.
#[cfg(test)]
pub(super) fn post_refine(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    if !(64..=320_000).contains(&pattern.n) || pattern.nnz() > 1_500_000 { return; }
    let original_entries = pattern.n + (0..pattern.n).map(|v| {
        pattern.row_idx[pattern.col_ptr[v]..pattern.col_ptr[v+1]].iter().filter(|&&u| u > v).count()
    }).sum::<usize>();
    if cost_and_entries(scoring, &best.perm).1 == original_entries { return; }
    let mut donors = vec![best.perm.clone()];
    let core = core_seeds(pattern, scoring, best, &mut donors, true);
    donors.push(best.perm.clone());
    let mut pool = decomposition::WholePool::new(pattern);
    let references: Vec<_> = donors.iter().map(Vec::as_slice).collect();
    best.consider(scoring, pool.combine(&references));
    best.consider(scoring, decomposition::prefix_closure(pattern, &references));
    let mut cfg = config(pattern);
    cfg.roots = 18;
    cfg.big_cap_roots = 4;
    cfg.skeleton_modes = vec![0, 1, 2, 5, 7, 9, 11];
    cfg.dense_modes = 3;
    cfg.rotation_work = 4_000_000;
    cfg.metis_tasks = 4;
    cfg.unions = 16;
    cfg.bag_search = true;
    cfg.prefix_grid = true;
    for round in 0..3 {
        cfg.root_select = round;
        best.consider(scoring, recovery_separator::recovered(pattern, &best.perm, &cfg));
        if let Some(core) = &core {
            best.consider(scoring, residual_proposal(pattern, &best.perm, best.cost, core, &cfg));
        }
        donors.push(best.perm.clone());
        let references: Vec<_> = donors.iter().map(Vec::as_slice).collect();
        best.consider(scoring, pool.combine(&references));
        best.consider(scoring, decomposition::prefix_closure(pattern, &references));
    }
    best.consider(scoring, recovery_regions::refine_global(pattern, &best.perm));
    fresh_start(pattern, scoring, best);
}

/// Refine a native seed independently of the legacy incumbent. An initially
/// losing seed may reach a completion that the incumbent's local moves miss.
#[cfg(test)]
fn fresh_start(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    if !(10_000..=80_000).contains(&pattern.n) || pattern.nnz() > 400_000 { return; }
    let cp: Vec<i32> = pattern.col_ptr.iter().map(|&v| v as i32).collect();
    let ri: Vec<i32> = pattern.row_idx.iter().map(|&v| v as i32).collect();
    let Some(graph) = feral_ordering_core::CscPattern::new(pattern.n, &cp, &ri) else { return; };
    let Ok(order) = feral_amd::amd_order(&graph) else { return; };
    let perm: Vec<_> = order.into_iter().map(|v| v as usize).collect();
    let cost = flops_of(scoring, &perm);
    if cost < 512 * pattern.n as u64 { return; }
    let mut branch = Candidate { perm, cost };
    let mut donors = vec![branch.perm.clone()];
    if let Ok((order, ..)) = feral_amf::amf_order_opts(&graph, &feral_amf::AmfOptions {
        dense_alpha: 5.0, ..Default::default()
    }) {
        let order: Vec<_> = order.into_iter().map(|v| v as usize).collect();
        donors.push(order.clone());
        branch.consider(scoring, Some(order));
    }
    let core = core_seeds(pattern, scoring, &mut branch, &mut donors, true);
    branch.consider(scoring, promotions::refine(pattern, &branch.perm));
    advancements::complete(pattern, scoring, &mut branch.perm, &mut branch.cost);
    branch.consider(scoring, recovery_regions::refine(pattern, &branch.perm));
    branch.consider(scoring, recovery_separator::refine(pattern, &branch.perm));
    broad_portfolio(pattern, scoring, &mut branch, core.as_ref(), donors);
    best.consider(scoring, Some(branch.perm));
}

/// Retain complementary separator searches from the same starting order.
/// A narrower branch can donate regions even when its whole order loses.
#[cfg(test)]
fn portfolio_configs(pattern: &Pattern) -> Vec<recovery_separator::RecoveryCfg> {
    let validated = recovery_separator::RecoveryCfg::fast(pattern.nnz() <= 5 * pattern.n);
    let mut narrow = validated.clone();
    narrow.skeleton_modes = vec![0, 1, 2, 5, 7, 9];
    narrow.full_tasks = 8;
    narrow.big_cap_roots = 4;
    narrow.dense_modes = 8;
    narrow.improve_rounds = 1;
    narrow.restarts = 4;
    narrow.atoms = true;
    narrow.unions = 8;
    narrow.root_select = 1;
    let mut wide = narrow.clone();
    wide.roots = 12;
    wide.full_tasks = usize::MAX;
    wide.metis_tasks = 1;
    wide.dense_modes = 0;
    wide.improve_rounds = 0;
    wide.restarts = 0;
    wide.root_select = 0;
    wide.skeleton_modes = vec![0, 5, 9];
    wide.big_cap_roots = 1;
    let mut expanded = wide.clone();
    expanded.roots = 18;
    expanded.big_cap_roots = 4;
    expanded.skeleton_modes = vec![0, 1, 2, 5, 7, 9, 11];
    expanded.modes = vec![0, 3, 6];
    expanded.greedy_work = 1_200_000;
    expanded.rotation_work = 4_000_000;
    expanded.metis_tasks = 4;
    expanded.dense_modes = 2;
    expanded.improve_rounds = 1;
    expanded.restarts = 4;
    expanded.unions = 16;
    let mut pooled = expanded.clone();
    pooled.pool_sources = true;
    let mut focused = expanded.clone();
    focused.root_select = 4;
    focused.full_tasks = 8;
    focused.unions = 4;
    let mut configs = vec![validated, narrow, wide, expanded, pooled, focused];
    for cfg in &mut configs { cfg.threads = 3; }
    configs
}

#[cfg(test)]
fn broad_portfolio(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate,
    core: Option<&ladder::Core>, mut donors: Vec<Vec<usize>>) {
    let configs = portfolio_configs(pattern);
    donors.push(best.perm.clone());
    let mut pool = decomposition::WholePool::new(pattern);
    for _ in 0..8 {
        let before = best.cost;
        let seed = best.perm.clone();
        let results = parallel::map_indexed(&configs, 2, |_, cfg| {
            let mut branch = Candidate { perm: seed.clone(), cost: before };
            branch.consider(scoring, recovery_separator::recovered(pattern, &branch.perm, cfg));
            if let Some(core) = core {
                branch.consider(scoring, residual_proposal(pattern, &branch.perm, branch.cost, core, cfg));
            }
            branch.perm
        });
        donors.extend(results.iter().cloned());
        let references: Vec<_> = donors.iter().map(Vec::as_slice).collect();
        for order in results {
            let crossed = recovery_regions::crossover_many(pattern, &order, &references, 64);
            best.consider(scoring, Some(order));
            best.consider(scoring, crossed);
        }
        best.consider(scoring, pool.combine(&references));
        best.consider(scoring, decomposition::prefix_closure(pattern, &references));
        donors.push(best.perm.clone());
        if best.cost.saturating_mul(1000) >= before.saturating_mul(998) { break; }
    }
}

/// Equal-cost PEOs expose different separator trees. Refine each before
/// comparing costs, so neutral changes survive long enough to become useful.
#[cfg(test)]
fn completion_orbit(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    let initial_cost = best.cost;
    let permuted = permute_pattern(scoring, &best.perm);
    let tree = EliminationTree::from_pattern(&permuted);
    let counts: Vec<u32> = symbolic_counts(&permuted, &tree).1
        .into_iter().map(|c| c as u32).collect();
    let limits = completion::CompletionLimits {
        max_n: 320_000,
        max_input_nnz: 1_500_000,
        max_lnnz: 12_000_000,
    };
    let Some(orders) = completion::peo_candidates(
        pattern.n, &pattern.col_ptr, &pattern.row_idx, &best.perm, &counts, &limits,
    ) else { return; };
    let mut donors = vec![best.perm.clone()];
    let mut cfg = config(pattern);
    for order in orders {
        if donors.contains(&order) { continue; }
        let mut branch = Candidate { cost: flops_of(scoring, &order), perm: order };
        if branch.cost > initial_cost { continue; }
        donors.push(branch.perm.clone());
        for round in 0..3 {
            cfg.root_select = round;
            let before = branch.cost;
            branch.consider(scoring, recovery_separator::recovered(pattern, &branch.perm, &cfg));
            if branch.cost == before { break; }
        }
        donors.push(branch.perm.clone());
        best.consider(scoring, Some(branch.perm));
    }
    let references: Vec<_> = donors.iter().map(Vec::as_slice).collect();
    best.consider(scoring, decomposition::WholePool::new(pattern).combine(&references));
    best.consider(scoring, decomposition::prefix_closure(pattern, &references));
}

#[cfg(test)]
fn search_rounds(
    pattern: &Pattern,
    scoring: &ScoringPattern,
    best: &mut Candidate,
    core: Option<&ladder::Core>,
    mut donors: Vec<Vec<usize>>,
) {
    #[cfg(test)]
    let mut mark = stage_timer("QUALITY_ROUND");
    let mut pool = decomposition::WholePool::new(pattern);
    for round in 0..8 {
        let before = best.cost;
        let seed = best.perm.clone();
        let mut cfg = config(pattern);
        if round % 3 == 1 {
            cfg.skeleton_modes = vec![5, 9];
            cfg.root_select = 2;
        } else if round % 3 == 2 {
            cfg.root_select = 1;
        }
        let (whole, residual) = std::thread::scope(|scope| {
            let residual = scope.spawn(|| core.and_then(|core| {
                residual_proposal(pattern, &seed, best.cost, core, &cfg)
            }));
            let whole = recovery_separator::recovered(pattern, &seed, &cfg);
            (whole, residual.join().unwrap())
        });
        for order in [whole, residual].into_iter().flatten() {
            donors.push(order.clone());
            best.consider(scoring, Some(order));
        }
        #[cfg(test)] mark("separator_round");
        donors.push(best.perm.clone());
        let references: Vec<_> = donors.iter().map(Vec::as_slice).collect();
        best.consider(scoring, recovery_regions::crossover_many(pattern, &best.perm, &references, 32));
        #[cfg(test)] mark("crossover");
        let mut references = references;
        references.push(&best.perm);
        let combined = pool.combine(&references);
        best.consider(scoring, combined);
        // Bound alignment to a small pool even when region donors accumulate.
        let mut prefix_sources = vec![best.perm.as_slice(), donors[0].as_slice()];
        prefix_sources.extend(donors.iter().rev().take(2).map(Vec::as_slice));
        let aligned = decomposition::prefix_closure(pattern, &prefix_sources);
        best.consider(scoring, aligned);
        #[cfg(test)] mark("decomposition");
        if best.cost.saturating_mul(1000) >= before.saturating_mul(998) { break; }
    }
}

#[cfg(test)]
fn core_seeds(
    pattern: &Pattern,
    scoring: &ScoringPattern,
    best: &mut Candidate,
    donors: &mut Vec<Vec<usize>>,
    alternatives: bool,
) -> Option<ladder::Core> {
    let donor_limit = if alternatives { donors.len() + 8 } else { 5 };
    let mut ladder = ladder::CoreLadder::new(pattern.n, &pattern.col_ptr, &pattern.row_idx, 12);
    let mut core3 = None;
    // The alternative pool also needs whole-graph donors when peeling does
    // nothing; only subsequent identical residuals are redundant.
    let mut previous = if alternatives { usize::MAX } else { pattern.n };
    let mut work = ladder::WorkLedger::new(12_000_000);
    for threshold in [2, 3, 4, 5, 6, 12] {
        ladder.advance(threshold, 3_000_000);
        let core = ladder.export();
        if threshold == 3 { core3 = Some(core.clone()); }
        let k = core.ids.len();
        if k == 0 {
            best.consider(scoring, Some(core.prefix));
            break;
        }
        if k == previous || k > 40_000 || core.row_idx.len() > 600_000
            || (matches!(threshold, 3 | 5) && k > 4_000)
        {
            continue;
        }
        previous = k;
        if let Some(order) = linegraph::order(&core.col_ptr, &core.row_idx) {
            let mut full = core.prefix.clone();
            full.extend(order.iter().map(|&v| core.ids[v]));
            best.consider(scoring, Some(full));
            continue;
        }
        let cp: Vec<i32> = core.col_ptr.iter().map(|&v| v as i32).collect();
        let ri: Vec<i32> = core.row_idx.iter().map(|&v| v as i32).collect();
        let graph = feral_ordering_core::CscPattern::new(k, &cp, &ri)?;
        let local = ScoringPattern { n: k, col_ptr: core.col_ptr.clone(), row_idx: core.row_idx.clone() };
        let sources: Vec<_> = [0, 1, 7, 21].into_iter()
            .take_while(|_| work.try_charge((k + core.row_idx.len()) as u64)).collect();
        let orders = parallel::map_indexed(&sources, 4, |_, &source| {
            let order = if source == 0 {
                feral_amf::amf_order_opts(&graph, &feral_amf::AmfOptions::default()).map(|(p, ..)| p)
            } else {
                feral_metis::metis_order_full(&graph, &feral_metis::MetisOptions {
                    seed: source, niparts: 7, ..Default::default()
                }).map(|(p, ..)| p)
            }.ok()?;
            let order: Vec<_> = order.into_iter().map(|v| v as usize).collect();
            if !is_bijection(&order, k) { return None; }
            let cost = core.prefix_flops + flops_of(&local, &order);
            Some((order, cost))
        });
        let selected = orders.iter().enumerate().skip(usize::from(alternatives)).filter_map(|(i, r)| r.as_ref().map(|(_, c)| (i, *c)))
            .min_by_key(|&(_, cost)| cost).map(|(i, _)| i);
        for (i, result) in orders.into_iter().enumerate() {
            let Some((order, cost)) = result else { continue; };
            if alternatives {
                if i != 0 && Some(i) != selected { continue; }
            } else if Some(i) != selected && !(i == 0 && donors.len() < 5) { continue; }
            let mut full = core.prefix.clone();
            full.extend(order.iter().map(|&v| core.ids[v]));
            if donors.len() < donor_limit { donors.push(full.clone()); }
            if cost < best.cost { best.consider(scoring, Some(full)); }
        }
    }
    core3
}

fn residual_proposal(
    pattern: &Pattern,
    seed: &[usize],
    cost: u64,
    core: &ladder::Core,
    cfg: &recovery_separator::RecoveryCfg,
) -> Option<Vec<usize>> {
    let k = core.ids.len();
    if !(1000..=20_000).contains(&k) { return None; }
    let mut inverse = vec![usize::MAX; pattern.n];
    for (i, &v) in core.ids.iter().enumerate() { inverse[v] = i; }
    let order: Vec<_> = seed.iter().map(|&v| inverse[v]).filter(|&v| v != usize::MAX).collect();
    let local = Pattern { n: k, col_ptr: core.col_ptr.clone(), row_idx: core.row_idx.clone() };
    let scoring = ScoringPattern { n: k, col_ptr: core.col_ptr.clone(), row_idx: core.row_idx.clone() };
    let local_cost = flops_of(&scoring, &order);
    if (core.prefix_flops + local_cost).saturating_mul(100) > cost.saturating_mul(110) { return None; }
    let proposal = recovery_separator::recovered(&local, &order, cfg)?;
    let mut full = core.prefix.clone();
    full.extend(proposal.iter().map(|&v| core.ids[v]));
    Some(full)
}

#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn probe_large_native_separators() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,88,131,189,247,267];
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/wide_joint_full");
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let incumbent: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &incumbent);
            let mut cfg = super::recovery_separator::RecoveryCfg::fast(false);
            cfg.roots = 4;
            cfg.caps = vec![4096,8192];
            cfg.big_cap_roots = 2;
            cfg.skeleton_modes = vec![0,5,9];
            cfg.max_interface = 9216;
            cfg.patch_work = 16_000_000;
            cfg.modes.clear();
            cfg.rotation_work = 0;
            cfg.metis_tasks = 2;
            cfg.threads = 4;
            for mode in [0,1,3] {
                cfg.root_select = mode;
                let started = std::time::Instant::now();
                let mut best = super::Candidate { perm: incumbent.clone(), cost: before };
                for round in 0..8 {
                    let old = best.cost;
                    best.consider(&scoring, super::recovery_separator::recovered(&pattern, &best.perm, &cfg));
                    if best.cost == old { break; }
                    eprintln!("LARGE_NATIVE_STEP {index} {mode} {round} {old} {}", best.cost);
                }
                eprintln!("LARGE_NATIVE {index} {name} {mode} {before} {} {:.6}", best.cost, started.elapsed().as_secs_f64());
            }
        }
    }
    #[test]
    #[ignore]
    fn probe_residual_scales() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,88,131,189,247,267];
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/wide_joint_full");
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let incumbent: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &incumbent);
            let mut ladder = super::ladder::CoreLadder::new(pattern.n, &pattern.col_ptr, &pattern.row_idx, 12);
            let mut previous = usize::MAX;
            for threshold in [2,3,4,6,12] {
                let started = std::time::Instant::now();
                ladder.advance(threshold, 3_000_000);
                let core = ladder.export();
                let k = core.ids.len();
                if k == previous || !(64..=20_000).contains(&k) { continue; }
                previous = k;
                let local = crate::Pattern { n: k, col_ptr: core.col_ptr.clone(), row_idx: core.row_idx.clone() };
                let local_scoring = super::ScoringPattern { n: k, col_ptr: local.col_ptr.clone(), row_idx: local.row_idx.clone() };
                let mut inverse = vec![usize::MAX; pattern.n];
                for (i, &v) in core.ids.iter().enumerate() { inverse[v] = i; }
                let order: Vec<_> = incumbent.iter().map(|&v| inverse[v]).filter(|&v| v != usize::MAX).collect();
                let mut best = super::Candidate { cost: super::flops_of(&local_scoring, &order), perm: order };
                let projected = core.prefix_flops + best.cost;
                let mut cfg = super::portfolio_configs(&local).pop().unwrap();
                cfg.full_tasks = 6;
                cfg.threads = 4;
                best.consider(&local_scoring, super::recovery_separator::recovered(&local, &best.perm, &cfg));
                let mut full = core.prefix;
                full.extend(best.perm.iter().map(|&v| core.ids[v]));
                let actual = super::flops_of(&scoring, &full);
                assert_eq!(actual, core.prefix_flops + best.cost);
                eprintln!("RESIDUAL_SCALE {index} {name} {threshold} {k} {before} {projected} {actual} {:.6}", started.elapsed().as_secs_f64());
            }
        }
    }
    #[test]
    #[ignore]
    fn probe_independent_native_branch() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,88,131,189,247,267];
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/neutral_tail_full");
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) || pattern.n > 80_000 { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let incumbent: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &incumbent);
            let start = std::time::Instant::now();
            let mut ladder = super::ladder::CoreLadder::new(pattern.n, &pattern.col_ptr, &pattern.row_idx, 3);
            ladder.advance(3, 3_000_000);
            let core = ladder.export();
            if !(64..=30_000).contains(&core.ids.len()) || core.row_idx.len() > 600_000 { continue; }
            let local = crate::Pattern { n: core.ids.len(), col_ptr: core.col_ptr.clone(), row_idx: core.row_idx.clone() };
            let local_scoring = super::ScoringPattern { n: local.n, col_ptr: local.col_ptr.clone(), row_idx: local.row_idx.clone() };
            let cp: Vec<i32> = local.col_ptr.iter().map(|&v| v as i32).collect();
            let ri: Vec<i32> = local.row_idx.iter().map(|&v| v as i32).collect();
            let graph = feral_ordering_core::CscPattern::new(local.n, &cp, &ri).unwrap();
            let Ok((perm, ..)) = feral_metis::metis_order_full(&graph, &feral_metis::MetisOptions { seed: 7, niparts: 7, ..Default::default() }) else { continue; };
            let perm: Vec<_> = perm.into_iter().map(|v| v as usize).collect();
            let mut branch = super::Candidate { cost: super::flops_of(&local_scoring, &perm), perm };
            let mut cfg = super::recovery_separator::RecoveryCfg::fast(local.nnz() <= 5 * local.n);
            cfg.skeleton_modes = vec![0,5,9];
            cfg.modes = vec![0,3,6];
            cfg.atoms = true;
            cfg.threads = 4;
            for _ in 0..2 {
                branch.consider(&local_scoring, super::recovery_separator::recovered(&local, &branch.perm, &cfg));
            }
            cfg = super::portfolio_configs(&local).pop().unwrap();
            cfg.full_tasks = 6;
            cfg.threads = 4;
            branch.consider(&local_scoring, super::recovery_separator::recovered(&local, &branch.perm, &cfg));
            let alternative = core.prefix_flops + branch.cost;
            let mut full = core.prefix;
            full.extend(branch.perm.iter().map(|&v| core.ids[v]));
            assert_eq!(alternative, super::flops_of(&scoring, &full));
            let mut best = super::Candidate { perm: incumbent.clone(), cost: before };
            best.consider(&scoring, Some(full.clone()));
            best.consider(&scoring, super::decomposition::WholePool::new(&pattern).combine(&[&incumbent, &full]));
            eprintln!("CORE_METIS_BRANCH {index} {name} {before} {alternative} {} {:.6}", best.cost, start.elapsed().as_secs_f64());
        }
    }
    #[test]
    #[ignore]
    fn probe_fast_escapes() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,88,131,189,247,267];
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/responsive_directed_full");
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let perm: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &perm);
            let mut cfg = super::portfolio_configs(&pattern).pop().unwrap();
            cfg.full_tasks = 6;
            cfg.threads = 4;
            for arm in 0..4 {
                if arm != 1 { continue; }
                let started = std::time::Instant::now();
                let mut best = super::Candidate { perm: perm.clone(), cost: before };
                if arm == 0 {
                    best.consider(&scoring, super::recovery_regions::refine_global(&pattern, &perm));
                } else if arm == 1 {
                    let (order, counts, _) = super::etree_prep(&scoring, &perm);
                    let limits = super::completion::CompletionLimits { max_n: 80_000, max_input_nnz: 600_000, max_lnnz: 2_000_000 };
                    if let Some(orders) = super::completion::peo_candidates(pattern.n, &pattern.col_ptr, &pattern.row_idx, &order, &counts, &limits) {
                        for (variant, order) in orders.into_iter().take(2).enumerate() {
                            let started = std::time::Instant::now();
                            let mut branch = super::Candidate { perm: order, cost: before };
                            branch.cost = super::flops_of(&scoring, &branch.perm);
                            branch.consider(&scoring, super::recovery_separator::recovered(&pattern, &branch.perm, &cfg));
                            eprintln!("PEO {index} {name} {variant} {before} {} {:.6}", branch.cost, started.elapsed().as_secs_f64());
                            best.consider(&scoring, Some(branch.perm));
                        }
                    }
                } else {
                    cfg.bag_search = arm == 3;
                    cfg.pool_sources = arm == 3;
                    best.consider(&scoring, super::recovery_separator::recovered(&pattern, &perm, &cfg));
                }
                assert!(best.cost <= before);
                eprintln!("ESCAPE {index} {name} {arm} {before} {} {:.6}", best.cost, started.elapsed().as_secs_f64());
            }
        }
    }
    #[test]
    #[ignore]
    fn profile_quality_mechanisms() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74];
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../experiments/production_076/budget_sampled_full");
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let perm: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &perm);
            for (arm, cfg) in super::portfolio_configs(&pattern).iter().enumerate() {
                if arm != 3 && arm != 5 { continue; }
                let started = std::time::Instant::now();
                let mut candidate = None;
                let mut current = perm.clone();
                for _ in 0..if arm == 5 { 3 } else { 1 } {
                    let Some(next) = super::recovery_separator::recovered(&pattern, &current, cfg) else { break; };
                    current = next;
                    candidate = Some(current.clone());
                }
                let seconds = started.elapsed().as_secs_f64();
                let after = candidate.as_ref().map_or(before, |p| super::flops_of(&scoring, p));
                assert!(after <= before);
                eprintln!("MECHANISM {index} {name} {arm} {before} {after} {seconds:.6}");
                super::recovery_separator::profile_report(&format!("{index}/{arm}"));
            }
        }
    }
    #[test]
    #[ignore]
    fn post_refine_frozen_sample() {
        frozen_sample("post_refine_sample_02", super::post_refine);
    }
    #[test]
    #[ignore]
    fn fresh_start_frozen_sample() {
        frozen_sample("fresh_start_sample_01", super::fresh_start);
    }
    fn frozen_sample(label: &str, refine: fn(&crate::Pattern, &super::ScoringPattern, &mut super::Candidate)) {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74];
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let source = root.join("../../experiments/production_076/broad_online");
        let output = root.join(format!("../target/{label}"));
        std::fs::create_dir(&output).unwrap();
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let bytes = std::fs::read(source.join(format!("{index:03}.0.perm"))).unwrap();
            let perm: Vec<_> = bytes.chunks_exact(8).skip(1)
                .map(|b| u64::from_le_bytes(b.try_into().unwrap()) as usize).collect();
            let scoring = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr.clone(), row_idx: pattern.row_idx.clone() };
            let before = super::flops_of(&scoring, &perm);
            let mut best = super::Candidate { cost: before, perm };
            let started = std::time::Instant::now();
            refine(&pattern, &scoring, &mut best);
            assert!(best.cost <= before);
            assert!(super::is_bijection(&best.perm, pattern.n));
            assert_eq!(best.cost, super::flops_of(&scoring, &best.perm));
            let mut bytes = Vec::with_capacity((pattern.n+1)*8);
            for v in std::iter::once(pattern.n).chain(best.perm) { bytes.extend_from_slice(&(v as u64).to_le_bytes()); }
            std::fs::write(output.join(format!("{index:03}.perm")), bytes).unwrap();
            eprintln!("POST_REFINEMENT {index} {name} {before} {} {:.4}", best.cost, started.elapsed().as_secs_f64());
        }
    }
    #[test]
    #[ignore]
    fn profile_representative_route() {
        let selected = [0,22,41,71,155,178,218,252,2,24,60,109,138,185,210,254,
                        50,79,126,268,285,216,150,74,208,209];
        for (index, (name, pattern)) in crate::corpus::corpus().into_iter().enumerate() {
            if !selected.contains(&index) { continue; }
            let started = std::time::Instant::now();
            let order = crate::ordering::order(&pattern);
            let elapsed = started.elapsed().as_secs_f64();
            let graph = super::ScoringPattern { n: pattern.n, col_ptr: pattern.col_ptr, row_idx: pattern.row_idx };
            let cost = super::flops_of(&graph, &order);
            eprintln!("ROUTE\t{index}\t{name}\t{cost}\t{elapsed:.6}");
        }
    }
}
