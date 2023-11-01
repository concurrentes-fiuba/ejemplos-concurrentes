use std::{io, thread};
use std::collections::HashMap;
use std::convert::TryInto;
use std::io::BufRead;
use std::mem::size_of;
use std::net::UdpSocket;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

fn id_to_addr(id: usize) -> String { "127.0.0.1:1234".to_owned() + &*id.to_string() }

const STAKEHOLDERS: usize = 3;
const TIMEOUT: Duration = Duration::from_secs(10);
const TRANSACTION_COORDINATOR_ADDR: &str = "127.0.0.1:1234";

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
struct TransactionId(u32);

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
enum TransactionState {
    Wait,
    Commit,
    Abort,
}

struct TransactionCoordinator {
    log: HashMap<TransactionId, TransactionState>,
    socket: UdpSocket,
    responses: Arc<(Mutex<Vec<Option<TransactionState>>>, Condvar)>,
}


impl TransactionCoordinator {
    fn new() -> Self {
        let mut ret = TransactionCoordinator {
            log: HashMap::new(),
            socket: UdpSocket::bind(TRANSACTION_COORDINATOR_ADDR).unwrap(),
            responses: Arc::new((Mutex::new(vec![None; STAKEHOLDERS]), Condvar::new())),
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.receiver());

        ret
    }

    fn submit(&mut self, t: TransactionId) -> bool {
        match self.log.get(&t) {
            None => self.full_protocol(t, false),
            Some(TransactionState::Wait) => self.full_protocol(t, true),
            Some(TransactionState::Commit) => self.commit(t),
            Some(TransactionState::Abort) => self.abort(t)
        }
    }

    fn full_protocol(&mut self, t: TransactionId, force_continue: bool) -> bool {
        if self.prepare(t) {
            if t.0 % 10 != 5 || force_continue { self.commit(t) } else { self.abort(t) }
        } else {
            self.abort(t)
        }
    }

    fn prepare(&mut self, t: TransactionId) -> bool {
        self.log.insert(t, TransactionState::Wait);
        println!("[COORDINATOR] prepare {}", t.0);
        self.broadcast_and_wait(b'P', t, TransactionState::Commit)
    }

    fn commit(&mut self, t: TransactionId) -> bool {
        self.log.insert(t, TransactionState::Commit);
        println!("[COORDINATOR] commit {}", t.0);
        self.broadcast_and_wait(b'C', t, TransactionState::Commit)
    }

    fn abort(&mut self, t: TransactionId) -> bool {
        self.log.insert(t, TransactionState::Abort);
        println!("[COORDINATOR] abort {}", t.0);
        !self.broadcast_and_wait(b'A', t, TransactionState::Abort)
    }

    fn broadcast_and_wait(&self, message: u8, t: TransactionId, expected: TransactionState) -> bool {
        *self.responses.0.lock().unwrap() = vec![None; STAKEHOLDERS];
        let mut msg = vec!(message);
        msg.extend_from_slice(&t.0.to_le_bytes());
        for stakeholder in 0..STAKEHOLDERS {
            println!("[COORDINATOR] envio {} id {} a {}", message, t.0, stakeholder);
            self.socket.send_to(&msg, id_to_addr(stakeholder)).unwrap();
        }
        let responses = self.responses.1.wait_timeout_while(self.responses.0.lock().unwrap(), TIMEOUT, |responses| responses.iter().any(Option::is_none));
        if responses.is_err() {
            println!("[COORDINATOR] timeout {}", t.0);
            false
        } else {
            responses.unwrap().0.iter().all(|opt| opt.is_some() && opt.unwrap() == expected)
        }
    }

    fn receiver(&mut self) {
        loop {
            let mut buf = [0; size_of::<usize>() + 1];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            let id_from = usize::from_le_bytes(buf[1..].try_into().unwrap());

            match &buf[0] {
                b'C' => {
                    println!("[COORDINATOR] recibí COMMIT de {}", id_from);
                    self.responses.0.lock().unwrap()[id_from] = Some(TransactionState::Commit);
                    self.responses.1.notify_all();
                }
                b'A' => {
                    println!("[COORDINATOR] recibí ABORT de {}", id_from);
                    self.responses.0.lock().unwrap()[id_from] = Some(TransactionState::Abort);
                    self.responses.1.notify_all();
                }
                _ => {
                    println!("[COORDINATOR] ??? {}", id_from);
                }
            }
        }
    }

    fn clone(&self) -> Self {
        TransactionCoordinator {
            log: HashMap::new(),
            socket: self.socket.try_clone().unwrap(),
            responses: self.responses.clone(),
        }
    }
}

fn prompt() {
    println!("ingrese id de transaccion");
}

fn main() {
    let mut coordinator = TransactionCoordinator::new();

    prompt();
    for line in io::stdin().lock().lines() {
        if let Ok(transaction_id) = line.unwrap().parse::<u32>() {
            println!("{}", coordinator.submit(TransactionId(transaction_id)));
        }

        prompt();
    }
}