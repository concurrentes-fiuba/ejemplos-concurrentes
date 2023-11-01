use std::any::Any;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, BufReader, Write, Cursor, Read};
use std::mem::size_of;
use std::net::{SocketAddr, TcpListener, TcpStream, UdpSocket};
use std::sync::{Arc, Condvar, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use rand::{Rng, thread_rng};
use std::convert::TryInto;

fn id_to_ctrladdr(id: usize) -> String { "127.0.0.1:1234".to_owned() + &*id.to_string() }
fn id_to_dataaddr(id: usize) -> String { "127.0.0.1:1238".to_owned() + &*id.to_string() }

const TEAM_MEMBERS: usize = 5;
const TIMEOUT: Duration = Duration::from_secs(5);

struct LeaderElection {
    id: usize,
    socket: UdpSocket,
    leader_id: Arc<(Mutex<Option<usize>>, Condvar)>,
    got_ack: Arc<(Mutex<Option<usize>>, Condvar)>,
    stop: Arc<(Mutex<bool>, Condvar)>,
}

impl LeaderElection {
    fn new(id: usize) -> LeaderElection {
        let mut ret = LeaderElection {
            id,
            socket: UdpSocket::bind(id_to_ctrladdr(id)).unwrap(),
            leader_id: Arc::new((Mutex::new(Some(id)), Condvar::new())),
            got_ack: Arc::new((Mutex::new(None), Condvar::new())),
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

    fn next(&self, id:usize) -> usize {
        (id + 1) % TEAM_MEMBERS
    }

    fn find_new(&mut self) {
        if *self.stop.0.lock().unwrap() {
            return
        }
        println!("[{}] buscando lider", self.id);
        *self.leader_id.0.lock().unwrap() = None;

        self.safe_send_next(&self.ids_to_msg(b'E', &[self.id]), self.id);

        self.leader_id.1.wait_while(self.leader_id.0.lock().unwrap(), |leader_id| leader_id.is_none());

    }

    fn ids_to_msg(&self, header:u8, ids:&[usize]) -> Vec<u8> {
        let mut msg = vec!(header);
        msg.extend_from_slice(&ids.len().to_le_bytes());
        for id in ids {
            msg.extend_from_slice(&id.to_le_bytes());
        }
        msg
    }

    fn safe_send_next(&self, msg: &[u8], id:usize) {
        let next_id = self.next(id);
        if next_id == self.id {
            println!("[{}] enviando {} a {}", self.id, msg[0] as char, next_id);
            panic!("Di toda la vuelta sin respuestas")
        }
        *self.got_ack.0.lock().unwrap() = None;
        self.socket.send_to(msg, id_to_ctrladdr(next_id));
        let got_ack = self.got_ack.1.wait_timeout_while(self.got_ack.0.lock().unwrap(), TIMEOUT, |got_it| got_it.is_none() || got_it.unwrap() != next_id );
        if got_ack.unwrap().1.timed_out() {
            self.safe_send_next(msg, next_id)
        }

    }

    fn receiver(&mut self) {
        while !*self.stop.0.lock().unwrap() {
            let mut buf = [0; 1 + size_of::<usize>() + (TEAM_MEMBERS+1) * size_of::<usize>()];
            let (size, from) = self.socket.recv_from(&mut buf).unwrap();
            let (msg_type, mut ids) = self.parse_message(&buf);

            match msg_type {
                b'A' => {
                    println!("[{}] recibí ACK de {}", self.id, from);
                    *self.got_ack.0.lock().unwrap() = Some(ids[0]);
                    self.got_ack.1.notify_all();
                }
                b'E' => {
                    println!("[{}] recibí Election de {}, ids {:?}", self.id, from, ids);
                    self.socket.send_to(&self.ids_to_msg(b'A', &[self.id]), from).unwrap();
                    if ids.contains(&self.id) {
                        // dio toda la vuelta, cambiar a COORDINATOR
                        let winner = *ids.iter().max().unwrap();
                        self.socket.send_to(&self.ids_to_msg(b'C', &[winner, self.id]), from).unwrap();
                    } else {
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'E', &ids);
                        let clone = self.clone();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                b'C' => {
                    println!("[{}] recibí nuevo coordinador de {}, ids {:?}", self.id, from, ids);
                    *self.leader_id.0.lock().unwrap() = Some(ids[0]);
                    self.leader_id.1.notify_all();
                    self.socket.send_to(&self.ids_to_msg(b'A', &[self.id]), from).unwrap();
                    if !ids[1..].contains(&self.id) {
                        ids.push(self.id);
                        let msg = self.ids_to_msg(b'C', &ids);
                        let clone = self.clone();
                        thread::spawn(move || clone.safe_send_next(&msg, clone.id));
                    }
                }
                _ => {
                    println!("[{}] ??? {:?}", self.id, ids);
                }
            }
        }
        *self.stop.0.lock().unwrap() = false;
        self.stop.1.notify_all();
    }

    fn parse_message(&self, buf: &[u8]) -> (u8, Vec<usize>) {
        let mut ids = vec!();

        let mut count = usize::from_le_bytes(buf[1..1+size_of::<usize>()].try_into().unwrap());

        let mut pos = 1+size_of::<usize>();
        for id in 0..count {
            ids.push(usize::from_le_bytes(buf[pos..pos+size_of::<usize>()].try_into().unwrap()));
            pos += size_of::<usize>();
        }

        (buf[0], ids)
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
            got_ack: self.got_ack.clone(),
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

        let mut socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        let mut buf = [0; 4];
        let mut scrum_master = LeaderElection::new(id);

        loop {

            if scrum_master.am_i_leader() {
                println!("[{}] soy SM", id);
                if thread_rng().gen_range(0, 100) >= 90 {
                    println!("[{}] me tomo vacaciones", id);
                    scrum_master.stop();
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

        thread::sleep(Duration::from_secs(20));

    }
}