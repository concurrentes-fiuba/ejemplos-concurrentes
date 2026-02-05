extern crate rand;

use std::time::Duration;

use std::thread;
use std::thread::JoinHandle;

use rand::{Rng, thread_rng};

const WORKERS: i32 = 5;

fn main() {

    let mut signal = 100.0;
    let mut epoch = 1;

    loop {
        println!("[COORDINADOR] epoch {}, señal {}", epoch, signal);
        let mut workers = vec![];
        let signal_worker = signal / (WORKERS as f64);
        signal = 0.0;

        for id in 0..WORKERS {
            workers.push(thread::spawn(move || worker(id, signal_worker)))
        }

        // Para pensar: Por qué dos fors?
        for worker in workers {
            match worker.join() {
                Ok(processed) => signal += processed,
                Err(str) => panic!("{:?}", str)
            }
        }

        epoch += 1
    }
}

fn worker(id: i32, signal: f64) -> f64 {
    println!("[WORKER {}] recibo {}", id, signal);

    // Para probar panic en thread
    // if (signal < 1) {
    //  panic!("señal muy debil, no se puede analizar");
    // }

    let random_result: f64 = rand::thread_rng().gen_range(0.0, 1.0);
    thread::sleep(Duration::from_millis((random_result * 1000.0) as u64));

    let result = signal * random_result;

    println!("[WORKER {}] devuelvo {}", id, result);
    result
}
