use std::thread;

use mockall_double::double;

mod processor {
    use std::thread;
    use std::time::Duration;

    use mockall::automock;

    pub struct Process;

    #[cfg_attr(test, automock)]
    impl Process {
        pub fn run(input: i32) -> i32 {
            thread::sleep(Duration::from_secs((input / 3) as u64));
            input
        }
    }

}

#[double]
use processor::Process;

fn run_process() -> i32 {
    let a = thread::spawn(|| Process::run(42));
    let b = thread::spawn(|| Process::run(13));

    a.join().expect("missing a") + b.join().expect("missing b")
}

fn main() {
    println!("{}", run_process())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_sleep() {
        let ctx = Process::run_context();
        ctx.expect()
            .times(2)
            .returning(|x| {
                println!("sleeping");
                thread::yield_now();
                x
            });
        assert_eq!(55, run_process());
    }
}