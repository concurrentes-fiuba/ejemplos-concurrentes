use crate::sync::{Arc, Mutex, thread};

mod sync {
    #[cfg(not(loom))]
    pub(crate) use std::sync::{Arc, Mutex};

    #[cfg(loom)]
    pub(crate) use loom::sync::{Arc, Mutex};

    #[cfg(not(loom))]
    pub(crate) use std::thread;

    #[cfg(loom)]
    pub(crate) use loom::thread;
}

fn play(contenders_count:i32) -> i32 {

    let winner = Arc::new(Mutex::new(None));

    let contenders = (0..contenders_count)
        .map(|id| {
            let winner_local = winner.clone();
            sync::thread::spawn(move || {
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

#[cfg(loom)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;

    #[test]
    fn loom_test() {

        let result = std::sync::Arc::new(std::sync::Mutex::new(vec!(false, false, false)));
        let result_local = result.clone();

        loom::model(move || {
            let run_result = play(3);
            println!("winner {}", run_result);
            result_local.lock().unwrap()[run_result as usize] = true;
        });

        assert_eq!(vec!(true, true, true), *result.lock().unwrap());
    }

}
