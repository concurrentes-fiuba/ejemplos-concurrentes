use std::thread;
use std::sync::mpsc;
use std::time::Duration;

fn main() {
    let (tx, rx) = mpsc::channel();

    thread::spawn(move || {
        let val = String::from("hola");
        println!("Voy a enviar");
        thread::sleep(Duration::from_millis(5000));
        tx.send(val).unwrap();
    });

    let received = rx.recv().unwrap();
    println!("Recibido: {}", received);
    //let received = rx.recv().unwrap();
    //println!("Recibido: {}", received);

}
