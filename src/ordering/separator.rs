//! Connected elimination-tree separators. Hanging branches are represented by
//! their exact boundary cliques. A tree DP combines compatible replacements.
use super::recovery_separator::{assemble, greedy, Proposal, Tree};
use crate::Pattern;
use std::collections::BTreeSet;

pub(super) fn refine(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, false, false)
}
pub(super) fn refine_large(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, true, false)
}
pub(super) fn refine_core(pattern: &Pattern, incumbent: &[usize]) -> Option<Vec<usize>> {
    refine_impl(pattern, incumbent, true, true)
}
fn refine_impl(
    pattern: &Pattern,
    incumbent: &[usize],
    large: bool,
    core: bool,
) -> Option<Vec<usize>> {
    let n = pattern.n;
    if !(1000..=80_000).contains(&n) || pattern.nnz() > 500_000 {
        return None;
    }
    if large && !core && n < 8000 {
        return None;
    }
    let tree = Tree::build_limited(
        pattern,
        incumbent,
        4_000_000,
        if large { 256 * n as u64 } else { 0 },
    )?;
    let roots = tree.roots_by_cap(
        512,
        if core {
            1
        } else if large {
            3
        } else {
            12
        },
    );
    let mut proposals: Vec<Vec<Proposal>> = (0..n).map(|_| Vec::new()).collect();
    let mut work = 24_000_000usize;
    let mut map = vec![usize::MAX; n];
    let mut metis_tickets = 2usize;
    let mut joint_tickets = 2usize;
    let mut flip_tickets = 1usize;
    let mut seen_regions = BTreeSet::new();
    let caps: &[usize] = if large {
        &[8192, 2048, 512, 128]
    } else {
        &[512, 1024, 256]
    };
    for &cap in caps {
        for &root in &roots {
            if work < 10000 {
                break;
            }
            let vertices = tree.grow(root, cap, 0);
            if !seen_regions.insert((root, vertices.clone())) {
                continue;
            }
            let k = vertices.len();
            let Some((children, g, total)) = tree.compress(root, &vertices, &mut map, &mut work)
            else {
                continue;
            };
            let w = total.div_ceil(64);
            let mut best: Option<(Vec<usize>, u64)> = None;
            let original = tree.original(&vertices);
            if large && k >= 8 && k <= 512 && total <= 1024 && joint_tickets > 0 {
                let mut selected: Vec<_> = (0..k).collect();
                selected.sort_by_key(|&v| {
                    (
                        std::cmp::Reverse(
                            g[v * w..(v + 1) * w]
                                .iter()
                                .map(|a| a.count_ones())
                                .sum::<u32>(),
                        ),
                        v,
                    )
                });
                selected.truncate(8);
                joint_tickets -= 1;
                if let Some((order, cost)) = super::joint::schedule(&g, total, k, &selected) {
                    if cost < original {
                        best = Some((order, cost));
                    }
                }
            }
            if large {
                let (cp, ri) = super::recovery_separator::csc_of(&g, total);
                if ri.len() <= 1_000_000 {
                    let cc: Vec<i32> = cp.iter().map(|&x| x as i32).collect();
                    let rr: Vec<i32> = ri.iter().map(|&x| x as i32).collect();
                    if let Some(graph) = feral_ordering_core::CscPattern::new(total, &cc, &rr) {
                        let sources: &[usize] = if metis_tickets > 0 {
                            metis_tickets -= 1;
                            &[0, 1, 2]
                        } else {
                            &[0, 1]
                        };
                        let candidates = super::map_indexed(sources, |_, &source| match source {
                            0 => super::quotient::order(
                                &graph,
                                10.0,
                                true,
                                true,
                                super::quotient::WORK,
                            ),
                            1 => super::quotient::order(
                                &graph,
                                10.0,
                                true,
                                false,
                                super::quotient::WORK,
                            ),
                            _ => feral_metis::metis_order_full(
                                &graph,
                                &feral_metis::MetisOptions {
                                    niparts: 4,
                                    ..Default::default()
                                },
                            )
                            .map(|(p, ..)| p),
                        });
                        let local_pattern = Pattern {
                            n: total,
                            col_ptr: cp.clone(),
                            row_idx: ri.clone(),
                        };
                        let sp = super::ScoringPattern {
                            n: total,
                            col_ptr: cp,
                            row_idx: ri,
                        };
                        for result in candidates {
                            if let Ok(order) = result {
                                let inside: Vec<usize> = order
                                    .into_iter()
                                    .map(|v| v as usize)
                                    .filter(|&v| v < k)
                                    .collect();
                                if !super::is_bijection(&inside, k) {
                                    continue;
                                }
                                let mut full = inside.clone();
                                full.extend(k..total);
                                let b = (total - k) as u64;
                                let cost = super::flops_of(&sp, &full)
                                    .checked_sub(b * (b + 1) * (2 * b + 1) / 6)?;
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((inside, cost));
                                }
                            }
                        }
                        if k <= 512 {
                            if let Some((order, cost)) = greedy(g.clone(), total, k, 3, &mut work) {
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((order, cost));
                                }
                            }
                        }
                        let mut initial = best
                            .as_ref()
                            .map_or_else(|| (0..k).collect::<Vec<_>>(), |b| b.0.clone());
                        initial.extend(k..total);
                        if k <= 512 && total <= 1200 && flip_tickets > 0 {
                            flip_tickets -= 1;
                            let permuted = super::permute_pattern(&sp, &initial);
                            let tree = super::EliminationTree::from_pattern(&permuted);
                            let counts: Vec<u32> = super::symbolic_counts(&permuted, &tree)
                                .1
                                .into_iter()
                                .map(|v| v as u32)
                                .collect();
                            if let Some(candidates) = super::completion::flip_candidates(
                                total,
                                &local_pattern.col_ptr,
                                &local_pattern.row_idx,
                                &initial,
                                &counts,
                                k,
                            ) {
                                for q in candidates {
                                    let b = (total - k) as u64;
                                    let cost =
                                        super::flops_of(&sp, &q) - b * (b + 1) * (2 * b + 1) / 6;
                                    if cost < best.as_ref().map_or(original, |b| b.1) {
                                        best = Some((q[..k].to_vec(), cost));
                                        initial = q;
                                    }
                                }
                            }
                        }
                        if let Some(order) =
                            super::promotions::boundary(&local_pattern, &initial, k)
                        {
                            if order[..k].iter().all(|&v| v < k) {
                                let b = (total - k) as u64;
                                let cost =
                                    super::flops_of(&sp, &order) - b * (b + 1) * (2 * b + 1) / 6;
                                if cost < best.as_ref().map_or(original, |b| b.1) {
                                    best = Some((order[..k].to_vec(), cost));
                                }
                            }
                        }
                    }
                }
            }
            for mode in if large { Vec::new() } else { vec![3, 1, 2, 4] } {
                let Some((order, cost)) = greedy(g.clone(), total, k, mode, &mut work) else {
                    break;
                };
                if cost < best.as_ref().map_or(original, |b| b.1) {
                    best = Some((order, cost));
                }
            }
            if let Some((order, cost)) = best {
                proposals[root].push(Proposal {
                    root,
                    order: order.iter().map(|&j| vertices[j]).collect(),
                    children,
                    cost,
                    old: original,
                    skeleton: vertices,
                });
            }
        }
    }
    assemble(&tree, &proposals)
}
