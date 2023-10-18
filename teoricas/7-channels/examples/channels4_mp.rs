use std::thread;
use std::sync::mpsc;
//use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();
    let tx1 = mpsc::Sender::clone(&tx);

    thread::spawn(move || {
        let vals = vec![
            String::from("Hilo 1, mensaje 1"),
            String::from("Hilo 1, mensaje 2"),
            String::from("Hilo 1, mensaje 3"),
            String::from("Hilo 1, mensaje 4"),
        ];

        for val in vals {
            tx1.send(val).unwrap();
            //thread::sleep(Duration::from_millis(1));
        }
    });

    thread::spawn(move || {
        let vals = vec![
            String::from("Hilo 2, mensaje 1"),
            String::from("Hilo 2, mensaje 2"),
            String::from("Hilo 2, mensaje 3"),
            String::from("Hilo 2, mensaje 4"),
        ];

        for val in vals {
            tx.send(val).unwrap();
            //thread::sleep(Duration::from_millis(1));
        }
    });

    for received in rx {
        println!("Recibido: {}", received);
    }
}
