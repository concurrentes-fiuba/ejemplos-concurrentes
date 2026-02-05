extern crate rand;
extern crate std_semaphore;

use std::sync::{Arc, Barrier, Mutex, RwLock};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use std_semaphore::Semaphore;
use rand::{thread_rng, Rng};
use std::sync::atomic::{AtomicI32, Ordering};

fn main() {
    const N: usize = 5;

    let customer_waiting = Arc::new(Semaphore::new(0));
    let barber_ready = Arc::new(Semaphore::new(0));
    let haircut_done = Arc::new(Semaphore::new(0));
    let current_customer = Arc::new(Mutex::new(-1));
    let current_customer_set = Arc::new(Barrier::new(2));

    let customer_waiting_barber = customer_waiting.clone();
    let barber_ready_barber = barber_ready.clone();
    let haircut_done_barber = haircut_done.clone();
    let current_customer_barber = current_customer.clone();
    let current_customer_set_barber = current_customer_set.clone();
    let barber = thread::spawn(move || loop {
        println!("[Barbero] Esperando cliente");
        customer_waiting_barber.acquire();


        barber_ready_barber.release();

        current_customer_set_barber.wait();
        println!("[Barbero] Cortando pelo a {}", current_customer_barber.lock().unwrap());





        thread::sleep(Duration::from_secs(2));

        haircut_done_barber.release();
        println!("[Barbero] Terminé");
    });

    let customers: Vec<JoinHandle<()>> = (0..(N+1))
        .map(|_| {
            let barber_ready_customer = barber_ready.clone();
            let customer_waiting_customer = customer_waiting.clone();
            let haircut_done_customer = haircut_done.clone();
            let current_customer_id_customer = current_customer.clone();
            let current_customer_set_customer = current_customer_set.clone();
            thread::spawn(move || loop {
                thread::sleep(Duration::from_secs(thread_rng().gen_range(2, 10)));

                let me = rand::thread_rng().gen();

                println!("[Cliente {}] Entro a la barberia", me);
                customer_waiting_customer.release();

                println!("[Cliente {}] Esperando barbero", me);
                barber_ready_customer.acquire();

                println!("[Cliente {}] Me siento en la silla del barbero", me);
                *current_customer_id_customer.lock().unwrap() = me;

                current_customer_set_customer.wait();

                println!("[Cliente {}] Esperando a que me termine de cortar", me);
                haircut_done_customer.acquire();

                println!("[Cliente {}] Me terminaron de cortar", me);
            })
        })
        .collect();

    let _:Vec<()> = customers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    barber.join().unwrap();
}