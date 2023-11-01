use std::thread;

mod processor {
    use std::thread;
    use std::time::Duration;

    pub struct Process;

    impl Process {
        pub fn run(input: i32) -> i32 {
            thread::sleep(Duration::from_secs((input / 3) as u64));
            input
        }
    }

}

use processor::Process;

fn run_process() -> i32 {
    let a = thread::spawn(|| Process::run(42));
    let b = thread::spawn(|| Process::run(13));

    a.join().expect("missing a") + b.join().expect("missing b")
}

fn main() {
    println!("{}", run_process())
}