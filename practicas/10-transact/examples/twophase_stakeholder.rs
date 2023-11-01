use std::{env, thread};
use std::collections::HashMap;
use std::convert::TryInto;
use std::mem::size_of;
use std::net::UdpSocket;
use std::time::Duration;

const STAKEHOLDERS: usize = 3;

enum TransactionState {
    Accepted,
    Commit,
    Abort
}

fn msg(message: u8, id:usize) -> Vec<u8> {
    let mut msg = vec!(message);
    msg.extend_from_slice(&id.to_le_bytes());
    msg
}

fn stakeholder(id:usize) {

    let mut log = HashMap::new();
    let socket = UdpSocket::bind("127.0.0.1:1234".to_owned() + &*id.to_string()).unwrap();

    println!("[{}] hola", id);

    loop {
        let mut buf = [0; size_of::<usize>() + 1];
        let (size, from) = socket.recv_from(&mut buf).unwrap();
        let transaction_id = usize::from_le_bytes(buf[1..].try_into().unwrap());


        match &buf[0] {
            b'P' => {
                println!("[{}] recibí PREPARE para {}", id, transaction_id);
                let m = match log.get(&transaction_id) {
                    Some(TransactionState::Accepted) | Some(TransactionState::Commit) => b'C',
                    Some(TransactionState::Abort) => b'A',
                    None => {
                        if transaction_id % 10 != id {
                            // TODO tomar recursos
                            log.insert(transaction_id, TransactionState::Accepted);
                            b'C'
                        } else {
                            log.insert(transaction_id, TransactionState::Abort);
                            b'A'
                        }
                    }
                };
                thread::sleep(Duration::from_millis(1000));
                socket.send_to(&*msg(m, id), from).unwrap();
                // TODO: iniciar un timeout
            }
            b'C' => {
                println!("[{}] recibí COMMIT de {}", id, transaction_id);
                // TODO: verificar el estado. Si es Accepted, realizar el commit internamente
                // TODO: si es commit, solo contestar
                // TODO: de otra forma, fallar
                log.insert(transaction_id, TransactionState::Commit);
                thread::sleep(Duration::from_millis(1000));
                socket.send_to(&*msg(b'C', id), from).unwrap();
            }
            b'A' => {
                println!("[{}] recibí ABORT de {}", id, transaction_id);
                // TODO: verificar el estado. Si es Accepted, liberar recursos.
                // TODO: si es abort o no conocia esta transacción, solo contestar
                // TODO: de otra forma, fallar
                log.insert(transaction_id, TransactionState::Abort);
                thread::sleep(Duration::from_millis(1000));
                socket.send_to(&*msg(b'A', id), from).unwrap();
            }
            _ => {
                println!("[{}] ??? {}", id, transaction_id);
            }
        }

    }

}

fn main() {

    let args: Vec<String> = env::args().collect();

    match args[1].parse() {
        Ok(id) if id >= 0 && id < STAKEHOLDERS => {
            stakeholder(id);
        }
        _ => println!("id invalido")
    }
}