use crate::utils::{task_work, analyze_metrics};
mod utils;

use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use std::time::{Duration, SystemTime, UNIX_EPOCH, Instant};
use std::collections::HashMap;
use serde::{Serialize, Deserialize};

#[cfg(target_os = "hermit")]
use hermit as _;

#[derive(Debug, Serialize, Deserialize)]
struct BenchmarkOutput {
    scenario: String,
    nb_tasks: usize,
    length: i64,
    length_short: Option<i64>,
    length_long: Option<i64>,
    io_time_ms: u64,
    io_time_short_ms: u64,
    cores: usize,
    total_time_ms: u128,
    waiting_time: Stats,      
    execution_time: Stats,   
    response_time: Stats,     
    run: Option<u32>,        
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
struct Stats {
    min: u64,
    max: u64,
    mean: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProcessMetrics {
    tag: u8,
    index: usize,
    creation_time: u64,
    start_work_time: u64,
    end_work_time: u64,
    waiting_time: u64,
    execution_time: u64,
    response_time: u64,
}


const CORES: usize = 16; 
fn main() {
    let io_time = 600;
    let nb_tasks_list = [50, 100, 250, 300,400];
    let lengths = [64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536, 2048];

    for run in 1..=5 {
        for &nb_tasks in &nb_tasks_list {
            for &length in &lengths {
                // Benchmark "mix"
                let output_mix = run_benchmark(nb_tasks, length, io_time, "mix", run);
                let json_mix = serde_json::to_string(&output_mix).unwrap();
                println!("{}", json_mix);

                std::thread::sleep(Duration::from_secs(3));
            }
        }
    }
}

fn run_benchmark(nb_tasks: usize, length: i64, io_time: u64, mode: &str, run: u32) -> BenchmarkOutput {
            let (total_time, waiting, execution, response) = scenario_mix(
                nb_tasks,
                length / 2,
                length,
                Duration::from_millis(io_time),
                Duration::from_millis(0),
            );

            BenchmarkOutput {
                scenario: "mix".to_string(),
                nb_tasks,
                length: 0,
                length_short: Some(length / 2),
                length_long: Some(length),
                io_time_ms: io_time,
                io_time_short_ms: 0,
                cores: CORES,
                total_time_ms: total_time.as_millis(), 
                waiting_time: waiting,
                execution_time: execution,
                response_time: response,
                run: Some(run),
            }
}


fn scenario_mix(
    nb_task: usize,
    length_short: i64,
    length_long: i64,
    blocking_time_l: Duration,
    blocking_time_s: Duration,
) -> (Duration, Stats, Stats, Stats) {
    let mut tids = Vec::new();
    let metrics_storage = Arc::new(Mutex::new(Vec::<ProcessMetrics>::new()));
    let start = Instant::now();

    for i in 0..nb_task {
        let creation_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        let storage_clone = Arc::clone(&metrics_storage);
        let tid = thread::spawn(move || {
            let start_work_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            if i % 2 == 0 {
                task_work(length_short, blocking_time_s);
            } else {
                task_work(length_long, blocking_time_l);
            }

            let end_work_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64;

            let waiting_time = start_work_time - creation_time;
            let execution_time = end_work_time - start_work_time;
            let response_time = end_work_time - creation_time;

            let metrics = ProcessMetrics {
                tag: if i % 2 == 0 { 0 } else { 1 },
                index: i,
                creation_time,
                start_work_time,
                end_work_time,
                waiting_time,
                execution_time,
                response_time,
            };
            storage_clone.lock().unwrap().push(metrics);
        });
        tids.push(tid);
    }

    for tid in tids {
        tid.join().unwrap();
    }

    let total_time = start.elapsed();

    let all_metrics = metrics_storage.lock().unwrap();
    let mut timing_metric = HashMap::new();
    for metrics in all_metrics.iter() {
        timing_metric.insert(metrics.index, metrics.clone());
    }

    let (waiting, execution, response) = analyze_metrics(timing_metric);

    (total_time, waiting, execution, response)
}
