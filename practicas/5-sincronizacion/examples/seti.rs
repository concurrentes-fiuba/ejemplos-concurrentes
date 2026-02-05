extern crate rand;

use std::sync::{Arc, RwLock, Barrier};
use std::thread;
use std::time::Duration;
use rand::Rng;

const WORKERS: u32 = 10;

fn main() {
    let signal: f64 = 100.0;
    let lock = Arc::new(RwLock::new(signal));
    let barrier = Arc::new(Barrier::new(WORKERS as usize));
    let barrier2 = Arc::new(Barrier::new(WORKERS as usize));

    let mut workers = vec![];

    for id in 0..WORKERS {
        let lock_clone = lock.clone();
        let barrier_clone = barrier.clone();
        let barrier2_clone = barrier2.clone();
        workers.push(thread::spawn(move || worker(id, lock_clone, barrier_clone, barrier2_clone)));
    }

    for worker in workers {
        worker.join().unwrap();
    }

}

fn worker(id: u32, lock: Arc<RwLock<f64>>, barrier: Arc<Barrier>, barrier2: Arc<Barrier>) {
    let mut epoch = 0;

    loop {
        // Espero a que todos inicien la vuelta
        barrier.wait();

        let signal = *lock.read().unwrap() / WORKERS as f64;
        println!("[WORKER {}] inicio epoch {} signal {}", id, epoch, signal);
        // Espero a que todos hayan leido el saldo disponible
        barrier2.wait();

        // Tomo el dinero
        if let Ok(mut money_guard) = lock.write() {
            *money_guard -= signal;
        }

        epoch += 1;
        let mut rng = rand::thread_rng();
        let random_result: f64 = rng.gen_range(0.0, 1.0);
        thread::sleep(Duration::from_millis((2000 as f64 * random_result) as u64));
        let result = signal * random_result;
        println!("[WORKER {}] voy a retornar {}", id, result);

        if let Ok(mut guard) = lock.write() {
            *guard += result;
        }

        println!("[WORKER] {} retorné {}", id, result);
    }
}
