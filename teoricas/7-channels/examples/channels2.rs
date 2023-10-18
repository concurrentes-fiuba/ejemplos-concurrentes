use std::thread;
use std::sync::mpsc;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let val = String::from("hola");
        tx.send(val).unwrap();
        println!("Valor es {}", val);
    });

    let received = rx.recv().unwrap();
    println!("Recibido: {}", received);
}
