use std::collections::HashMap;
use std::time::Duration;

use mockall_double::double;
use crate::sync::{Arc, Mutex, thread};

mod sync {
    use std::time::Duration;

    #[cfg(not(loom))]
    pub(crate) use std::sync::{Arc, Mutex};

    #[cfg(loom)]
    pub(crate) use loom::sync::{Arc, Mutex};

    #[cfg(not(loom))]
    pub(crate) use std::thread;

    #[cfg(loom)]
    pub(crate) use loom::thread;
    use rand::{Rng, thread_rng};
    use rand::distributions::uniform::SampleUniform;

    #[cfg(not(loom))]
    pub(crate) fn sleep(d:Duration) {
        thread::sleep(d);
    }

    #[cfg(loom)]
    pub(crate) fn sleep(d:Duration) {
        loom::thread::yield_now();
    }

    #[cfg(not(loom))]
    pub(crate) fn rand<T: SampleUniform>(low:T, high:T) -> T {
        thread_rng().gen_range(low, high)
    }

    #[cfg(loom)]
    pub(crate) fn rand<T>(low:T, high:T) -> T {
        low
    }
}

fn play(contenders_count:i32) -> i32 {

    let winner = Arc::new(Mutex::new(None));

    let contenders = (0..contenders_count)
        .map(|id| {
            let winner_local = winner.clone();
            sync::thread::spawn(move || {
                sync::sleep(Duration::from_millis(sync::rand(500, 2000)));
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
