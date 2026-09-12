//! Four workers across nested calls; results retain input order.
use std::sync::atomic::{AtomicUsize, Ordering};

static AVAILABLE: AtomicUsize = AtomicUsize::new(3);

struct Worker;
impl Drop for Worker {
    fn drop(&mut self) {
        AVAILABLE.fetch_add(1, Ordering::Release);
    }
}

pub(super) fn map_indexed<T: Sync, R: Send>(
    items: &[T],
    threads: usize,
    f: impl Fn(usize, &T) -> R + Sync,
) -> Vec<R> {
    let threads = threads.clamp(1, 4).min(items.len().max(1));
    if threads == 1 {
        return items
            .iter()
            .enumerate()
            .map(|(i, item)| f(i, item))
            .collect();
    }
    let next = AtomicUsize::new(0);
    let mut slots: Vec<Option<R>> = (0..items.len()).map(|_| None).collect();
    let run = || {
        let mut results = Vec::new();
        loop {
            let i = next.fetch_add(1, Ordering::Relaxed);
            if i >= items.len() {
                break;
            }
            results.push((i, f(i, &items[i])));
        }
        results
    };
    std::thread::scope(|scope| {
        let mut workers = Vec::with_capacity(threads);
        loop {
            while workers.len() + 1 < threads {
                if AVAILABLE
                    .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| n.checked_sub(1))
                    .is_err()
                {
                    break;
                }
                let worker = Worker;
                let run = &run;
                workers.push(scope.spawn(move || {
                    let _worker = worker;
                    run()
                }));
            }
            let i = next.fetch_add(1, Ordering::Relaxed);
            if i >= items.len() {
                break;
            }
            slots[i] = Some(f(i, &items[i]));
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
