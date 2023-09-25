use std_semaphore::Semaphore;

fn main() {
    // Crear un semaforo que represente 1 recurso
    let sem = Semaphore::new(0);

    // Adquirir el recurso
    sem.acquire();

    println!("Semaforo adquirido!");

    // Liberarlo
    sem.release();

    println!("Semaforo liberado!");
}
