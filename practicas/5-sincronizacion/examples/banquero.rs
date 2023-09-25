extern crate rand;

use std::sync::{Arc, RwLock, Barrier};
use std::thread;
use std::time::Duration;
use rand::Rng;

const FRIENDS: u32 = 10;

/**
Al tiempo el señor banquero fallece.
Los hijos deciden que los inversores sigan trabajando el
dinero pero ellos no se hacen cargo de nada. Los inversores solos deberán tomar el
dinero de la cuenta al inicio de la semana y devolverlo al final.
*/

fn main() {
    let money: f64 = 1000.0;
    let lock = Arc::new(RwLock::new(money));
    let barrier = Arc::new(Barrier::new(FRIENDS as usize));
    let barrier2 = Arc::new(Barrier::new(FRIENDS as usize));

    let mut friends = vec![];

    for id in 0..FRIENDS {
        let lock_clone = lock.clone();
        let barrier_clone = barrier.clone();
        let barrier2_clone = barrier2.clone();
        friends.push(thread::spawn(move || inversor(id, lock_clone, barrier_clone, barrier2_clone)));
    }

    for friend in friends {
        friend.join().unwrap();
    }

}

fn inversor(id: u32, lock: Arc<RwLock<f64>>, barrier: Arc<Barrier>, barrier2: Arc<Barrier>) {
    let mut semana = 0;

    while *lock.read().unwrap() > 1.0 {
        // Espero a que todos inicien la semana
        barrier.wait();

        let prestamo = *lock.read().unwrap() / FRIENDS as f64;
        println!("inversor {} inicio semana {} plata {}", id, semana, prestamo);
        // Espero a que todos hayan leido el saldo disponible
        barrier2.wait();

        // Tomo el dinero
        if let Ok(mut money_guard) = lock.write() {
            *money_guard -= prestamo;
        }

        semana += 1;
        let mut rng = rand::thread_rng();
        let random_result: f64 = rng.gen();
        thread::sleep(Duration::from_millis((2000 as f64 * random_result) as u64));
        let earn = prestamo * (random_result + 0.5);
        println!("inversor {} voy a devolver {}", id, earn);

        if let Ok(mut money_guard) = lock.write() {
            *money_guard += earn;
        }

        println!("inversor {} devolví {}", id, earn);
    }
}
