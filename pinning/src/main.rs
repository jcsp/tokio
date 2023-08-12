use tokio::{self, task::yield_now};
mod core_tracker;
use core_tracker::CoreTracker;
use log::*;
use std::{sync::Mutex, time::Instant};

fn main() {
    env_logger::builder().format_timestamp_micros().init();
    let cpu_cores = core_affinity::get_core_ids().unwrap();
    // We need a counter to index the round-robin assignment of home shards to threads,
    // because core Ids may not be contiguous or start at zero.
    let thread_counter = Mutex::new(0 as usize);

    let workers = 4;

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(workers)
        .on_thread_start({
            debug!("on_thread_start");
            move || {
                let mut thread_counter = thread_counter.lock().unwrap();

                // For first N workers on an N core environment, bind them to a CPU core
                // (even if worker count is lower, tokio may spawn more)
                if *thread_counter < cpu_cores.len() {
                    let my_cpu = cpu_cores[*thread_counter];
                    core_affinity::set_for_current(my_cpu);
                    CoreTracker::register_thread(my_cpu.id);
                }

                *thread_counter += 1;
            }
        })
        .build()
        .unwrap();

    runtime.block_on(async { tokio::spawn(round_robin(workers)).await });
}

async fn jump_to_core(core: usize) {
    tokio::task::set_worker_pin(core as u8);
    yield_now().await;
}

async fn round_robin(workers: usize) {
    info!("I can run on any core I want!");
    info!("Starting on {}", CoreTracker::get_cpu_id());

    let warmup_iterations = workers * 100000;
    for n in 0..warmup_iterations {
        let core = n % workers;
        jump_to_core(core).await;
    }

    let iterations = workers * 1000000;

    let initial_t = Instant::now();
    for n in 0..iterations {
        let core = n % workers;
        jump_to_core(core).await;
    }
    let duration = initial_t.elapsed();
    println!("jumps: {} iterations in {}ms, {}us per jump",
        iterations, duration.as_millis(), (duration / iterations as u32).as_micros());
}

