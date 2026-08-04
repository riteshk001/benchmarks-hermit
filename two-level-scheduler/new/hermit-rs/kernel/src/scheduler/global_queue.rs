//Impementation of globale queue with probabilistic selection

use core::sync::atomic::AtomicU32;
use super::*;



pub const MAX_GLOBAL_QUEUES: usize = 32; //need to think exactly how many ?
// atomic counter
static NUM_GLOBAL_QUEUES: AtomicU32 = AtomicU32::new(0);

static GLOBAL_QUEUES: [InterruptTicketMutex<VecDeque<NewTask>>; MAX_GLOBAL_QUEUES] =
    [const { InterruptTicketMutex::new(VecDeque::new()) }; MAX_GLOBAL_QUEUES];

static GLOBAL_QUEUE_LENGTHS: [AtomicU32; MAX_GLOBAL_QUEUES] =
    [const { AtomicU32::new(0) }; MAX_GLOBAL_QUEUES];
// global queue takes directly the tasks not the handles




static GLOBAL_TOTAL_TASKS: AtomicU32 = AtomicU32::new(0);
pub fn init_global_queues(detected_core_count: usize) {
    // Ratio : 1 global queue  for 2 cores
    let ratio = detected_core_count / 2;

    let n = ratio.max(1).min(MAX_GLOBAL_QUEUES); // at least 1 global queue and don't go above MAX
    NUM_GLOBAL_QUEUES.store(n as u32, Ordering::Relaxed);
      for i in 0..n {
        GLOBAL_QUEUE_LENGTHS[i].store(0, Ordering::Relaxed);
        GLOBAL_QUEUES[i].lock().clear();
    }
    GLOBAL_TOTAL_TASKS.store(0, Ordering::Relaxed);
    
    info!("Initialized {} global queues for {} cores", n, detected_core_count);
}

// ====
// Opex on queues 
//=====

pub fn global_queue_push(queue_index: usize, task: NewTask) {
    let num_queues = NUM_GLOBAL_QUEUES.load(Ordering::Relaxed) as usize;
    assert!(queue_index < num_queues);
    
    GLOBAL_QUEUES[queue_index].lock().push_back(task);
    GLOBAL_QUEUE_LENGTHS[queue_index].fetch_add(1, Ordering::Relaxed);
    GLOBAL_TOTAL_TASKS.fetch_add(1, Ordering::Relaxed);
}

pub fn global_queue_pop(queue_index: usize) -> Option<NewTask> {
    let num_queues = NUM_GLOBAL_QUEUES.load(Ordering::Relaxed) as usize;
    assert!(queue_index < num_queues); // check if push in a valid queue index

    let mut queue = GLOBAL_QUEUES[queue_index].lock();
    let task = queue.pop_front();
    
    if task.is_some() { // if actually poped , then decrement
        GLOBAL_QUEUE_LENGTHS[queue_index].fetch_sub(1, Ordering::Relaxed);
        GLOBAL_TOTAL_TASKS.fetch_sub(1, Ordering::Relaxed);
    }
    
    task
}

pub fn get_queue_load(queue_index: usize) -> u32 {
    GLOBAL_QUEUE_LENGTHS[queue_index].load(Ordering::Relaxed)
}

pub fn select_queue_for_enqueue() -> usize {
    // select the queue with the minimum load
    // other idea if the overhead is too high, select min of two random queues ?
    let num_queues = NUM_GLOBAL_QUEUES.load(Ordering::Relaxed) as usize;
    let mut min_load = u32::MAX;
    let mut selected = 0;
    
    for i in 0..num_queues {
        let load = GLOBAL_QUEUE_LENGTHS[i].load(Ordering::Relaxed);
        if load < min_load {
            min_load = load;
            selected = i;
        }
    }
    
    selected
}


pub fn get_total_load() -> u32 {
    GLOBAL_TOTAL_TASKS.load(Ordering::Relaxed)
}

pub fn select_queue_for_dequeue(rng: &mut XorShiftRng) -> usize {
    // select a queue based on probabilistic selection, proportional to the load of each queue
    let num_queues = NUM_GLOBAL_QUEUES.load(Ordering::Relaxed) as usize;
    let total = get_total_load();
    
    if total == 0 {
        //if no tasks , give a random one, later will go to run fct
       return rng.gen_range(num_queues as u32) as usize;
    }
    
    //Probabilistic selection : P = load/total
    let random = rng.gen_range(total);
    let mut cumulative = 0;
    
    for i in 0..num_queues {
        cumulative += GLOBAL_QUEUE_LENGTHS[i].load(Ordering::Relaxed);
        if random < cumulative {
            return i;
        }
    }
    num_queues - 1
}
// asking for a task
pub fn refill_from_global_queue(scheduler: &mut PerCoreScheduler) {
    let total = get_total_load();
    if total == 0 {
        return;
    }

    let queue_index = select_queue_for_dequeue(&mut scheduler.rng);

    if let Some(mut new_task) = global_queue_pop(queue_index) {
        let task_id = new_task.tid;

        debug!(
            "Core {} took task {} from global queue {}",
            scheduler.core_id, task_id, queue_index
        );

        
        #[cfg(feature = "smp")]
        {
            new_task.core_id = scheduler.core_id;
        }

        let rc_task = Rc::new(RefCell::new(Task::from(new_task)));


        let mut tasks = TASKS.lock();
        if tasks.contains_key(&task_id) {
            #[cfg(feature = "smp")]
            {
                tasks.insert(
                    task_id,
                    TaskHandle::new(task_id, rc_task.borrow().prio, scheduler.core_id),
                );
            }
        }
    

        scheduler.ready_queue.push(rc_task);
    }
}
