use std::thread;
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let vals = vec![
            String::from("Mensaje 1"),
            String::from("Mensaje 2"),
            String::from("Mensaje 3"),
            String::from("Mensaje 4"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            thread::sleep(Duration::from_secs(1));
        }
    });

    // tratar rx como un iterador
    for received in rx {
        println!("Recibido: {}", received);
    }
}
