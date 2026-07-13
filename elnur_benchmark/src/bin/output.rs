use std::fs;

use crate::types10::{
    QueueSnapshot, RunSummary, TaskResult, STEAL_LOWER_BOUNDARY, STEAL_THRESHOLD, TASKS_COUNT,
    WORKERS,
};

pub fn write_queue_csv(snapshots: &[QueueSnapshot]) {
    let mut csv = String::new();

    csv.push_str("run_id,time_us");

    for i in 0..WORKERS {
        csv.push_str(&format!(",worker_{}", i));
    }

    csv.push('\n');

    for snapshot in snapshots {
        csv.push_str(&format!("{},{}", snapshot.run_id, snapshot.time_us));

        for len in &snapshot.queue_lengths {
            csv.push_str(&format!(",{}", len));
        }

        csv.push('\n');
    }

    fs::write("queue_lengths.csv", csv).unwrap();
}

pub fn write_task_csv(results: &[TaskResult]) {
    let mut csv = String::new();

    csv.push_str(
        "run_id,worker_id,task_id,task_kind,work_units,blocking_time_configured_us,stolen,stolen_from,\
         created_time_us,start_time_us,end_time_us,\
         waiting_time_us,blocking_time_real_us,cpu_work_time_us,\
         execution_time_us,response_time_us\n",
    );

    for r in results {
        let stolen_from = match r.stolen_from {
            Some(id) => id.to_string(),
            None => "None".to_string(),
        };

        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            r.run_id,
            r.worker_id,
            r.task_id,
            r.task_kind,
            r.work_units,
            r.blocking_time_configured_us,
            r.stolen,
            stolen_from,
            r.created_time_us,
            r.start_time_us,
            r.end_time_us,
            r.waiting_time_us,
            r.blocking_time_real_us,
            r.cpu_work_time_us,
            r.execution_time_us,
            r.response_time_us,
        ));
    }

    fs::write("task_results.csv", csv).unwrap();
}

pub fn write_summary_csv(summaries: &[RunSummary]) {
    let mut csv = String::new();

    csv.push_str(
        "run_id,scenario,workers,tasks_count,long_tasks,short_tasks,steal_threshold,steal_lower_boundary,\
         total_runtime_us,stolen_tasks,\
         waiting_min_us,waiting_max_us,waiting_mean_us,\
         blocking_min_us,blocking_max_us,blocking_mean_us,\
         cpu_work_min_us,cpu_work_max_us,cpu_work_mean_us,\
         execution_min_us,execution_max_us,execution_mean_us,\
         response_min_us,response_max_us,response_mean_us\n",
    );

    for s in summaries {
        csv.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}\n",
            s.run_id,
            s.scenario.as_str(),
            WORKERS,
            TASKS_COUNT,
            s.long_tasks,
            s.short_tasks,
            STEAL_THRESHOLD,
            STEAL_LOWER_BOUNDARY,
            s.total_time_us,
            s.stolen_count,
            s.waiting.min, s.waiting.max, s.waiting.mean,
            s.blocking.min, s.blocking.max, s.blocking.mean,
            s.cpu_work.min, s.cpu_work.max, s.cpu_work.mean,
            s.execution.min, s.execution.max, s.execution.mean,
            s.response.min, s.response.max, s.response.mean,
        ));
    }

    fs::write("summary.csv", csv).unwrap();
}

pub fn print_worker_summary(results: &[TaskResult], run_id: usize) {
    println!();
    println!("Per-worker summary for run {}:", run_id);

    for worker_id in 0..WORKERS {
        let executed = results
            .iter()
            .filter(|r| r.worker_id == worker_id)
            .count();

        let stolen_executed = results
            .iter()
            .filter(|r| r.worker_id == worker_id && r.stolen)
            .count();

        println!(
            "  Worker {}: executed {}, stolen tasks executed {}",
            worker_id, executed, stolen_executed
        );
    }
}

pub fn print_summary(summary: &RunSummary) {
    println!();
    println!("Time statistics for run {} [us]:", summary.run_id);
    println!("  Total simulation runtime: {}", summary.total_time_us);
    println!(
        "  Queue waiting: min {}, max {}, mean {}",
        summary.waiting.min, summary.waiting.max, summary.waiting.mean
    );
    println!(
        "  Blocking:      min {}, max {}, mean {}",
        summary.blocking.min, summary.blocking.max, summary.blocking.mean
    );
    println!(
        "  CPU work:      min {}, max {}, mean {}",
        summary.cpu_work.min, summary.cpu_work.max, summary.cpu_work.mean
    );
    println!(
        "  Execution:     min {}, max {}, mean {}",
        summary.execution.min, summary.execution.max, summary.execution.mean
    );
    println!(
        "  Response:      min {}, max {}, mean {}",
        summary.response.min, summary.response.max, summary.response.mean
    );
}
