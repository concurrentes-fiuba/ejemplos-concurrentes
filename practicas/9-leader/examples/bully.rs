use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write};
use std::mem::size_of;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};
use std::convert::TryInto;

fn id_to_ctrladdr(id: usize) -> String { "127.0.0.1:1234".to_owned() + &*id.to_string() }
fn id_to_dataaddr(id: usize) -> String { "127.0.0.1:1235".to_owned() + &*id.to_string() }

const TEAM_MEMBERS: usize = 5;
const TIMEOUT: Duration = Duration::from_secs(5);

struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ok: Arc<(Mutex<bool>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
}

impl LeaderElection {
    fn new(id: usize) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ok: Arc::new((Mutex::new(false), Condvar::new())),
            stop: Arc::new((Mutex::new(false), Condvar::new()))
        };

        let mut clone = ret.clone();
        thread::spawn(move || clone.receiver());

        ret.find_new();
        ret
    }

    fn am_i_leader(&self) -> bool {
        self.get_leader_id() == self.id
    }

    fn get_leader_id(&self) -> usize {
        self.leader_id.1.wait_while(self.leader_id.0.lock().unwrap(), |leader_id| leader_id.is_none()).unwrap().unwrap()
    }

    fn find_new(&mut self) {
        if *self.stop.0.lock().unwrap() {
            return
        }
        if self.leader_id.0.lock().unwrap().is_none() {
            // ya esta buscando lider
            return
        }
        println!("[{}] buscando lider", self.id);
        *self.got_ok.0.lock().unwrap() = false;
        *self.leader_id.0.lock().unwrap() = None;
        self.send_election();
        let got_ok = self.got_ok.1.wait_timeout_while(self.got_ok.0.lock().unwrap(), TIMEOUT, |got_it| !*got_it );
        if !*got_ok.unwrap().0 {
            self.make_me_leader()
        } else {
            self.leader_id.1.wait_while(self.leader_id.0.lock().unwrap(), |leader_id| leader_id.is_none() );
        }

    }

    fn id_to_msg(&self, header:u8) -> Vec<u8> {
        let mut msg = vec!(header);
        msg.extend_from_slice(&self.id.to_le_bytes());
        msg
    }

    fn send_election(&self) {
        // P envía el mensaje ELECTION a todos los procesos que tengan número mayor
        let msg = self.id_to_msg(b'E');
        for peer_id in (self.id+1)..TEAM_MEMBERS {
            self.socket.send_to(&msg, id_to_ctrladdr(peer_id)).unwrap();
        }
    }

    fn make_me_leader(&self) {
        // El nuevo coordinador se anuncia con un mensaje COORDINATOR
        println!("[{}] me anuncio como lider", self.id);
        let msg = self.id_to_msg(b'C');
        for peer_id in 0..TEAM_MEMBERS {
            if peer_id != self.id {
                self.socket.send_to(&msg, id_to_ctrladdr(peer_id)).unwrap();
            }
        }
        *self.leader_id.0.lock().unwrap() = Some(self.id);
    }

    fn receiver(&mut self) {
        while !*self.stop.0.lock().unwrap() {
            let mut buf = [0; size_of::<usize>() + 1];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            let id_from = usize::from_le_bytes(buf[1..].try_into().unwrap());
            if *self.stop.0.lock().unwrap() {
                break;
            }
            match &buf[0] {
                b'O' => {
                    println!("[{}] recibí OK de {}", self.id, id_from);
                    *self.got_ok.0.lock().unwrap() = true;
                    self.got_ok.1.notify_all();
                }
                b'E' => {
                    println!("[{}] recibí Election de {}", self.id, id_from);
                    if id_from < self.id {
                        self.socket.send_to(&self.id_to_msg(b'O'), id_to_ctrladdr(id_from)).unwrap();
                        let mut me = self.clone();
                        thread::spawn(move || me.find_new());
                    }
                }
                b'C' => {
                    println!("[{}] recibí nuevo coordinador {}", self.id, id_from);
                    *self.leader_id.0.lock().unwrap() = Some(id_from);
                    self.leader_id.1.notify_all();
                }
                _ => {
                    println!("[{}] ??? {}", self.id, id_from);
                }
            }
        }
        *self.stop.0.lock().unwrap() = false;
        self.stop.1.notify_all();
    }

    fn stop(&mut self) {
        *self.stop.0.lock().unwrap() = true;
        self.stop.1.wait_while(self.stop.0.lock().unwrap(), |should_stop| *should_stop);
    }

    fn clone(&self) -> LeaderElection {
        LeaderElection {
            id: self.id,
            socket: self.socket.try_clone().unwrap(),
            leader_id: self.leader_id.clone(),
            got_ok: self.got_ok.clone(),
            stop: self.stop.clone(),
        }
    }
}

fn main() {
    let mut handles = vec!();
    for id in 0..TEAM_MEMBERS {
        handles.push(thread::spawn(move || { team_member(id) }));
    }
    handles.into_iter().for_each(|h| { h.join(); });
}

fn team_member(id: usize) {

    loop {

        println!("[{}] inicio", id);
        let mut scrum_master = LeaderElection::new(id);
        let mut socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        let mut buf = [0; 4];

        loop {

            if scrum_master.am_i_leader() {
                println!("[{}] soy SM", id);
                if thread_rng().gen_range(0, 100) >= 95 {
                    println!("[{}] me tomo vacaciones", id);
                    break;
                }
                socket.set_read_timeout(None);
                let (size, from) = socket.recv_from(&mut buf).unwrap();
                println!("[{}] doy trabajo a {}", id, from);
                socket.send_to("PONG".as_bytes(), from).unwrap();
            } else {
                let leader_id = scrum_master.get_leader_id();
                println!("[{}] pido trabajo al SM {}", id, leader_id);
                socket.send_to("PING".as_bytes(), id_to_dataaddr(leader_id)).unwrap();
                socket.set_read_timeout(Some(TIMEOUT)).unwrap();
                if let Ok((size, from)) = socket.recv_from(&mut buf) {
                    println!("[{}] trabajando", id);
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
                } else {
                    // por simplicidad consideramos que cualquier error necesita un lider nuevo
                    println!("[{}] SM caido, disparo elección", id);
                    scrum_master.find_new()
                }
            }
        }

        scrum_master.stop();

        thread::sleep(Duration::from_secs(30));

    }
}