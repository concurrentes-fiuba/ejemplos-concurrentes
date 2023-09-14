use std::sync::{Arc, RwLock};
use std::{thread, time};
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;



fn main() -> std::io::Result<()> {

    let file = File::create("file.txt")?;
    let lock = Arc::new(RwLock::new(file));
    
    let lock_clone_a = lock.clone();
    let thread_a = thread::Builder::new().name("thread-A".to_string()).spawn(move || {
        for _ in 1..10 {
            if let Ok(mut file_guard) = lock_clone_a.write() {
                let file_size = file_guard.metadata().unwrap().len();
                file_guard.seek(SeekFrom::Start(file_size)).unwrap();
                file_guard.write_all(b"A").unwrap();
            }
            println!("yield A");
            thread::yield_now();
          
        }

    }).unwrap();

    let lock_clone_b = lock.clone();
    let thread_b =  thread::Builder::new().name("thread-B".to_string()).spawn(move || {
        for _ in 1..10 {
            if let Ok(mut file_guard) = lock_clone_b.write() {
                let file_size = file_guard.metadata().unwrap().len();
                file_guard.seek(SeekFrom::Start(file_size)).unwrap();
                file_guard.write_all(b"B").unwrap();
            }
            println!("yield B");
            thread::yield_now();
          
        }

    }).unwrap();

    let ten_millis = time::Duration::from_millis(10);
    thread::sleep(ten_millis);
    
    thread_b.join().unwrap();
    thread_a.join().unwrap();

    Ok(())    
}