use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn main() {
    let val = Arc::new(AtomicUsize::new(5));

    for _ in 0..10 {
        let val = Arc::clone(&val);

        thread::spawn(move || {
            let v = val.fetch_add(1, Ordering::SeqCst);
            println!("{:?}", v);
        });
    }
}