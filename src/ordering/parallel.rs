//! Deterministic fixed-width parallel map. Each item is processed exactly once
//! by a pure function of the item and its index; results are returned in item
//! order, so thread scheduling never influences the output.
use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) fn map_indexed<T: Sync, R: Send>(
    items: &[T],
    threads: usize,
    f: impl Fn(usize, &T) -> R + Sync,
) -> Vec<R> {
    let threads = threads.max(1).min(items.len().max(1));
    if threads == 1 {
        return items
            .iter()
            .enumerate()
            .map(|(i, item)| f(i, item))
            .collect();
    }
    let next = AtomicUsize::new(0);
    let mut slots: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
    std::thread::scope(|scope| {
        let mut workers = Vec::with_capacity(threads);
        for _ in 0..threads {
            workers.push(scope.spawn(|| {
                let mut results = Vec::new();
                loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= items.len() {
                        break;
                    }
                    results.push((i, f(i, &items[i])));
                }
                results
            }));
        }
        for worker in workers {
            for (i, result) in worker.join().unwrap() {
                slots[i] = Some(result);
            }
        }
    });
    slots
        .into_iter()
        .map(|r| r.expect("every task completes"))
        .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn preserves_item_order() {
        let items: Vec<usize> = (0..1000).collect();
        let out = super::map_indexed(&items, 7, |i, &v| {
            assert_eq!(i, v);
            v * v
        });
        assert_eq!(out, items.iter().map(|v| v * v).collect::<Vec<_>>());
    }
}
