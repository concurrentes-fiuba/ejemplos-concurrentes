use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::mem::size_of;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::AtomicBool;
use std::{env, thread};
use std::process::{Child, Command};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rand::{Rng, thread_rng};
use std_semaphore::Semaphore;

fn id_to_addr(id: usize) -> String {
    "127.0.0.1:1234".to_owned() + &*id.to_string()
}

struct DistMutex {
    id: usize,
    socket: UdpSocket,
    lock_needed: Arc<(Mutex<bool>, Condvar)>,
    has_token: Arc<(Mutex<bool>, Condvar)>,
}

impl DistMutex {
    fn new(id: usize) -> DistMutex {

        let ret = DistMutex {
            id,
            socket: UdpSocket::bind(id_to_addr(id)).unwrap(),
            lock_needed: Arc::new((Mutex::new(false), Condvar::new())),
            has_token: Arc::new((Mutex::new(false), Condvar::new()))
        };

        let mut clone = ret.clone();

        thread::spawn(move || clone.receiver());

        ret
    }

    fn acquire(&mut self) {
        *self.lock_needed.0.lock().unwrap() = true;
        self.lock_needed.1.notify_all();

        self.has_token.1.wait_while(self.has_token.0.lock().unwrap(), |has_it| !*has_it);
    }

    fn release(&mut self) {
        *self.lock_needed.0.lock().unwrap() = false;
        self.lock_needed.1.notify_all();
    }

    fn receiver(&mut self) {
        if self.id == 0 {
            self.socket.send_to("TOKEN".as_bytes(), id_to_addr(0)).unwrap();
        }
        loop {
            let mut buf = [0; 10];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            println!("[{}] recibí token", self.id);
            *self.has_token.0.lock().unwrap() = true;
            self.has_token.1.notify_all();
            self.lock_needed.1.wait_while(self.lock_needed.0.lock().unwrap(), |needs_it| *needs_it);
            *self.has_token.0.lock().unwrap() = false;
            self.has_token.1.notify_all();
            thread::sleep(Duration::from_millis(100));
            self.socket.send_to("TOKEN".as_bytes(), id_to_addr((self.id + 1) % CLIENTS)).unwrap();
        }
    }

    fn clone(&self) -> DistMutex {
        DistMutex {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            lock_needed: self.lock_needed.clone(),
            has_token: self.has_token.clone(),
        }
    }
}

const CLIENTS: usize = 5;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let clients: Vec<Child> = (0..CLIENTS).map(|id| {
            Command::new("cargo")
                .arg("run")
                .arg("--example")
                .arg("tokenring")
                .arg("--")
                .arg(id.to_string())
                .spawn()
                .expect("failed to start child")
        }).collect();

        clients.into_iter().for_each(|mut client| { client.wait().unwrap(); })

    } else {
        let id = args[1].parse().expect("can't parse id");
        println!("soy proceso {}", id);
        client(id)
    }
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