extern crate rand;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;

fn main() {

    const N:i32 = 5;

    let pair = Arc::new((Mutex::new(Vec::new()), Condvar::new()));

    let pair_clone = pair.clone();
    let producer = thread::spawn(move || {
        let mut produced:i32 = 0;
        loop {
            let (lock, cvar) = &*pair_clone;

            println!("[PRODUCER] doing expensive computation");
            // jugar con los waits
            thread::sleep(Duration::from_millis(1000));
            produced+=1;
            println!("[PRODUCER] done");
            let mut buffer = lock.lock().unwrap();
            println!("[PRODUCER] got lock");
            buffer.push(produced);
            println!("[PRODUCER] notifying");

            // play with notify_one
            cvar.notify_all();
        }
    });

    let consumers:Vec<JoinHandle<()>> = (1..N).map(|i| {
        let pair_clone_waiter = pair.clone();
        thread::spawn(move || {
            loop {
                let (lock, cvar) = &*pair_clone_waiter;

                let to_consume = {
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |buffer| {
                        println!("[CONSUMER {}] checking condition {}", i, buffer.len());
                        buffer.is_empty()
                    }).unwrap();
                    _guard.pop().unwrap()
                };

                println!("[CONSUMER {}] woke up - consuming {}", i, to_consume);
                thread::sleep(Duration::from_millis(1000));
                println!("[CONSUMER {}] done", i);

            }
        })
    }).collect();

    let _:Vec<()> = consumers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    producer.join().unwrap();
}