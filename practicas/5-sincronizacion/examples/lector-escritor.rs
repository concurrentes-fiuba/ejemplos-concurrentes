extern crate num_derive;
extern crate num_traits;
extern crate rand;

use std::cell::UnsafeCell;
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use rand::{Rng, thread_rng};
use rand::seq::SliceRandom;

#[derive(Debug)]
struct ReadWrite {
    readers: i32,
    writing: bool
}

struct DataHolder {
    data: UnsafeCell<i32>
}
unsafe impl Sync for DataHolder {}


fn main() {
    const READERS: i32 = 5;
    const WRITERS: i32 = 2;

    let pair = Arc::new((Mutex::new(ReadWrite { readers: 0, writing: false }), Condvar::new()));
    let data = Arc::new(DataHolder { data: UnsafeCell::new(42) } );

    let readers: Vec<JoinHandle<()>> = (0..READERS)
        .map(|me| {
            let pair_reader = pair.clone();
            let data_reader = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_reader;

                // Sacar esto para llegar a starvation del writer
                // thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                {
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Lector {}] Chequeando {:?}", me, state);
                        state.writing
                    }).unwrap();
                    _guard.readers += 1;
                }

                unsafe {
                    println!("[Lector {:?}] Leyendo {}", me, data_reader.data.get().read());
                }
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Lector {:?}] Terminé", me);

                lock.lock().unwrap().readers -= 1;
                cvar.notify_all();
            })
        })
        .collect();

    let writers: Vec<JoinHandle<()>> = (0..WRITERS)
        .map(|me| {
            let pair_writer = pair.clone();
            let data_writer = data.clone();

            thread::spawn(move || loop {
                let (lock, cvar) = &*pair_writer;

                {
                    let mut _guard = cvar.wait_while(lock.lock().unwrap(), |state| {
                        println!("[Escritor {}] Chequeando {:?}", me, state);
                        state.writing || state.readers > 0
                    }).unwrap();
                    _guard.writing = true;
                }

                unsafe {
                    println!("[Escritor {:?}] Escribiendo", me);
                    data_writer.data.get().write(me);
                }
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
                println!("[Escritor {:?}] Terminé", me);

                lock.lock().unwrap().writing = false;
                cvar.notify_all();
            })
        })
        .collect();

    let _:Vec<()> = readers.into_iter()
        .chain(writers.into_iter())
        .flat_map(|x| x.join())
        .collect();

}