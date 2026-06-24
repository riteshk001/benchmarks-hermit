use std::time::Duration;
use std::thread::sleep;
use std::hint::black_box;
use crate::HashMap;
 use crate::{ProcessMetrics,Stats};
use rand::{Rng};
use std::os::fd::RawFd;

  use libc::{read, c_void};

pub fn read_exact(fd: RawFd, buf: &mut [u8]) {
    let mut total = 0;
    while total < buf.len() {
        let n = unsafe { read(fd, buf[total..].as_mut_ptr() as *mut c_void, buf.len() - total) };
        if n <= 0 {
            panic!("read error or EOF");
        }
        total += n as usize;
    }
}
pub fn task_work(n: i64, blocking_time: Duration) {
    let mut rng = rand::thread_rng();
    let offset1 = black_box(rng.gen_range(0..10)); // so it is differnt everytime and cannot be optimized
    let offset2 = black_box(rng.gen_range(0..10));
    
    matrix_work(n/2, offset1);
    sleep(blocking_time);
    matrix_work(n/2, offset2);
}

fn matrix_work(n: i64, offset: i64) {
    let size = black_box(n as usize); // to not optimised for loops
    let mut a = vec![vec![0_i64; size]; size];
    let mut b = vec![vec![0_i64; size]; size];
    let mut c = vec![vec![0_i64; size]; size];
    
    for i in 0..size {
        for j in 0..size {
            let idx = (i * size + j) as i64 + offset;  
            a[i][j] = idx;
            b[i][j] = idx * 2;
        }
    }
    
    for i in 0..size {
        for j in 0..size {
            for k in 0..size {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    
    black_box(c);
}

pub fn analyze_metrics(hashmap: HashMap<usize, ProcessMetrics>) -> (Stats, Stats, Stats) {
    let mut waiting_time_sum: u64 = 0;
    let mut waiting_time_min: u64 = u64::MAX;
    let mut waiting_time_max: u64 = 0;

    let mut execution_time_sum: u64 = 0;
    let mut execution_time_min: u64 = u64::MAX;
    let mut execution_time_max: u64 = 0;

    let mut response_time_sum: u64 = 0;
    let mut response_time_min: u64 = u64::MAX;
    let mut response_time_max: u64 = 0;

    let count = hashmap.len() as u64;

    for (_pid, metrics) in hashmap.iter() {
        let wait = metrics.waiting_time;
        let exec = metrics.execution_time;
        let resp = metrics.response_time;

        waiting_time_sum += wait;
        waiting_time_min = waiting_time_min.min(wait);
        waiting_time_max = waiting_time_max.max(wait);

        execution_time_sum += exec;
        execution_time_min = execution_time_min.min(exec);
        execution_time_max = execution_time_max.max(exec);

        response_time_sum += resp;
        response_time_min = response_time_min.min(resp);
        response_time_max = response_time_max.max(resp);
    }

    let waiting = Stats {
        min: waiting_time_min,
        max: waiting_time_max,
        mean: waiting_time_sum / count,
    };

    let execution = Stats {
        min: execution_time_min,
        max: execution_time_max,
        mean: execution_time_sum / count,
    };

    let response = Stats {
        min: response_time_min,
        max: response_time_max,
        mean: response_time_sum / count,
    };

    (waiting, execution, response)
}