pub const WORKERS: usize = 8;
pub const TASKS_COUNT: usize = 250;
pub const STEAL_THRESHOLD: usize = 25;
pub const STEAL_LOWER_BOUNDARY: usize = 5;

#[derive(Debug, Clone, Copy)]
pub enum Scenario {
    Regular,
    Burst,
    Mixed,
    Random,
}

impl Scenario {
    pub fn from_arg(arg: Option<String>) -> Self {
        match arg.as_deref() {
            Some("regular") => Scenario::Regular,
            Some("burst") => Scenario::Burst,
            Some("mixed") => Scenario::Mixed,
            Some("random") => Scenario::Random,
            _ => Scenario::Burst,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Scenario::Regular => "regular",
            Scenario::Burst => "burst",
            Scenario::Mixed => "mixed",
            Scenario::Random => "random",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: usize,
    pub work_units: u64,
    pub blocking_time_us: u64,
    pub created_time_us: u128,
    pub stolen_from: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    pub run_id: usize,
    pub worker_id: usize,
    pub task_id: usize,
    pub task_kind: &'static str,
    pub work_units: u64,
    pub blocking_time_configured_us: u64,
    pub stolen: bool,
    pub stolen_from: Option<usize>,

    pub created_time_us: u128,
    pub start_time_us: u128,
    pub end_time_us: u128,

    pub waiting_time_us: u128,
    pub blocking_time_real_us: u128,
    pub cpu_work_time_us: u128,
    pub execution_time_us: u128,
    pub response_time_us: u128,
}

#[derive(Debug, Clone)]
pub struct QueueSnapshot {
    pub run_id: usize,
    pub time_us: u128,
    pub queue_lengths: Vec<usize>,
}

#[derive(Debug, Clone, Copy)]
pub struct TimeStats {
    pub min: u128,
    pub max: u128,
    pub mean: u128,
}

#[derive(Debug, Clone)]
pub struct RunSummary {
    pub run_id: usize,
    pub scenario: Scenario,
    pub total_time_us: u128,
    pub stolen_count: usize,
    pub long_tasks: usize,
    pub short_tasks: usize,
    pub waiting: TimeStats,
    pub blocking: TimeStats,
    pub cpu_work: TimeStats,
    pub execution: TimeStats,
    pub response: TimeStats,
}
