mod benchmark;
mod output;
mod types;
mod workload;

use benchmark::run_benchmark;
use output::{
    print_summary, print_worker_summary, write_queue_csv, write_summary_csv, write_task_csv,
};
use types::{
    QueueSnapshot, RunSummary, Scenario, TaskResult, STEAL_LOWER_BOUNDARY, STEAL_THRESHOLD,
    TASKS_COUNT, WORKERS,
};

fn main() {
    let scenario = Scenario::from_arg(std::env::args().nth(1));

    let blocking_time_us: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    let repetitions: usize = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(1);

    let mut all_results: Vec<TaskResult> = Vec::new();
    let mut all_snapshots: Vec<QueueSnapshot> = Vec::new();
    let mut all_summaries: Vec<RunSummary> = Vec::new();

    println!("Dynamic work-stealing benchmark");
    println!("Scenario: {:?}", scenario);
    println!("Workers: {}", WORKERS);
    println!("Tasks per run: {}", TASKS_COUNT);
    println!("Steal threshold per worker: {}", STEAL_THRESHOLD);
    println!("Steal lower boundary: {}", STEAL_LOWER_BOUNDARY);
    println!("Configured blocking time per task: {} us", blocking_time_us);
    println!("Repetitions: {}", repetitions);

    for run_id in 1..=repetitions {
        println!();
        println!("================ Run {}/{} ================", run_id, repetitions);

        let (results, snapshots, summary) = run_benchmark(run_id, scenario, blocking_time_us);

        println!(
            "Run {} finished. Total time: {} us",
            run_id, summary.total_time_us
        );
        println!("Stolen tasks: {}", summary.stolen_count);

        print_worker_summary(&results, run_id);
        print_summary(&summary);

        all_results.extend(results);
        all_snapshots.extend(snapshots);
        all_summaries.push(summary);
    }

    write_queue_csv(&all_snapshots);
    write_task_csv(&all_results);
    write_summary_csv(&all_summaries);

    println!();
    println!("CSV files created:");
    println!("  queue_lengths.csv");
    println!("  task_results.csv");
    println!("  summary.csv");
}
