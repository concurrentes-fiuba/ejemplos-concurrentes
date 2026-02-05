extern crate rand;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use rand::{Rng, thread_rng};

const WORKERS: u32 = 5;

fn main() {
    let signal = Arc::new(RwLock::new(100.0));

    let workers: Vec<JoinHandle<()>> = (0..WORKERS)
        .map(|id| {
            let signal_local = signal.clone();
            thread::spawn(move || worker(id, signal_local))
        })
        .collect();

    workers.into_iter()
        .flat_map(|x| x.join())
        .for_each(drop);

}

fn worker(id:u32,  signal_source:Arc<RwLock<f64>>) {
    loop {
        let signal = *signal_source.read().unwrap() / (WORKERS as f64);
        println!("[WORKER {}] inicio con señal {}", id, signal);
        let rand = thread_rng().gen_range(0.0, 1.0);
        thread::sleep(Duration::from_millis((2000.0 * rand) as u64));
        let result = signal * rand;
        if let Ok(mut guard) = signal_source.write() {
            *guard += result - signal;
        }
        println!("[WORKER {}] resultado {}", id, result);
    }

}
