use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    mpsc, Arc, Mutex,
};
use std::thread;
use std::time::{Duration, Instant};

use crate::types::{
    QueueSnapshot, RunSummary, Scenario, Task, TaskResult, TimeStats, STEAL_LOWER_BOUNDARY,
    STEAL_THRESHOLD, TASKS_COUNT, WORKERS,
};
use crate::workload::do_work;

fn run_task(
    run_id: usize,
    worker_id: usize,
    task: Task,
    tx: &mpsc::Sender<TaskResult>,
    remaining_tasks: &AtomicUsize,
    start_total: Instant,
) {
    let start_time_us = start_total.elapsed().as_micros();
    let execution_start = Instant::now();

    // CPU work phase 1:
    // task.work_units now represents matrix size N,
    // so do_work(300) means one 300x300 matrix multiplication.
    let cpu_start_1 = Instant::now();
    do_work(task.work_units);
    let cpu_work_1_us = cpu_start_1.elapsed().as_micros();

    // Blocking phase
    let sleep_start = Instant::now();
    thread::sleep(Duration::from_micros(task.blocking_time_us));
    let blocking_time_real_us = sleep_start.elapsed().as_micros();

    // CPU work phase 2:
    // same matrix size again after blocking.
    let cpu_start_2 = Instant::now();
    do_work(task.work_units);
    let cpu_work_2_us = cpu_start_2.elapsed().as_micros();

    let execution_time_us = execution_start.elapsed().as_micros();
    let end_time_us = start_total.elapsed().as_micros();

    remaining_tasks.fetch_sub(1, Ordering::SeqCst);

    let waiting_time_us = start_time_us.saturating_sub(task.created_time_us);
    let response_time_us = end_time_us.saturating_sub(task.created_time_us);
    let cpu_work_time_us = cpu_work_1_us + cpu_work_2_us;

    let task_kind = if task.work_units >= 1024 {
        "long"
    } else {
        "short"
    };

    tx.send(TaskResult {
        run_id,
        worker_id,
        task_id: task.id,
        task_kind,
        work_units: task.work_units,
        blocking_time_configured_us: task.blocking_time_us,
        stolen: task.stolen_from.is_some(),
        stolen_from: task.stolen_from,

        created_time_us: task.created_time_us,
        start_time_us,
        end_time_us,

        waiting_time_us,
        blocking_time_real_us,
        cpu_work_time_us,
        execution_time_us,
        response_time_us,
    })
    .unwrap();
}

fn steal_until_threshold(worker_id: usize, queues: &Vec<Arc<Mutex<VecDeque<Task>>>>) {
    loop {
        let own_len = queues[worker_id].lock().unwrap().len();

        if own_len >= STEAL_THRESHOLD {
            break;
        }

        let mut stolen_anything = false;

        for victim_id in 0..queues.len() {
            if victim_id == worker_id {
                continue;
            }

            let stolen_task = {
                let mut victim_queue = queues[victim_id].lock().unwrap();

                if victim_queue.len() <= STEAL_LOWER_BOUNDARY {
                    None
                } else {
                    victim_queue.pop_front()
                }
            };

            if let Some(mut task) = stolen_task {
                task.stolen_from = Some(victim_id);
                queues[worker_id].lock().unwrap().push_back(task);

                stolen_anything = true;
                break;
            }
        }

        if !stolen_anything {
            break;
        }
    }
}

fn create_task(id: usize, start_total: Instant, blocking_time_us: u64) -> Task {
    let work_units = if id % 5 == 0 {
        1024
    } else {
        512
    };

    Task {
        id,
        work_units,
        blocking_time_us,
        created_time_us: start_total.elapsed().as_micros(),
        stolen_from: None,
    }
}

fn push_task(
    queues: &Vec<Arc<Mutex<VecDeque<Task>>>>,
    remaining_tasks: &AtomicUsize,
    task: Task,
    target_worker: usize,
) {
    remaining_tasks.fetch_add(1, Ordering::SeqCst);
    queues[target_worker].lock().unwrap().push_back(task);
}

fn producer_thread(
    scenario: Scenario,
    queues: Arc<Vec<Arc<Mutex<VecDeque<Task>>>>>,
    remaining_tasks: Arc<AtomicUsize>,
    producer_done: Arc<AtomicBool>,
    start_total: Instant,
    blocking_time_us: u64,
) {
    match scenario {
        Scenario::Regular => {
            for id in 0..TASKS_COUNT {
                let task = create_task(id, start_total, blocking_time_us);
                let target_worker = id % WORKERS;

                push_task(&queues, &remaining_tasks, task, target_worker);
                thread::sleep(Duration::from_micros(15_000));
            }
        }

        Scenario::Burst => {
            let burst_size = 50;
            let mut id = 0;

            while id < TASKS_COUNT {
                for _ in 0..burst_size {
                    if id >= TASKS_COUNT {
                        break;
                    }

                    let task = create_task(id, start_total, blocking_time_us);
                    push_task(&queues, &remaining_tasks, task, 0);
                    id += 1;
                }

                thread::sleep(Duration::from_micros(200_000));
            }
        }

        Scenario::Mixed => {
            for id in 0..TASKS_COUNT {
                let task = create_task(id, start_total, blocking_time_us);

                if id % 20 < 10 {
                    push_task(&queues, &remaining_tasks, task, 0);
                } else {
                    let target_worker = id % WORKERS;
                    push_task(&queues, &remaining_tasks, task, target_worker);
                    thread::sleep(Duration::from_micros(20_000));
                }
            }
        }

        Scenario::Random => {
            let mut seed: u64 = 123456789;

            for id in 0..TASKS_COUNT {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

                let target_worker = (seed as usize) % WORKERS;
                let sleep_us = 5_000 + (seed % 40_000);
                let task = create_task(id, start_total, blocking_time_us);

                push_task(&queues, &remaining_tasks, task, target_worker);
                thread::sleep(Duration::from_micros(sleep_us));
            }
        }
    }

    producer_done.store(true, Ordering::SeqCst);
}

