use std::time::Duration;

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Arc, RwLock};

const INVERSORES:i64 = 10;

fn main() {

    let cuenta = Arc::new(RwLock::new(10000.0));

    loop {
        let mut inversores: Vec<JoinHandle<()>> = vec!();
        for id in 1..INVERSORES {
            let cuenta_local = cuenta.clone();
            inversores.push(thread::spawn(move || inversor(id, cuenta_local)));
        }

        for inversor in inversores {
            inversor.join().unwrap();
        }
    }

}


fn inversor(id: i64, cuenta:Arc<RwLock<f64>>) {

    if id == 5 {
        if let Ok(mut saldo) = cuenta.write() {
            println!("thread {} adentro write", id);
            thread::yield_now();
            println!("thread {} por salir de write", id);
        }

    } else {
        if let Ok(saldo) = cuenta.read() {
            println!("thread {} adentro read", id);
            thread::yield_now();
            println!("thread {} por salir de read", id);

        }

    }

    thread::sleep(Duration::from_millis(1000));

}