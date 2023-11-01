use rand::{thread_rng, Rng};
use rand::prelude::ThreadRng;

fn main() {
    let random:f64 = thread_rng().gen();
    if random >= 0.5 {
        println!("Ganaste")
    } else {
        println!("Perdiste")
    }
}

