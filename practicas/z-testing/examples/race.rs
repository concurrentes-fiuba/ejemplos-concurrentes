use std::sync::{Arc, Barrier, Mutex};
use std::thread;

fn play(contenders_count:usize) -> usize {

    let winner = Arc::new(Mutex::new(None));

    let contenders = (0..contenders_count)
        .map(|id| {
            let winner_local = winner.clone();
            thread::spawn(move || {
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
    let contenders_count = 5;
    let mut wins = vec![0; contenders_count];
    for i in 0..10000 {
        wins[play(contenders_count)]+=1
    }
    println!("{:?}", wins);
}