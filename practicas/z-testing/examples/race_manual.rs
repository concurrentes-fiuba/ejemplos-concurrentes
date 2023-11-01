use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;

use rand::Rng;

#[cfg(not(test))]
use contender::ContenderImpl;
#[cfg(test)]
use tests::TestableContenderImpl as ContenderImpl;

use crate::contender::Contender;

mod contender {
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use rand::{Rng, thread_rng};

    pub(crate) trait Contender {

        fn new(id: i32) -> Self;

        fn run(&self, winner_mutex: Arc<Mutex<Option<i32>>>) {
            self.sleep();
            let mut winner = winner_mutex.lock().unwrap();
            if *winner == None {
                *winner = Some(self.get_id())
            }
            self.finish();

        }

        fn sleep(&self) {
            thread::sleep(Duration::from_millis(thread_rng().gen_range(500, 2000)));
        }

        fn finish(&self) {}

        fn get_id(&self) -> i32;
    }

    pub(crate) struct ContenderImpl {
        id: i32
    }

    impl Contender for ContenderImpl {

        fn new(id: i32) -> ContenderImpl {
            ContenderImpl {
                id,
            }
        }

        fn get_id(&self) -> i32 {
            return self.id
        }
    }

}

fn play(contenders_count:i32) -> i32 {

    let winner = Arc::new(Mutex::new(None));

    let contenders = (0..contenders_count)
        .map(|id| {
            let winner_local = winner.clone();
            thread::spawn(move || {
                ContenderImpl::new(id).run(winner_local);
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

#[cfg(test)]
mod tests {
    use std::sync::Condvar;

    use lazy_static::lazy_static;
    use serial_test::serial;

    use crate::contender::Contender;
    use crate::tests::TestableContenderState::{AWAKE, FINISHED, SLEEPING};

    use super::*;

    lazy_static! {
        static ref contenders: (Mutex<HashMap<i32, TestableContenderState>>, Condvar) = (Mutex::new(HashMap::new()), Condvar::new());
    }

    #[derive(PartialEq)]
    enum TestableContenderState {
        STARTED,
        SLEEPING,
        AWAKE,
        FINISHED
    }

    pub(crate) struct TestableContenderImpl {
        id: i32,
    }

    impl Contender for TestableContenderImpl {

        fn new(id: i32) -> TestableContenderImpl {

            let contender = TestableContenderImpl {
                id
            };

            set_and_notify(id, TestableContenderState::STARTED);

            contender

        }

        fn sleep(&self) {
            set_and_wait(self.id, SLEEPING, TestableContenderState::AWAKE);
        }

        fn finish(&self) {
            set_and_notify(self.id, FINISHED);
        }

        fn get_id(&self) -> i32 {
            self.id
        }
    }

    fn set_and_notify(id:i32, state:TestableContenderState) {
        contenders.0.lock().unwrap().insert(id, state);
        contenders.1.notify_all();
    }

    fn set_and_wait(id:i32, to_set:TestableContenderState, to_await:TestableContenderState) {
        let mut guard = contenders.0.lock().unwrap();
        guard.insert(id, to_set);
        contenders.1.notify_all();
        let _ = contenders.1.wait_while(guard, |c| *c.get(&id).unwrap() != to_await).unwrap();
    }

    #[test]
    #[serial]
    fn test_should_win_0() {

        contenders.0.lock().unwrap().clear();

        let playing = thread::spawn(move || {
            play(3)
        });

        {
            let _ = contenders.1
                .wait_while(contenders.0.lock().unwrap(), |w| w.len() < 3 || w.values().any(|s| *s != SLEEPING))
                .unwrap();
        }
        set_and_wait(0, AWAKE, FINISHED);
        set_and_wait(1, AWAKE, FINISHED);
        set_and_wait(2, AWAKE, FINISHED);

        assert_eq!(0, playing.join().unwrap());
    }

    #[test]
    #[serial]
    fn test_should_win_2() {

        contenders.0.lock().unwrap().clear();

        let playing = thread::spawn(move || {
            play(3)
        });

        let _ = contenders.1
            .wait_while(contenders.0.lock().unwrap(), |w| w.len() < 3 || w.values().any(|s| *s != SLEEPING)).unwrap();

        set_and_wait(2, AWAKE, FINISHED);
        set_and_wait(1, AWAKE, FINISHED);
        set_and_wait(0, AWAKE, FINISHED);

        assert_eq!(2, playing.join().unwrap());
    }

    #[test]
    #[serial]
    fn test_should_win_1() {

        contenders.0.lock().unwrap().clear();

        let playing = thread::spawn(move || {
            play(3)
        });

        let _ = contenders.1
            .wait_while(contenders.0.lock().unwrap(), |w| w.len() < 3 || w.values().any(|s| *s != SLEEPING)).unwrap();

        set_and_wait(1, AWAKE, FINISHED);
        set_and_wait(0, AWAKE, FINISHED);
        set_and_wait(2, AWAKE, FINISHED);
        assert_eq!(1, playing.join().unwrap());
    }
}