extern crate rand;

use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};

fn main() {

    #[derive(Debug)]
    struct State {
        pending_first: bool,
        pending_second: bool
    }

    let pair = Arc::new((Mutex::new(State { pending_first: true, pending_second: true} ), Condvar::new()));

    let pair_clone = pair.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;

        println!("[awaited 1] doing expensive computation");
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("[awaited 1] done");
        let mut state = lock.lock().unwrap();
        println!("[awaited 1] got lock");
        state.pending_first = false;
        println!("[awaited 1] notifying");
        cvar.notify_all();
    });


    let pair_clone2 = pair.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone2;

        println!("[awaited 2] doing expensive computation");
        thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("[awaited 2] done");
        let mut state = lock.lock().unwrap();
        println!("[awaited 2] got lock");
        state.pending_second = false;
        println!("[awaited 2] notifying");
        cvar.notify_all();
    });


    let (lock, cvar) = &*pair;

    // As long as the value inside the `Mutex<bool>` is `true`, we wait.
    let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| {
        println!("[waiter] checking condition {:?}", *pending);
        pending.pending_first || pending.pending_second
    }).unwrap();

    println!("[waiter] done");

}