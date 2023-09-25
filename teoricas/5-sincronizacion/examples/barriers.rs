use std::sync::{Arc, Barrier};
use std::thread;

fn main() {
    let mut handles = Vec::with_capacity(10);
    let barrier = Arc::new(Barrier::new(10));
    for i in 0..10 {
        let c = barrier.clone();
        // Todos los mensajes se imprimen juntos
        // No hay solapamiento
        handles.push(thread::spawn(move|| {
            println!("Antes del wait");
            let barrier_wait_result = c.wait();
            println!("Despues del wait");
            println!("Soy el hilo {}: {:?}", i, barrier_wait_result.is_leader());
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }
}
