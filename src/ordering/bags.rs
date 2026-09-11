//! Restricted clique-bag DP. Donor elimination cliques supply separators;
//! connected components can be recombined across donors.
use std::collections::BTreeMap;

type Mask = Vec<u64>;
fn vertices(m: &[u64]) -> impl Iterator<Item = usize> + '_ {
    m.iter().enumerate().flat_map(|(j, &word)| {
        let mut bits = word;
        std::iter::from_fn(move || {
            if bits == 0 { return None; }
            let v = j * 64 + bits.trailing_zeros() as usize;
            bits &= bits - 1;
            Some(v)
        })
    })
}
fn has(m: &[u64], v: usize) -> bool { m[v / 64] & (1 << (v % 64)) != 0 }
fn subset(a: &[u64], b: &[u64]) -> bool { a.iter().zip(b).all(|(x,y)| x & !y == 0) }
fn overlap(a: &[u64], b: &[u64]) -> bool { a.iter().zip(b).any(|(x,y)| x & y != 0) }

struct Region {
    inside: Mask,
    boundary: Mask,
    closure: Mask,
    size: usize,
    cost: u64,
    choice: Option<usize>,
    first: Option<usize>,
}

pub(super) fn recombine(
    graph: &[u64], n: usize, k: usize, weights: &[u64], cliques: &[bool],
    bags: &[Mask], initial: &[usize],
) -> (u64, Vec<usize>) {
    let w = n.div_ceil(64);
    let weight = |m: &[u64]| vertices(m).map(|v| weights[v]).sum::<u64>();
    let root_cost = |bag: &[u64], boundary: u64, inside: &[u64], touched: &[u64]| {
        let total = weight(bag);
        let mut discount = 0;
        let mut first = None;
        for v in vertices(bag).filter(|&v| v < k && has(inside, v) && !cliques[v] && !has(touched, v)) {
            let d = total - weights[v];
            let gain = super::patch::sum_squares(total) - super::patch::sum_squares(d)
                - weights[v] * (d + 1).pow(2);
            if gain > discount { discount = gain; first = Some(v); }
        }
        (super::patch::sum_squares(total) - super::patch::sum_squares(boundary) - discount, first)
    };
    let components = |excluded: &[u64]| {
        let mut seen = vec![false; k];
        let mut result = Vec::new();
        for root in 0..k {
            if seen[root] || has(excluded, root) { continue; }
            seen[root] = true;
            let mut queue = vec![root];
            let mut mask = vec![0; w];
            let mut head = 0;
            while head < queue.len() {
                let v = queue[head]; head += 1;
                mask[v / 64] |= 1 << (v % 64);
                for u in vertices(&graph[v*w..(v+1)*w]).filter(|&u| u < k) {
                    if !seen[u] && !has(excluded, u) { seen[u] = true; queue.push(u); }
                }
            }
            result.push(mask);
        }
        result
    };
    let mut regions = Vec::<Region>::new();
    let mut ids = BTreeMap::new();
    let mut state = |inside: Mask| {
        if let Some(&id) = ids.get(&inside) { return id; }
        let mut boundary = vec![0; w];
        for v in vertices(&inside) {
            for j in 0..w { boundary[j] |= graph[v*w+j]; }
        }
        let mut closure = inside.clone();
        for j in 0..w { boundary[j] &= !inside[j]; closure[j] |= boundary[j]; }
        let (cost, first) = root_cost(&closure, weight(&boundary), &inside, &vec![0; w]);
        let id = regions.len();
        ids.insert(inside.clone(), id);
        let size = vertices(&inside).count();
        regions.push(Region { inside, boundary, closure, size, cost, first, choice: None });
        id
    };
    let roots: Vec<_> = components(&vec![0; w]).into_iter().map(&mut state).collect();
    let children: Vec<Vec<usize>> = bags.iter().map(|b| components(b).into_iter().map(&mut state).collect()).collect();
    let mut sequence: Vec<_> = (0..regions.len()).collect();
    sequence.sort_by_key(|&id| (regions[id].size, id));
    let mut containing = vec![Vec::new(); n];
    for (id, bag) in bags.iter().enumerate() {
        for v in vertices(bag) { containing[v].push(id); }
    }
    let all: Vec<_> = (0..bags.len()).collect();
    for id in sequence {
        let anchor = vertices(&regions[id].boundary).min_by_key(|&v| containing[v].len());
        let choices = anchor.map_or(&all, |v| &containing[v]);
        for &j in choices {
            let r = &regions[id];
            if !subset(&r.boundary, &bags[j]) || !subset(&bags[j], &r.closure) || !overlap(&bags[j], &r.inside) { continue; }
            let mut touched = vec![0; w];
            let mut sub = 0;
            let mut valid = true;
            for &child in &children[j] {
                let c = &regions[child];
                if !overlap(&c.inside, &r.inside) { continue; }
                if c.size >= r.size || !subset(&c.inside, &r.inside) || !subset(&c.boundary, &bags[j]) { valid = false; break; }
                sub += c.cost;
                for x in 0..w { touched[x] |= c.boundary[x]; }
            }
            if !valid { continue; }
            let (root, first) = root_cost(&bags[j], weight(&r.boundary), &r.inside, &touched);
            if sub + root < r.cost {
                regions[id].cost = sub + root;
                regions[id].choice = Some(j);
                regions[id].first = first;
            }
        }
    }
    let mut rank = vec![0; k];
    for (i, &v) in initial.iter().enumerate() { rank[v] = i; }
    let mut output = Vec::with_capacity(k);
    let bound = roots.iter().map(|&id| regions[id].cost).sum();
    let mut stack: Vec<_> = roots.into_iter().rev().map(|r| (r, false)).collect();
    while let Some((id, emit)) = stack.pop() {
        let r = &regions[id];
        if !emit {
            stack.push((id, true));
            if let Some(j) = r.choice {
                for &child in children[j].iter().rev() {
                    if overlap(&regions[child].inside, &r.inside) { stack.push((child, false)); }
                }
            }
        } else {
            let mut last: Vec<_> = vertices(&r.inside)
                .filter(|&v| r.choice.is_none_or(|j| has(&bags[j], v))).collect();
            last.sort_by_key(|&v| rank[v]);
            if let Some(first) = r.first {
                let at = last.iter().position(|&v| v == first).expect("bag first belongs to root");
                last[..=at].rotate_right(1);
            }
            output.extend(last);
        }
    }
    (bound, output)
}
