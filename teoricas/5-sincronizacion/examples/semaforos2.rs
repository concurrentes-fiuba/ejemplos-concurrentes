use std_semaphore::Semaphore;

fn main() {
    // Crear un semaforo que represente 1 recurso
    let sem = Semaphore::new(1);

    let a = 10;
    // Adquirir el recurso durante el scope
    {
        let _guard = sem.access();
        println!("Semaforo adquirido!");
        println!("{}", a);
    } // recurso liberado aca

    println!("Semaforo liberado!");
}
