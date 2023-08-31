//use crossbeam_utils::thread;

fn main() {
    crossbeam::scope(|s| {
        let handle = s.spawn(|_| {
            println!("A child thread is running");
            42
        });
    
        // Join the thread and retrieve its result.
        let res = handle.join().unwrap();
        assert_eq!(res, 42);
    }).unwrap();
}
