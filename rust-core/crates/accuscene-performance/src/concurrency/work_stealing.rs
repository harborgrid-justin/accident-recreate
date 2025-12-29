//! Work-stealing thread pool for parallel task execution

use crossbeam::deque::{Injector, Stealer, Worker};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;

/// Task to be executed by the work-stealing pool
pub trait WorkStealingTask: Send + 'static {
    /// Execute the task
    fn execute(self: Box<Self>);
}

impl<F> WorkStealingTask for F
where
    F: FnOnce() + Send + 'static,
{
    fn execute(self: Box<Self>) {
        self()
    }
}

/// Work-stealing thread pool
pub struct WorkStealingPool {
    injector: Arc<Injector<Box<dyn WorkStealingTask>>>,
    stealers: Arc<Vec<Stealer<Box<dyn WorkStealingTask>>>>,
    threads: Vec<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
    active_tasks: Arc<AtomicUsize>,
}

impl WorkStealingPool {
    /// Create a new work-stealing pool
    pub fn new(num_threads: usize) -> Self {
        let injector = Arc::new(Injector::new());
        let shutdown = Arc::new(AtomicBool::new(false));
        let active_tasks = Arc::new(AtomicUsize::new(0));

        let mut stealers = Vec::new();

        // Create workers and stealers
        let mut workers_with_stealers = Vec::new();
        for _ in 0..num_threads {
            let worker = Worker::new_fifo();
            stealers.push(worker.stealer());
            workers_with_stealers.push(worker);
        }

        let stealers = Arc::new(stealers);
        let mut threads = Vec::new();

        // Spawn worker threads
        for (i, worker) in workers_with_stealers.into_iter().enumerate() {
            let injector = injector.clone();
            let stealers = stealers.clone();
            let shutdown = shutdown.clone();
            let active_tasks = active_tasks.clone();

            let handle = thread::Builder::new()
                .name(format!("work-stealing-{}", i))
                .spawn(move || {
                    Self::worker_loop(worker, injector, stealers, shutdown, active_tasks);
                })
                .expect("Failed to spawn worker thread");

            threads.push(handle);
        }

        Self {
            injector,
            stealers,
            threads,
            shutdown,
            active_tasks,
        }
    }

    /// Submit a task to the pool
    pub fn submit<T: WorkStealingTask>(&self, task: T) {
        self.active_tasks.fetch_add(1, Ordering::SeqCst);
        self.injector.push(Box::new(task));
    }

    /// Submit a closure to the pool
    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.submit(f);
    }

    /// Get the number of active tasks
    pub fn active_tasks(&self) -> usize {
        self.active_tasks.load(Ordering::SeqCst)
    }

    /// Wait for all tasks to complete
    pub fn wait(&self) {
        while self.active_tasks.load(Ordering::SeqCst) > 0 {
            thread::yield_now();
        }
    }

    /// Worker thread loop
    fn worker_loop(
        worker: Worker<Box<dyn WorkStealingTask>>,
        injector: Arc<Injector<Box<dyn WorkStealingTask>>>,
        stealers: Arc<Vec<Stealer<Box<dyn WorkStealingTask>>>>,
        shutdown: Arc<AtomicBool>,
        active_tasks: Arc<AtomicUsize>,
    ) {
        while !shutdown.load(Ordering::SeqCst) {
            // Try to get task from local queue
            if let Some(task) = worker.pop() {
                task.execute();
                active_tasks.fetch_sub(1, Ordering::SeqCst);
                continue;
            }

            // Try to get task from global injector
            loop {
                match injector.steal_batch_and_pop(&worker) {
                    crossbeam::deque::Steal::Success(task) => {
                        task.execute();
                        active_tasks.fetch_sub(1, Ordering::SeqCst);
                        break;
                    }
                    crossbeam::deque::Steal::Empty => break,
                    crossbeam::deque::Steal::Retry => {}
                }
            }

            // Try to steal from other workers
            for stealer in stealers.iter() {
                loop {
                    match stealer.steal_batch_and_pop(&worker) {
                        crossbeam::deque::Steal::Success(task) => {
                            task.execute();
                            active_tasks.fetch_sub(1, Ordering::SeqCst);
                            break;
                        }
                        crossbeam::deque::Steal::Empty => break,
                        crossbeam::deque::Steal::Retry => {}
                    }
                }
            }

            // No work available, yield
            thread::yield_now();
        }
    }

    /// Shutdown the pool
    pub fn shutdown(self) {
        self.shutdown.store(true, Ordering::SeqCst);

        for handle in self.threads {
            handle.join().expect("Worker thread panicked");
        }
    }
}

/// Parallel iterator using work-stealing
pub struct ParallelIter<I> {
    iter: Mutex<I>,
    pool: Arc<WorkStealingPool>,
}

impl<I> ParallelIter<I>
where
    I: Iterator + Send + 'static,
    I::Item: Send + 'static,
{
    /// Create a new parallel iterator
    pub fn new(iter: I, pool: Arc<WorkStealingPool>) -> Self {
        Self {
            iter: Mutex::new(iter),
            pool,
        }
    }

    /// Execute a function on each item in parallel
    pub fn for_each<F>(self, f: F)
    where
        F: Fn(I::Item) + Send + Sync + 'static,
    {
        let f = Arc::new(f);

        loop {
            let item = {
                let mut iter = self.iter.lock();
                iter.next()
            };

            match item {
                Some(item) => {
                    let f = f.clone();
                    self.pool.execute(move || f(item));
                }
                None => break,
            }
        }

        self.pool.wait();
    }
}

/// Batch work for parallel execution
pub struct BatchWork<T> {
    items: Vec<T>,
    batch_size: usize,
}

impl<T: Send + Clone + 'static> BatchWork<T> {
    /// Create new batch work
    pub fn new(items: Vec<T>, batch_size: usize) -> Self {
        Self { items, batch_size }
    }

    /// Execute a function on batches in parallel
    pub fn execute<F>(self, pool: &WorkStealingPool, f: F)
    where
        F: Fn(Vec<T>) + Send + Sync + 'static,
    {
        let f = Arc::new(f);

        for chunk in self.items.chunks(self.batch_size) {
            let batch = chunk.to_vec();
            let f = f.clone();
            pool.execute(move || f(batch));
        }

        pool.wait();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicUsize;

    #[test]
    fn test_work_stealing_pool() {
        let pool = WorkStealingPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        for _ in 0..100 {
            let counter = counter.clone();
            pool.execute(move || {
                counter.fetch_add(1, Ordering::SeqCst);
            });
        }

        pool.wait();
        assert_eq!(counter.load(Ordering::SeqCst), 100);

        pool.shutdown();
    }

    #[test]
    fn test_parallel_iter() {
        let pool = Arc::new(WorkStealingPool::new(4));
        let counter = Arc::new(AtomicUsize::new(0));

        let iter = ParallelIter::new(0..100, pool.clone());

        let counter_clone = counter.clone();
        iter.for_each(move |_| {
            counter_clone.fetch_add(1, Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 100);

        pool.wait();
    }

    #[test]
    fn test_batch_work() {
        let pool = WorkStealingPool::new(4);
        let counter = Arc::new(AtomicUsize::new(0));

        let items: Vec<usize> = (0..100).collect();
        let batch = BatchWork::new(items, 10);

        let counter_clone = counter.clone();
        batch.execute(&pool, move |batch| {
            counter_clone.fetch_add(batch.len(), Ordering::SeqCst);
        });

        assert_eq!(counter.load(Ordering::SeqCst), 100);

        pool.shutdown();
    }
}
