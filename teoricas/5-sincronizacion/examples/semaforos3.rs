use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std_semaphore::Semaphore;

fn main() {
    // Crear un semaforo que represente 1 recurso
    let sem = Arc::new(Semaphore::new(0));
    let c_sem = sem.clone();

    let _h = thread::spawn(move || {
        thread::sleep(Duration::from_millis(5_000));
        c_sem.release();
        println!("Soy el hijo: release!");
    });

    println!("Soy el padre: Voy a obtener el Semaforo.");

    let _guard = sem.access();

    println!("Soy el padre: Semaforo adquirido!");
}