fn time_stats<I>(values: I) -> TimeStats
where
    I: Iterator<Item = u128>,
{
    let values: Vec<u128> = values.collect();

    if values.is_empty() {
        return TimeStats {
            min: 0,
            max: 0,
            mean: 0,
        };
    }

    let min = *values.iter().min().unwrap();
    let max = *values.iter().max().unwrap();
    let sum: u128 = values.iter().sum();
    let mean = sum / values.len() as u128;

    TimeStats { min, max, mean }
}

pub fn run_benchmark(
    run_id: usize,
    scenario: Scenario,
    blocking_time_us: u64,
) -> (Vec<TaskResult>, Vec<QueueSnapshot>, RunSummary) {
    let mut queues: Vec<Arc<Mutex<VecDeque<Task>>>> = Vec::new();

    for _ in 0..WORKERS {
        queues.push(Arc::new(Mutex::new(VecDeque::new())));
    }

    let queues = Arc::new(queues);
    let remaining_tasks = Arc::new(AtomicUsize::new(0));
    let producer_done = Arc::new(AtomicBool::new(false));
    let sampler_done = Arc::new(AtomicBool::new(false));
    let snapshots = Arc::new(Mutex::new(Vec::new()));

    let (tx, rx) = mpsc::channel();
    let start_total = Instant::now();

    let producer_handle = {
        let queues = Arc::clone(&queues);
        let remaining_tasks = Arc::clone(&remaining_tasks);
        let producer_done = Arc::clone(&producer_done);

        thread::spawn(move || {
            producer_thread(
                scenario,
                queues,
                remaining_tasks,
                producer_done,
                start_total,
                blocking_time_us,
            );
        })
    };

    let sampler_handle = {
        let queues = Arc::clone(&queues);
        let sampler_done = Arc::clone(&sampler_done);
        let snapshots = Arc::clone(&snapshots);

        thread::spawn(move || {
            while !sampler_done.load(Ordering::SeqCst) {
                let mut lengths = Vec::new();

                for q in queues.iter() {
                    lengths.push(q.lock().unwrap().len());
                }

                snapshots.lock().unwrap().push(QueueSnapshot {
                    run_id,
                    time_us: start_total.elapsed().as_micros(),
                    queue_lengths: lengths,
                });

                thread::sleep(Duration::from_micros(5_000));
            }
        })
    };

    let mut worker_handles = Vec::new();

    for worker_id in 0..WORKERS {
        let all_queues = Arc::clone(&queues);
        let worker_tx = tx.clone();
        let worker_remaining = Arc::clone(&remaining_tasks);
        let worker_producer_done = Arc::clone(&producer_done);

        let handle = thread::spawn(move || {
            while !worker_producer_done.load(Ordering::SeqCst)
                || worker_remaining.load(Ordering::SeqCst) > 0
            {
                steal_until_threshold(worker_id, &all_queues);

                let local_task = {
                    let mut own_queue = all_queues[worker_id].lock().unwrap();
                    own_queue.pop_back()
                };

                if let Some(task) = local_task {
                    run_task(
                        run_id,
                        worker_id,
                        task,
                        &worker_tx,
                        &worker_remaining,
                        start_total,
                    );
                } else {
                    thread::yield_now();
                }
            }
        });

        worker_handles.push(handle);
    }

    drop(tx);

    producer_handle.join().unwrap();

    for handle in worker_handles {
        handle.join().unwrap();
    }

    sampler_done.store(true, Ordering::SeqCst);
    sampler_handle.join().unwrap();

    let total_time_us = start_total.elapsed().as_micros();

    let mut results = Vec::new();

    while let Ok(result) = rx.recv() {
        results.push(result);
    }

    let snapshots = snapshots.lock().unwrap().clone();

    let stolen_count = results.iter().filter(|r| r.stolen).count();
    let long_tasks = results.iter().filter(|r| r.task_kind == "long").count();
    let short_tasks = results.iter().filter(|r| r.task_kind == "short").count();

    let summary = RunSummary {
        run_id,
        scenario,
        total_time_us,
        stolen_count,
        long_tasks,
        short_tasks,
        waiting: time_stats(results.iter().map(|r| r.waiting_time_us)),
        blocking: time_stats(results.iter().map(|r| r.blocking_time_real_us)),
        cpu_work: time_stats(results.iter().map(|r| r.cpu_work_time_us)),
        execution: time_stats(results.iter().map(|r| r.execution_time_us)),
        response: time_stats(results.iter().map(|r| r.response_time_us)),
    };

    (results, snapshots, summary)
}
