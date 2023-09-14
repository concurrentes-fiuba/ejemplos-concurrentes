extern crate rand;

use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;

const INVERSORES: i32 = 5;
const SALDO_INICIAL: f64 = 100000.0;

fn main() {

    let mut saldo = SALDO_INICIAL;
    let mut semana = 1;

    loop {
        println!("[BANQUERO] semana {}, tengo saldo {}", semana, saldo);
        let inversores: Vec<JoinHandle<f64>> = (0..INVERSORES)
            .map(|id| thread::spawn(move || inversor(id, saldo / (INVERSORES as f64))))
            .collect();

        saldo = inversores.into_iter()
            .flat_map(|x| x.join())
            .sum();

        semana += 1
    }
}

fn inversor(id: i32, prestamo: f64) -> f64 {
    println!("[INVERSOR {}] tengo prestamo {}", id, prestamo);

    let random_result: f64 = rand::thread_rng().gen();
    thread::sleep(Duration::from_millis((random_result * 1000.0) as u64));

    let result = prestamo * (random_result + 0.5);

    println!("[INVESOR {}] devuelvo {}", id, result);
    result
}
