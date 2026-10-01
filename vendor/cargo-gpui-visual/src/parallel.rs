//! Order-preserving parallel map for independent per-file work (hashing,
//! decoding, exact comparison). Items never observe each other. Results keep
//! input order, and the reported error is always the first failing item in input
//! order, exactly as the former sequential loops: items are claimed in
//! increasing order and every item below the lowest failure is completed, so no
//! scheduling can hide an earlier failure behind a later one.
use anyhow::{anyhow, Result};
use std::sync::atomic::{AtomicUsize, Ordering};

/// Worker threads for `map`: the host's available parallelism.
pub fn workers() -> usize {
    std::thread::available_parallelism().map_or(1, |n| n.get())
}

pub fn map<T: Sync, R: Send>(
    items: &[T],
    operation: impl Fn(&T) -> Result<R> + Sync,
) -> Result<Vec<R>> {
    map_with(items, workers(), operation)
}

pub(crate) fn map_with<T: Sync, R: Send>(
    items: &[T],
    workers: usize,
    operation: impl Fn(&T) -> Result<R> + Sync,
) -> Result<Vec<R>> {
    let workers = workers.min(items.len());
    if workers <= 1 {
        return items.iter().map(operation).collect();
    }
    let next = AtomicUsize::new(0);
    let first_failure = AtomicUsize::new(usize::MAX);
    let mut slots: Vec<Option<Result<R>>> = (0..items.len()).map(|_| None).collect();
    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut done = Vec::new();
                    loop {
                        let index = next.fetch_add(1, Ordering::SeqCst);
                        // Work after a known failure cannot change the reported error.
                        if index >= items.len() || index > first_failure.load(Ordering::SeqCst) {
                            break;
                        }
                        let result = operation(&items[index]);
                        if result.is_err() {
                            first_failure.fetch_min(index, Ordering::SeqCst);
                        }
                        done.push((index, result));
                    }
                    done
                })
            })
            .collect();
        for handle in handles {
            let done = handle
                .join()
                .unwrap_or_else(|panic| std::panic::resume_unwind(panic));
            for (index, result) in done {
                slots[index] = Some(result);
            }
        }
    });
    let mut results = Vec::with_capacity(items.len());
    for (index, slot) in slots.into_iter().enumerate() {
        match slot {
            Some(Ok(value)) => results.push(value),
            Some(Err(error)) => return Err(error),
            // Only items after a recorded failure are skipped; that failure
            // appears earlier in this loop and has already returned.
            None => return Err(anyhow!("Parallel item {index} was not processed")),
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::bail;

    #[test]
    fn preserves_order_and_matches_sequential_results() {
        let items: Vec<u64> = (0..997).collect();
        let square = |n: &u64| Ok(n * n);
        let sequential: Vec<u64> = items.iter().map(square).collect::<Result<_>>().unwrap();
        for workers in [1, 2, 3, 8, 64] {
            assert_eq!(map_with(&items, workers, square).unwrap(), sequential);
        }
        assert!(map_with(&[] as &[u64], 8, square).unwrap().is_empty());
    }

    #[test]
    fn reports_the_first_failure_in_input_order() {
        let items: Vec<usize> = (0..500).collect();
        for workers in [1, 2, 7, 16] {
            let error = map_with(&items, workers, |n| {
                // A later item fails fast; an earlier one fails only after work.
                if *n == 400 {
                    bail!("late {n}");
                }
                if *n == 123 {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    bail!("early {n}");
                }
                Ok(*n)
            })
            .unwrap_err();
            assert_eq!(error.to_string(), "early 123", "{workers} workers");
        }
    }

    #[test]
    fn worker_panics_propagate() {
        let items: Vec<u32> = (0..64).collect();
        let panicked = std::panic::catch_unwind(|| {
            let _ = map_with(&items, 4, |n| {
                if *n == 17 {
                    panic!("item 17");
                }
                Ok(*n)
            });
        });
        assert!(panicked.is_err());
    }
}
