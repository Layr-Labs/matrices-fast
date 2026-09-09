//! Test-only canonical missed-hit probe for signature-window local problems.
//!
//! This is deliberately instrumentation only.  It never feeds the production
//! memo, budget, or chosen order; it only counts raw misses that an exact
//! boundary-aware canonical key would have merged.

use std::cell::RefCell;
use std::collections::HashSet;

const MAX_WIDTH: usize = 14;
const MAX_CANON_WIDTH: usize = 12;
const MAX_HIST_DISTINCT: usize = 64;
const MAX_CANDIDATES: usize = 64;
const MAX_ATTEMPT_WORK: u64 = 100_000;
const MAX_PROBE_WORK: u64 = 2_000_000;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub(crate) struct CanonicalKey {
    k: u8,
    inside: [u16; MAX_WIDTH],
    hist: Vec<(u16, u32)>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct CanonicalProbeStats {
    pub(crate) calls: u64,
    pub(crate) canonicalized: u64,
    pub(crate) canonical_hits: u64,
    pub(crate) skipped_width: u64,
    pub(crate) skipped_hist: u64,
    pub(crate) skipped_candidates: u64,
    pub(crate) skipped_work: u64,
    pub(crate) candidates: u64,
    pub(crate) cost: u64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SkipReason {
    Width,
    Hist,
    Candidates,
    Work,
}

struct Attempt {
    key: Option<CanonicalKey>,
    skip: Option<SkipReason>,
    candidates: u64,
    cost: u64,
}

#[derive(Default)]
struct ProbeState {
    enabled: bool,
    stats: CanonicalProbeStats,
    remaining: u64,
}

thread_local! {
    static PROBE: RefCell<ProbeState> = RefCell::new(ProbeState::default());
}

pub(crate) fn set_probe_enabled_for_test(enabled: bool) {
    PROBE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.enabled = enabled;
        state.remaining = MAX_PROBE_WORK;
    });
}

pub(crate) fn reset_probe_stats_for_test() {
    PROBE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.stats = CanonicalProbeStats::default();
        state.remaining = MAX_PROBE_WORK;
    });
}

pub(crate) fn take_probe_stats_for_test() -> CanonicalProbeStats {
    PROBE.with(|cell| std::mem::take(&mut cell.borrow_mut().stats))
}

pub(crate) fn probe_enabled_for_test() -> bool {
    PROBE.with(|cell| cell.borrow().enabled)
}

pub(crate) fn observe_raw_miss_for_test(
    seen: &mut HashSet<CanonicalKey>,
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
) {
    let limit = PROBE.with(|cell| cell.borrow().remaining.min(MAX_ATTEMPT_WORK));
    let attempt = canonicalize_limited(k, inside, hist, limit);
    PROBE.with(|cell| {
        let mut state = cell.borrow_mut();
        state.stats.calls += 1;
        state.stats.candidates += attempt.candidates;
        state.stats.cost = state.stats.cost.saturating_add(attempt.cost);
        state.remaining = state.remaining.saturating_sub(attempt.cost);
        match attempt.skip {
            Some(SkipReason::Width) => state.stats.skipped_width += 1,
            Some(SkipReason::Hist) => state.stats.skipped_hist += 1,
            Some(SkipReason::Candidates) => state.stats.skipped_candidates += 1,
            Some(SkipReason::Work) => state.stats.skipped_work += 1,
            None => {}
        }
        if let Some(key) = attempt.key {
            state.stats.canonicalized += 1;
            if !seen.insert(key) {
                state.stats.canonical_hits += 1;
            }
        }
    });
}

pub(crate) fn canonical_key_for_test(
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
) -> Option<CanonicalKey> {
    canonicalize(k, inside, hist).key
}

fn canonicalize(k: usize, inside: &[u16; MAX_WIDTH], hist: &[(u16, u32)]) -> Attempt {
    canonicalize_limited(k, inside, hist, MAX_ATTEMPT_WORK)
}

fn charge(cost: &mut u64, amount: u64, limit: u64) -> bool {
    if amount > limit.saturating_sub(*cost) {
        *cost = limit;
        return false;
    }
    *cost += amount;
    true
}

