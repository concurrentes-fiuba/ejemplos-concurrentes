extern crate std_semaphore;
extern crate rand;

use std_semaphore::Semaphore;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::sync::Arc;
use std::thread::JoinHandle;

const N:usize = 5;

/**
Cinco filósofos se sientan alrededor de una mesa y pasan su vida cenando y pensando.
Cada filósofo tiene un plato de fideos y un palito chino a la izquierda de su plato.
Para comer los fideos son necesarios dos palitos y cada filósofo sólo puede tomar los que
están a su izquierda y derecha. Si cualquier filósofo toma un palito y el otro está ocupado,
se quedará esperando, con el tenedor en la mano, hasta que pueda tomar el otro tenedor,
para luego empezar a comer.
*/

fn main() {
    let chopsticks:Arc<Vec<Semaphore>> = Arc::new((0 .. N)
        .map(|_| Semaphore::new(1))
        .collect());

    let philosophers:Vec<JoinHandle<()>> = (0 .. N)
        .map(|id| {
            let chopsticks_local = chopsticks.clone();
            thread::spawn(move || philosopher(id, chopsticks_local))
        })
        .collect();

    for philosopher in philosophers {
        philosopher.join();
    }

}

fn philosopher(id: usize, chopsticks: Arc<Vec<Semaphore>>) {
    let next = (id + 1) % N;
    let first_chopstick;
    let second_chopstick;

    // solucion al deadlock
    //if id == (N-1) {
    //    first_chopstick = &chopsticks[next];
    //    second_chopstick = &chopsticks[id];
    //} else {
       first_chopstick = &chopsticks[id];
       second_chopstick = &chopsticks[next];
    //}

    // tratar de forzar tomar en el primer palito en el orden de id
    thread::sleep(Duration::from_millis(100 * id as u64));

    loop {
        println!("filosofo {} pensando", id);
        //thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
        println!("filosofo {} esperando palito izquierdo", id);
        {
            let first_access = first_chopstick.access();
            // pausa despues del primer palito para forzar el deadlock
            thread::sleep(Duration::from_millis(1000));
            println!("filosofo {} esperando palito derecho", id);
            {
                let second_access = second_chopstick.access();
                println!("filosofo {} comiendo", id);
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)));
            }
        }
    }
}