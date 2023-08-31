use std::thread;
use std::time::Duration;

fn main() {
    println!("[PADRE] spawneo hijo");
    let join_handle = thread::spawn(move || {
        println!("[HIJO] spawnie");
        thread::sleep(Duration::from_secs(2));
        println!("[HIJO] me desperté");
        // panic!();
        42
    });
    println!("[PADRE] esperando hijo");
    let result = join_handle.join();
    match result {
        Ok(x) => println!("[PADRE] terminó y devolvió {}", x),
        Err(_) => println!("[PADRE] explotó")
    }
}