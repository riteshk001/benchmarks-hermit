#![allow(unused)]
use nix::unistd::{sysconf, SysconfVar};
// use procfs::process::Process;
use procfs::{CurrentSI, KernelStats};
use std::{collections::HashMap, ops::{Add, Mul, Sub}, sync::atomic::{AtomicU64, Ordering}, thread, time::{Duration, Instant}};
use fixed::{FixedU32, traits::Fixed, types::{I9F23, extra::U23}};

struct TaskId(i32);
// Magic constants (to be later replaced by fixed point numbers)
const EXP_1: f64 = 0.920044415;
const EXP_5: f64 = 0.983471454;
const EXP_15: f64 = 0.994459811;

// Variables for hermit kernel. To be tested later 
enum TaskStatus {
    Invalid,
    Ready,
    Running,
    Finished,
    Idle,
}

struct Priority(u8);
type CoreId = u32;

struct Task {
    id: TaskId, 
    status: TaskStatus,
    prio: Priority,
    core_id: CoreId,
    // stacks: TaskStacks,
}

#[derive(Default, Clone)]
struct LoadAvg {
    avg_1_fixed: FixedU32<U23>,
    avg_5_fixed: FixedU32<U23>,
    avg_15_fixed: FixedU32<U23>,
}

impl LoadAvg {
    fn new() -> Self {
        Self { avg_1_fixed: FixedU32::ZERO, avg_5_fixed: FixedU32::ZERO, avg_15_fixed: FixedU32::ZERO}
    }

    fn update_values(&mut self, active_tasks: f64, exp_1_fixed: &FixedU32<U23>, exp_5_fixed: &FixedU32<U23>, exp_15_fixed: &FixedU32<U23>) {
        // TODO: Write fixed point logic to update load values for every cycle. 
        self.avg_1_fixed = FixedU32::mul(self.avg_1_fixed, exp_1_fixed)
                                    .add((FixedU32::from_num(1.0).sub(exp_1_fixed))
                                    .mul(FixedU32::from_num(active_tasks)));
        self.avg_5_fixed = FixedU32::mul(self.avg_5_fixed, exp_5_fixed)
                                    .add((FixedU32::from_num(1.0).sub(exp_5_fixed))
                                    .mul(FixedU32::from_num(active_tasks)));
        self.avg_15_fixed = FixedU32::mul(self.avg_15_fixed, exp_15_fixed)
                                    .add((FixedU32::from_num(1.0).sub(exp_15_fixed))
                                    .mul(FixedU32::from_num(active_tasks)));
    }
}

static COUNT: AtomicU64 = AtomicU64::new(0);

fn increment() {
    COUNT.fetch_add(1, Ordering::Relaxed);
}

fn get_count() -> u64 {
    COUNT.load(Ordering::Relaxed)
}

fn return_no_cores() -> i64 {
    sysconf(SysconfVar::_NPROCESSORS_ONLN).unwrap().unwrap()
}

// fn list_tasks_with_cpu() -> procfs::ProcResult<()> {
//     for proc in procfs::process::all_processes()? {
//         let stat = proc.unwrap().stat()?;
//         if stat.state == 'R' || stat.state == 'D'{
//             println!("PID {} CPU {} CMD {}", stat.pid, stat.processor.unwrap_or(-1), stat.comm);
//             increment();
//         }
//     }
//     Ok(())
// }

fn active_tasks_per_cpu() -> procfs::ProcResult<HashMap<i32,u64>> {
    // let mut counts_per_cpu: HashMap<i32, u64> = HashMap::new();
    let mut counts_per_cpu: HashMap<i32, u64> = (0..return_no_cores() as i32).map(|c| (c, 0)).collect();

    for proc in procfs::process::all_processes()? {
        let Ok(proc) = proc else {continue};
        let tasks = proc.tasks()?;
        for task in tasks {
            let Ok(task) = task else {continue};
            let Ok(task_stat) = task.stat() else { continue };
            if matches!(task_stat.state, 'R' | 'D') {
                if let Some(cpu) = task_stat.processor {
                    *counts_per_cpu.entry(cpu).or_insert(0) += 1;
                }
            }

        }
    }
    Ok(counts_per_cpu)
}

fn main() {
    // println!("Hello, world!");

    let mut load_avg = LoadAvg::new();

    let mut exp_1_fixed: FixedU32<U23> = FixedU32::from_num(EXP_1);
    let mut exp_5_fixed: FixedU32<U23> = FixedU32::from_num(EXP_5);
    let mut exp_15_fixed: FixedU32<U23> = FixedU32::from_num(EXP_15);
    // let mut per_cpu_load: Vec<LoadAvg> = vec![LoadAvg::default(); return_no_cores() as usize];
    let mut per_cpu_load: Vec<LoadAvg> = vec![LoadAvg::new(); return_no_cores() as usize];

    loop {
        // thread::sleep(Duration::from_secs(5));
        let no_cores = sysconf(SysconfVar::_NPROCESSORS_ONLN).unwrap().unwrap();
        println!("No of cores {}", return_no_cores());

        let kstats  = procfs::KernelStats::current().unwrap();
        let active_procs = (kstats.procs_running.unwrap_or(0) + kstats.procs_blocked.unwrap_or(0)) as f64;

        // let mut per_cpu_load: Vec<LoadAvg> = vec![LoadAvg::default(); return_no_cores() as usize];
        let mut next_tick = Instant::now();

        let count_tasks_per_cpu = active_tasks_per_cpu().unwrap();

        for cpu in 0..return_no_cores() {
            // println!("CPU: {}", cpu);
            let active_tasks = count_tasks_per_cpu.get(&(cpu as i32)).copied().unwrap_or(0) as f64;
            per_cpu_load[cpu as usize].update_values(active_tasks, &exp_1_fixed, &exp_5_fixed, &exp_15_fixed);

        }

        for (cpu, load) in per_cpu_load.iter().enumerate() {
            println!("load avg for {}: {:.2} {:.2} {:.2}", cpu, load.avg_1_fixed.to_num::<f32>(),load.avg_5_fixed.to_num::<f32>(),load.avg_15_fixed.to_num::<f32>());
        }
        next_tick += std::time::Duration::from_secs(5);
        let now = Instant::now();
        if next_tick > now {
            std::thread::sleep(next_tick - now);
        }
    }

}