fn canonicalize_limited(
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
    limit: u64,
) -> Attempt {
    if limit == 0 {
        return Attempt {
            key: None,
            skip: Some(SkipReason::Work),
            candidates: 0,
            cost: 0,
        };
    }
    let mut cost = 1u64;
    if k == 0 || k > MAX_CANON_WIDTH {
        return Attempt {
            key: None,
            skip: Some(SkipReason::Width),
            candidates: 0,
            cost,
        };
    }
    if hist.len() > MAX_HIST_DISTINCT {
        return Attempt {
            key: None,
            skip: Some(SkipReason::Hist),
            candidates: 0,
            cost,
        };
    }
    if !charge(&mut cost, hist.len() as u64, limit) {
        return Attempt {
            key: None,
            skip: Some(SkipReason::Work),
            candidates: 0,
            cost,
        };
    }

    let mut candidates = Vec::new();
    let colors = vec![0u16; k];
    let mut leaf_count = 0u64;
    if !individualize(
        k,
        inside,
        hist,
        colors,
        &mut candidates,
        &mut leaf_count,
        &mut cost,
        limit,
    ) {
        return Attempt {
            key: None,
            skip: Some(if cost == limit {
                SkipReason::Work
            } else {
                SkipReason::Candidates
            }),
            candidates: leaf_count,
            cost,
        };
    }

    let mut best = None;
    for mapping in &candidates {
        if !charge(
            &mut cost,
            (4 * (k * k + hist.len() * (k + 2))) as u64,
            limit,
        ) {
            return Attempt {
                key: None,
                skip: Some(SkipReason::Work),
                candidates: leaf_count,
                cost,
            };
        }
        let key = transformed_key(k, inside, hist, mapping);
        if best.as_ref().is_none_or(|old| key < *old) {
            best = Some(key);
        }
    }
    Attempt {
        key: best,
        skip: None,
        candidates: leaf_count,
        cost,
    }
}

fn individualize(
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
    mut colors: Vec<u16>,
    candidates: &mut Vec<[u8; MAX_WIDTH]>,
    leaf_count: &mut u64,
    cost: &mut u64,
    limit: u64,
) -> bool {
    if !charge(cost, (8 * k * k + 8 * k) as u64, limit)
        || !refine_colors(k, inside, hist, &mut colors, cost, limit)
    {
        return false;
    }
    let mut cells = color_cells(k, &colors);
    if cells.iter().all(|cell| cell.len() == 1) {
        if *leaf_count == MAX_CANDIDATES as u64 {
            return false;
        }
        *leaf_count += 1;
        let mut raw_for_canon = [0u8; MAX_WIDTH];
        let mut by_color: Vec<(u16, usize)> = (0..k).map(|v| (colors[v], v)).collect();
        by_color.sort_unstable();
        for (canon, &(_, raw)) in by_color.iter().enumerate() {
            raw_for_canon[canon] = raw as u8;
        }
        candidates.push(raw_for_canon);
        return true;
    }

    cells.sort_by(|a, b| {
        a.len()
            .cmp(&b.len())
            .then_with(|| colors[a[0]].cmp(&colors[b[0]]))
    });
    let branch = cells.into_iter().find(|cell| cell.len() > 1).unwrap();
    if branch.len() as u64 > MAX_CANDIDATES as u64 - *leaf_count {
        return false;
    }
    let marker = colors.iter().copied().max().unwrap_or(0).saturating_add(1);
    for vertex in branch {
        let mut next = colors.clone();
        next[vertex] = marker;
        if !individualize(k, inside, hist, next, candidates, leaf_count, cost, limit) {
            return false;
        }
    }
    true
}

fn refine_colors(
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
    colors: &mut Vec<u16>,
    cost: &mut u64,
    limit: u64,
) -> bool {
    loop {
        if !charge(
            cost,
            (4 * (k * k + k * hist.len() * (k + 1)) + 16 * k) as u64,
            limit,
        ) {
            return false;
        }
        let mut descriptors: Vec<((u16, Vec<u16>, Vec<(Vec<u16>, u32)>), usize)> =
            Vec::with_capacity(k);
        for v in 0..k {
            let mut adj = Vec::new();
            for u in 0..k {
                if inside[v] & (1u16 << u) != 0 {
                    adj.push(colors[u]);
                }
            }
            adj.sort_unstable();

            let mut boundary = Vec::new();
            for &(mask, count) in hist {
                if mask & (1u16 << v) == 0 {
                    continue;
                }
                let mut shape = Vec::new();
                for (u, &color) in colors.iter().enumerate().take(k) {
                    if mask & (1u16 << u) != 0 {
                        shape.push(color);
                    }
                }
                shape.sort_unstable();
                boundary.push((shape, count));
            }
            boundary.sort_unstable();
            descriptors.push(((colors[v], adj, boundary), v));
        }
        descriptors.sort_unstable_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

        let mut next = vec![0u16; k];
        let mut color = 0u16;
        for i in 0..descriptors.len() {
            if i > 0 && descriptors[i].0 != descriptors[i - 1].0 {
                color += 1;
            }
            next[descriptors[i].1] = color;
        }
        if next == *colors {
            return true;
        }
        *colors = next;
    }
}

