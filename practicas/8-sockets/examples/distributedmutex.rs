use std::any::Any;
use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream, UdpSocket, SocketAddr};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use std::mem::size_of;
use std_semaphore::Semaphore;
use std::collections::{HashMap, HashSet};

fn id_to_addr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

struct DistMutex {
    id: usize,
    socket: UdpSocket,
    lock_needed: Arc<Mutex<(Option<u128>, Vec<SocketAddr>)>>,
    ok_acc: Arc<(Mutex<HashSet<SocketAddr>>, Condvar)>,
}

impl DistMutex {
    fn new(id: usize) -> DistMutex {
        let ret = DistMutex {
            id,
            socket: UdpSocket::bind(id_to_addr(id)).unwrap(),
            lock_needed: Arc::new(Mutex::new((None, vec!()))),
            ok_acc: Arc::new((Mutex::new(HashSet::new()), Condvar::new())),
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.receiver());

        ret
    }

    fn acquire(&mut self) {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis();

        self.lock_needed.lock().unwrap().0 = Some(now);

        for sibling in 0..CLIENTS {
            if self.id != sibling {
                self.socket.send_to(&now.to_le_bytes(), id_to_addr(sibling)).unwrap();
            }
        }

        println!("[{}] esperando respuestas", self.id);
        self.ok_acc.1.wait_while(self.ok_acc.0.lock().unwrap(), |responded| responded.len() < (CLIENTS - 1));
        self.ok_acc.0.lock().unwrap().clear();
    }

    fn release(&mut self) {
        {
            let mut pair = self.lock_needed.lock().unwrap();
            pair.0 = None;
            for addr in &pair.1 {
                self.socket.send_to("OK".as_bytes(), addr).unwrap();
                println!("[{}] contesté a {}", self.id, addr);
            }
            pair.1.clear();
        }
    }

    fn receiver(&mut self) {
        loop {
            let mut buf = [0; size_of::<u128>()];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            if [b'O', b'K'].eq(&buf[0..2]) {
                println!("[{}] recibí OK de {}", self.id, from);
                self.ok_acc.0.lock().unwrap().insert(from);
                self.ok_acc.1.notify_all();
            } else {
                let requested_timestamp = u128::from_le_bytes(buf);
                println!("[{}] recibí pedido de {}. timestamp {}", self.id, from, requested_timestamp);
                let mut pair = self.lock_needed.lock().unwrap();
                match pair.0 {
                    Some(my_timestamp) if requested_timestamp < my_timestamp => {
                        self.socket.send_to("OK".as_bytes(), from).unwrap();
                        println!("[{}] pidió timestamp menor, contesté a {}", self.id, from);
                    }
                    None => {
                        // Esperar para forzar el interleaving
                        thread::sleep(Duration::from_millis(thread_rng().gen_range(500..1000)));
                        self.socket.send_to("OK".as_bytes(), from).unwrap();
                        println!("[{}] contesté a {}", self.id, from);
                    }
                    _ => {
                        pair.1.push(from);
                        println!("[{}] encolando a {}", self.id, from);
                    }
                }
            }
        }
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            lock_needed: self.lock_needed.clone(),
            ok_acc: self.ok_acc.clone(),
        }
    }
}

const CLIENTS: usize = 5;

fn main() {
    let mut handles = vec!();
    for id in 0..CLIENTS {
        handles.push(thread::spawn(move || { client(id) }));
    }
    handles.into_iter().for_each(|h| { h.join(); });
}

fn client(id: usize) {
    let mut mutex = DistMutex::new(id);
    println!("[{}] conectado", id);

    loop {
        println!("[{}] durmiendo", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000..3000)));
        println!("[{}] pidiendo lock", id);

        mutex.acquire();
        println!("[{}] tengo el lock", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000..3000)));
        println!("[{}] libero el lock", id);
        mutex.release();
    }
}