extern crate rand;

use std::time::Duration;

use std::thread;
use std::thread::JoinHandle;

use rand::{Rng, thread_rng};

const INVERSORES: i32 = 5;
const SALDO_INICIAL: f64 = 100000.0;

fn main() {

    let mut saldo = SALDO_INICIAL;
    let mut semana = 1;

    loop {
        println!("[BANQUERO] semana {}, tengo saldo {}", semana, saldo);
        let mut inversores = vec![];
        let saldo_individual = saldo / (INVERSORES as f64);
        saldo = 0.0;

        for id in 0..INVERSORES {
            inversores.push(thread::spawn(move || inversor(id, saldo_individual)))
        }

        // Para pensar: Por qué dos fors?
        for inversor in inversores {
            match inversor.join() {
                Ok(plata) => saldo += plata,
                Err(str) => panic!(str)
            }
        }

        semana += 1
    }
}

fn inversor(id: i32, prestamo: f64) -> f64 {
    println!("[INVERSOR {}] tengo prestamo {}", id, prestamo);

    // Para probar panic en thread
    // if (prestamo < 100) {
    //  panic!("no alcanza para invertir");
    // }

    let random_result: f64 = rand::thread_rng().gen();
    thread::sleep(Duration::from_millis((random_result * 1000.0) as u64));

    let result = prestamo * (random_result + 0.5);

    println!("[INVESOR {}] devuelvo {}", id, result);
    result
}
