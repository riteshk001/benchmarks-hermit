use std::time::Duration;
use fork::{fork, Fork};
use crate::utils::{task_work};
mod utils;



const CORE: usize = 8;


fn main() {
    let nb_tasks = CORE * 2; // or more ?

    let lenght_short:i64 = 256;
    let blocking_time_short = Duration::from_millis(0);

    let lenght_long:i64 = 512;
    let blocking_time_long: Duration = Duration::from_millis(600); 
    
    // -- Short tasks scenario -- //
    
    let (total_time,blocked_time) = scenario_task(nb_tasks,lenght_short,blocking_time_short);
    let pourc_waiting = ((blocked_time.as_secs_f64() / total_time.as_secs_f64()) * 100.0) as u32;

    println!("-- Running scheduler benchmark (SHORT tasks scenario) --");
    println!("LENGHT: {}    BLOCKING_T: {:?}      CORE: {}", lenght_short, blocking_time_short, CORE);
    println!("NB of tasks: {}",nb_tasks);
    println!("Total time : {:?} s", total_time);
    println!("Blocking Time due to scheduling : {:?}",blocked_time);
    println!("Porcentage of blocking time: {:2}%",pourc_waiting);

    //--Long tasks scenario -- //

    let (total_time_l,blocked_time_l) = scenario_task(nb_tasks,lenght_long,blocking_time_long);
    let pourc_waiting_l = ((blocked_time_l.as_secs_f64() / total_time_l.as_secs_f64()) * 100.0) as u32;

    println!("-- Running scheduler benchmark (LONG tasks scenario) --");
    println!("LENGHT: {}    BLOCKING_T: {:?}      CORE: {}", lenght_long, blocking_time_long,CORE);
    println!("NB of tasks: {}",nb_tasks);
    println!("Total time : {:?} s", total_time_l);
    println!("Blocking Time due to scheduling : {:?}",blocked_time_l);
    println!("Porcentage of blocking time: {:2}%",pourc_waiting_l);


}


fn scenario_task(nb_task: usize, lenght: i64, blocking_time: Duration) -> (Duration,Duration) {

    //we are going to mesure first how long a short task takes without any contention:
    let start_init = std::time::Instant::now();
    match fork() {
            Ok(Fork::Parent(child)) => {
                let _ = fork::waitpid(child);
            }
            Ok(Fork::Child) => {
                
                task_work(lenght, blocking_time);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Fork échoué: {}", e);
            }
    }
    // when the child has finished
    let task_alone_t = start_init.elapsed();

    // now we do the real test with multiple tasks
    let mut pids = Vec::new();
    let start = std::time::Instant::now();
    
    for _i in 0..nb_task {
        match fork() {
            Ok(Fork::Parent(child)) => {
                pids.push(child);
                
            }
            Ok(Fork::Child) => {
                
                task_work(lenght, blocking_time);
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("Fork échoué: {}", e);
            }
        }
    }
    
    
    for pid in pids {
        let _ = fork::waitpid(pid);
    }

    // analysing part
    let total_time = start.elapsed();
    let ideal_time = task_alone_t + blocking_time;
    let blocked_time = match total_time.checked_sub(ideal_time) { 
        Some(diff) => diff,
        None => {
            eprintln!("ERR : total_time < ideal_time");
            eprintln!("   total_time: {:.9} s", total_time.as_secs_f64());
            eprintln!("   ideal_time: {:.9} s", ideal_time.as_secs_f64());
            eprintln!("   Différence: {:.9} s", 
                      (ideal_time.as_secs_f64() - total_time.as_secs_f64()));
            Duration::ZERO
        }
    };
    

    (total_time,blocked_time)
}
