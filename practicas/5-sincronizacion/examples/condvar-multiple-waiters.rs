use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;

fn main() {

    const N:i32 = 5;

    let pair = Arc::new((Mutex::new(true), Condvar::new()));

    let pair_clone = pair.clone();
    let awaited = thread::spawn(move || {
        loop {
            let (lock, cvar) = &*pair_clone;

            println!("[awaited] doing expensive computation");
            thread::sleep(Duration::from_millis(1000));
            println!("[awaited] done");
            let mut pending = lock.lock().unwrap();
            println!("[awaited] got lock");
            *pending = false;
            println!("[awaited] notifying");

            // play with notify_one
            cvar.notify_all();
        }
    });

    let waiters:Vec<JoinHandle<()>> = (1..N).map(|i| {
        let pair_clone_waiter = pair.clone();
        thread::spawn(move || {
            loop {
                let (lock, cvar) = &*pair_clone_waiter;

                let mut _guard = cvar.wait_while(lock.lock().unwrap(), |pending| {
                    println!("[waiter {}] checking condition {}", i, *pending);
                    *pending
                }).unwrap();

                println!("[waiter {}] woke up", i);

                thread::sleep(Duration::from_millis(1000));
                (*_guard) = true;

                println!("[waiter {}] done", i);
            }
        })
    }).collect();

    let _:Vec<()> = waiters.into_iter()
        .flat_map(|x| x.join())
        .collect();

    awaited.join().unwrap();
}