use super::*;

pub(super) fn admits(pattern: &Pattern, amd_cost: u64) -> bool {
    (64..=320_000).contains(&pattern.n)
        && pattern.nnz() <= 1_500_000
        && amd_cost >= 32 * pattern.n as u64
}

pub(super) struct Seeds {
    pub(super) core: Option<ladder::Core>,
    pub(super) donors: Vec<Vec<usize>>,
}

pub(super) fn refine_small(pattern: &Pattern, scoring: &ScoringPattern, best: &mut Candidate) {
    if !(64..10_000).contains(&pattern.n)
        || pattern.nnz() > 150_000
        || best.cost < 32 * pattern.n as u64
    {
        return;
    }
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
    best.consider(
        scoring,
        recovery_separator::recovered(pattern, &best.perm, &cfg),
    );
}

fn config(pattern: &Pattern, directed: bool) -> recovery_separator::RecoveryCfg {
    let mut cfg = recovery_separator::RecoveryCfg::fast(directed && pattern.nnz() <= 5 * pattern.n);
    cfg.atoms = true;
    cfg.modes = vec![0, 3, 6];
    if directed {
        cfg.roots = 18;
        cfg.big_cap_roots = 4;
        cfg.skeleton_modes = vec![0, 1, 2, 5, 7, 9, 11];
        cfg.directed = true;
        cfg.greedy_work = 800_000;
        cfg.rotation_work = 2_000_000;
        cfg.metis_tasks = 4;
        cfg.full_tasks = 4;
        cfg.dense_modes = 2;
        cfg.improve_rounds = 1;
        cfg.restarts = 2;
        cfg.unions = 4;
        cfg.threads = 4;
    } else {
        cfg.roots = 6;
        cfg.caps = vec![2048, 512, 128];
        cfg.skeleton_modes = vec![0, 5, 9];
        cfg.greedy_work = 200_000;
        cfg.rotation_work = 300_000;
        cfg.rotation_depth = 48;
        cfg.metis_tasks = 1;
        cfg.natives = 1;
        cfg.full_tasks = 16;
        cfg.threads = 3;
    }
    cfg
}

pub(super) fn bounded(
    pattern: &Pattern,
    scoring: &ScoringPattern,
    best: &mut Candidate,
    seeds: Seeds,
) {
    let Seeds { core, mut donors } = seeds;
    donors.push(best.perm.clone());
    let combine = pattern.n <= 50_000;
    let mut pool = decomposition::WholePool::new(pattern);
    best.consider(scoring, promotions::refine(pattern, &best.perm));
    advancements::complete(pattern, scoring, &mut best.perm, &mut best.cost);
    best.consider(scoring, recovery_regions::refine(pattern, &best.perm));
    best.consider(scoring, recovery_separator::refine(pattern, &best.perm));
    for directed in [false, true] {
        let cfg = config(pattern, directed);
        let cost = best.cost;
        let seed = best.perm.clone();
        let residual = if directed {
            core.as_ref()
                .and_then(|core| residual_proposal(pattern, &seed, cost, core, &cfg))
        } else {
            None
        };
        let proposal =
            residual.unwrap_or_else(|| recovery_separator::recovered(pattern, &seed, &cfg));
        if let Some(order) = proposal {
            donors.push(order.clone());
            best.consider(scoring, Some(order));
        }
        if combine {
            let mut refs: Vec<_> = donors.iter().rev().take(3).map(Vec::as_slice).collect();
            refs.push(donors[0].as_slice());
            best.consider(scoring, pool.combine(&refs));
            donors.push(best.perm.clone());
        }
    }
    donors.push(best.perm.clone());
    if combine {
        let mut refs = vec![donors.last().unwrap().as_slice(), donors[0].as_slice()];
        refs.extend(donors.iter().rev().skip(1).take(2).map(Vec::as_slice));
        best.consider(scoring, pool.combine(&refs));
        best.consider(scoring, decomposition::prefix_pool(pattern, &refs));
    }
}

fn residual_proposal(
    pattern: &Pattern,
    seed: &[usize],
    cost: u64,
    core: &ladder::Core,
    cfg: &recovery_separator::RecoveryCfg,
) -> Option<Option<Vec<usize>>> {
    let k = core.ids.len();
    if !(1000..=20_000).contains(&k) {
        return None;
    }
    let mut inverse = vec![usize::MAX; pattern.n];
    for (i, &v) in core.ids.iter().enumerate() {
        inverse[v] = i;
    }
    let order: Vec<_> = seed
        .iter()
        .map(|&v| inverse[v])
        .filter(|&v| v != usize::MAX)
        .collect();
    let local = Pattern {
        n: k,
        col_ptr: core.col_ptr.clone(),
        row_idx: core.row_idx.clone(),
    };
    let scoring = ScoringPattern {
        n: k,
        col_ptr: core.col_ptr.clone(),
        row_idx: core.row_idx.clone(),
    };
    let local_cost = flops_of(&scoring, &order);
    if (core.prefix_flops + local_cost).saturating_mul(100) > cost.saturating_mul(110) {
        return None;
    }
    Some(
        recovery_separator::recovered(&local, &order, cfg).map(|proposal| {
            let mut full = core.prefix.clone();
            full.extend(proposal.iter().map(|&v| core.ids[v]));
            full
        }),
    )
}
