extern crate rand;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const WORKERS: i32 = 10;

fn main() {
    let mut signal = 100.0;

    let (result_send, result_receive) = mpsc::channel();

    let workers: Vec<(Sender<f64>, JoinHandle<()>)> = (0..WORKERS)
        .map(|id| {
            let (worker_send, worker_receive) = mpsc::channel();
            let result_send_worker = result_send.clone();
            let t = thread::spawn(move || worker(id, worker_receive, result_send_worker));
            (worker_send, t)
        })
        .collect();


    loop {
        let mut signal_epoch = start_epoch(&mut signal, &workers);

        let mut results = HashSet::new();

        while(results.len() < (WORKERS as usize)) {
            let (who, result) = result_receive.recv().unwrap();
            println!("[COORDINADOR] recibí de {} señal {}", who, result);
            if !results.contains(&who) {
                results.insert(who);
                signal_epoch += result;
            }
        }

        println!("[COORDINADOR] señal final {}", signal_epoch);
        signal = signal_epoch
    }

    let _:Vec<()> = workers.into_iter()
        .flat_map(|(_,h)| h.join())
        .collect();
}

fn start_epoch(signal: &mut f64, workers: &Vec<(Sender<f64>, JoinHandle<()>)>) -> f64 {
    let signal_worker = *signal / (WORKERS as f64);
    for (worker, _) in workers {
        worker.send(signal_worker).unwrap();
    }

    let mut signal_epoch = 0.0;
    signal_epoch
}

fn worker(id: i32, signal_source: Receiver<f64>, result: Sender<(i32, f64)>) {
    loop {
        let signal = signal_source.recv().unwrap();
        println!("[WORKER {}] señal {}", id, signal);
        thread::sleep(Duration::from_secs(2));
        let resultado = signal * thread_rng().gen_range(0., 1.);
        println!("[WORKER {}] resultado {}", id, resultado);
        result.send((id, resultado));
    }
}