fn color_cells(k: usize, colors: &[u16]) -> Vec<Vec<usize>> {
    let mut by_color: Vec<(u16, usize)> = (0..k).map(|v| (colors[v], v)).collect();
    by_color.sort_unstable();
    let mut cells: Vec<Vec<usize>> = Vec::new();
    for (color, vertex) in by_color {
        if cells.last().is_some_and(|cell| colors[cell[0]] == color) {
            cells.last_mut().unwrap().push(vertex);
        } else {
            cells.push(vec![vertex]);
        }
    }
    cells
}

fn transformed_key(
    k: usize,
    inside: &[u16; MAX_WIDTH],
    hist: &[(u16, u32)],
    raw_for_canon: &[u8; MAX_WIDTH],
) -> CanonicalKey {
    let mut out_inside = [0u16; MAX_WIDTH];
    for canon in 0..k {
        let raw = raw_for_canon[canon] as usize;
        let mut mask = 0u16;
        for (other, &raw_other) in raw_for_canon.iter().enumerate().take(k) {
            if inside[raw] & (1u16 << raw_other) != 0 {
                mask |= 1u16 << other;
            }
        }
        out_inside[canon] = mask;
    }

    let mut out_hist = Vec::with_capacity(hist.len());
    for &(mask, count) in hist {
        let mut mapped = 0u16;
        for (canon, &raw) in raw_for_canon.iter().enumerate().take(k) {
            if mask & (1u16 << raw) != 0 {
                mapped |= 1u16 << canon;
            }
        }
        out_hist.push((mapped, count));
    }
    out_hist.sort_unstable();
    let mut deduped: Vec<(u16, u32)> = Vec::with_capacity(out_hist.len());
    for (mask, count) in out_hist {
        if let Some((last_mask, last_count)) = deduped.last_mut() {
            if *last_mask == mask {
                *last_count += count;
                continue;
            }
        }
        deduped.push((mask, count));
    }

    CanonicalKey {
        k: k as u8,
        inside: out_inside,
        hist: deduped,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn inside(k: usize, edges: &[(usize, usize)]) -> [u16; MAX_WIDTH] {
        let mut inside = [0u16; MAX_WIDTH];
        for &(a, b) in edges {
            assert!(a < k && b < k);
            inside[a] |= 1u16 << b;
            inside[b] |= 1u16 << a;
        }
        inside
    }

    fn relabel(
        k: usize,
        inside: &[u16; MAX_WIDTH],
        hist: &[(u16, u32)],
        old_for_new: &[usize],
    ) -> ([u16; MAX_WIDTH], Vec<(u16, u32)>) {
        let mut mapped_inside = [0u16; MAX_WIDTH];
        for new_v in 0..k {
            let old_v = old_for_new[new_v];
            for (new_u, &old_u) in old_for_new.iter().enumerate().take(k) {
                if inside[old_v] & (1u16 << old_u) != 0 {
                    mapped_inside[new_v] |= 1u16 << new_u;
                }
            }
        }
        let mut mapped_hist = Vec::new();
        for &(mask, count) in hist {
            let mut mapped = 0u16;
            for (new_v, &old_v) in old_for_new.iter().enumerate().take(k) {
                if mask & (1u16 << old_v) != 0 {
                    mapped |= 1u16 << new_v;
                }
            }
            mapped_hist.push((mapped, count));
        }
        mapped_hist.sort_unstable();
        (mapped_inside, mapped_hist)
    }

    fn cost(k: usize, inside: &[u16; MAX_WIDTH], hist: &[(u16, u32)], order: &[usize]) -> u64 {
        let mut parent: Vec<usize> = (0..k).collect();
        let mut component = vec![0u16; k];
        let mut nbr_union = vec![0u16; k];
        let mut active = 0u16;
        let mut total = 0u64;

        fn root(parent: &mut [usize], mut v: usize) -> usize {
            while parent[v] != v {
                parent[v] = parent[parent[v]];
                v = parent[v];
            }
            v
        }

        for &pivot in order {
            parent[pivot] = pivot;
            component[pivot] = 1u16 << pivot;
            nbr_union[pivot] = inside[pivot];
            let mut r = pivot;
            let mut neighbors = inside[pivot] & active;
            while neighbors != 0 {
                let u = neighbors.trailing_zeros() as usize;
                neighbors &= neighbors - 1;
                let ru = root(&mut parent, u);
                let rr = root(&mut parent, r);
                if ru != rr {
                    parent[ru] = rr;
                    component[rr] |= component[ru];
                    nbr_union[rr] |= nbr_union[ru];
                }
                r = root(&mut parent, r);
            }
            let c = component[r];
            let internal = (nbr_union[r] & !c).count_ones() as u64;
            let external: u64 = hist
                .iter()
                .filter_map(|&(mask, count)| (mask & c != 0).then_some(count as u64))
                .sum();
            let width = 1 + internal + external;
            total += width * width;
            active |= 1u16 << pivot;
        }
        total
    }

    fn best_cost(k: usize, inside: &[u16; MAX_WIDTH], hist: &[(u16, u32)]) -> u64 {
        fn visit(
            k: usize,
            inside: &[u16; MAX_WIDTH],
            hist: &[(u16, u32)],
            perm: &mut [usize],
            start: usize,
            best: &mut u64,
        ) {
            if start == k {
                *best = (*best).min(cost(k, inside, hist, perm));
                return;
            }
            for i in start..k {
                perm.swap(start, i);
                visit(k, inside, hist, perm, start + 1, best);
                perm.swap(start, i);
            }
        }

        let mut perm: Vec<_> = (0..k).collect();
        let mut best = u64::MAX;
        visit(k, inside, hist, &mut perm, 0, &mut best);
        best
    }

    #[test]
    fn relabelled_signature_canonicalizes_to_same_key() {
        let k = 5;
        let a = inside(k, &[(0, 1), (1, 2), (2, 3), (2, 4), (0, 4)]);
        let hist = vec![(0b0_0011, 2), (0b1_0100, 1), (0b0_1000, 3)];
        let (b, hist_b) = relabel(k, &a, &hist, &[2, 0, 4, 1, 3]);

        let key_a = canonical_key_for_test(k, &a, &hist).unwrap();
        let key_b = canonical_key_for_test(k, &b, &hist_b).unwrap();
        assert_eq!(key_a, key_b);
    }

    #[test]
    fn non_equivalent_halos_keep_distinct_canonical_keys() {
        let k = 4;
        let path = inside(k, &[(0, 1), (1, 2), (2, 3)]);
        let paired_edges = vec![(0b0011, 1), (0b1100, 1)];
        let crossed_edges = vec![(0b0101, 1), (0b1010, 1)];

        let key_a = canonical_key_for_test(k, &path, &paired_edges).unwrap();
        let key_b = canonical_key_for_test(k, &path, &crossed_edges).unwrap();
        assert_ne!(key_a, key_b);
        assert_ne!(
            best_cost(k, &path, &paired_edges),
            best_cost(k, &path, &crossed_edges)
        );
    }

    #[test]
    fn candidate_cap_skips_large_symmetric_partition() {
        let k = 8;
        let empty = [0u16; MAX_WIDTH];
        let attempt = canonicalize_limited(k, &empty, &[], u64::MAX);
        assert_eq!(attempt.key, None);
        assert_eq!(attempt.skip, Some(SkipReason::Candidates));
        assert!(attempt.candidates <= MAX_CANDIDATES as u64);
    }

    #[test]
    fn work_cap_stops_before_canonicalization_finishes() {
        let empty = [0u16; MAX_WIDTH];
        for budget in [0, 1, 100, 1_000] {
            let attempt = canonicalize_limited(8, &empty, &[], budget);
            assert_eq!(attempt.key, None);
            assert_eq!(attempt.skip, Some(SkipReason::Work));
            assert!(attempt.cost <= budget);
        }
    }
}
