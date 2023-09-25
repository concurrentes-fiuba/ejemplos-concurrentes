use std::sync::{Arc, Mutex, Condvar};
use std::thread;
use std::time::Duration;

fn main() {

    let pair = Arc::new((Mutex::new(true), Condvar::new()));

    let pair_clone = pair.clone();
    thread::spawn(move || {
        let (lock, cvar) = &*pair_clone;

        cvar.notify_all(); // spurious wakeup example

        println!("[awaited] doing expensive computation");
        thread::sleep(Duration::from_millis(1000));
        println!("[awaited] done");
        let mut pending = lock.lock().unwrap();
        println!("[awaited] got lock");
        *pending = false;
        println!("[awaited] notifying");
        cvar.notify_all();
    });


    let (lock, cvar) = &*pair;

    // As long as the value inside the `Mutex<bool>` is `true`, we wait.
    let _guard = cvar.wait_while(lock.lock().unwrap(), |pending| {
        println!("[waiter] checking condition {}", *pending);
        *pending
    }).unwrap();

    println!("[waiter] current mutex content {}", *_guard);
    println!("[waiter] done");

}
