use std::sync::atomic::{AtomicUsize, Ordering};

pub(super) fn map<T: Sync, R: Send>(
    items: &[T],
    threads: usize,
    f: impl Fn(usize, &T, &mut Vec<usize>) -> R + Sync,
) -> Vec<R> {
    let threads = threads.clamp(1, 4).min(items.len()).max(1);
    if threads == 1 {
        let mut scratch = Vec::new();
        return items
            .iter()
            .enumerate()
            .map(|(i, x)| f(i, x, &mut scratch))
            .collect();
    }
    let next = AtomicUsize::new(0);
    let run = || {
        let mut scratch = Vec::new();
        let mut out = Vec::new();
        loop {
            let i = next.fetch_add(1, Ordering::Relaxed);
            if i >= items.len() {
                break;
            }
            out.push((i, f(i, &items[i], &mut scratch)));
        }
        out
    };
    let mut results = std::thread::scope(|scope| {
        let workers: Vec<_> = (1..threads).map(|_| scope.spawn(&run)).collect();
        let mut results = run();
        for worker in workers {
            results.extend(worker.join().unwrap());
        }
        results
    });
    results.sort_unstable_by_key(|(i, _)| *i);
    results.into_iter().map(|(_, result)| result).collect()
}
