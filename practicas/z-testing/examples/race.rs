use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};

fn play(contenders_count:i32) -> i32 {

    let winner = Arc::new(Mutex::new(None));

    let contenders = (0..contenders_count)
        .map(|id| {
            let winner_local = winner.clone();
            thread::spawn(move || {
                thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 2000)));
                let mut winner = winner_local.lock().unwrap();
                if *winner == None {
                    *winner = Some(id)
                }
            })
        }).collect::<Vec<thread::JoinHandle<()>>>();

    contenders
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .for_each(drop);

    let and_the_winner_is = winner.lock().unwrap().unwrap();
    and_the_winner_is
}

fn main() {
    println!("{:?}", play(5))
}