use std::convert::TryInto;
use std::io::{Read, Write};
use std::mem::size_of;
use std::net::{SocketAddr, UdpSocket};
use std::sync::{Arc, Condvar, mpsc, Mutex, RwLock};
use std::sync::mpsc::{Receiver, Sender};
use std::{env, thread};
use std::process::{Child, Command};
use std::time::Duration;

use rand::{Rng, thread_rng};

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
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        let clients: Vec<Child> = (0..TEAM_MEMBERS).map(|id| {
            Command::new("cargo")
                .arg("run")
                .arg("--example")
                .arg("ring")
                .arg("--")
                .arg(id.to_string())
                .spawn()
                .expect("failed to start child")
        }).collect();

        clients.into_iter().for_each(|mut client| { client.wait().unwrap(); })

    } else {
        let id = args[1].parse().expect("can't parse id");
        println!("soy proceso {}", id);
        TeamMember::new(id).run()
    }
}

struct TeamMember {
    id: usize,
    socket: UdpSocket,
    enabled: RwLock<bool>,
}

impl TeamMember {

    fn new(id: usize) -> Arc<Self> {
        let socket = UdpSocket::bind(id_to_dataaddr(id)).unwrap();
        Arc::new(TeamMember {
            id,
            socket,
            enabled: RwLock::new(true),
        })
    }

    fn run(self: &Arc<Self>) {

        let (got_pong, pong): (Sender<SocketAddr>, Receiver<SocketAddr>) = mpsc::channel();
        let this = self.clone();
        thread::spawn(move || this.receiver(got_pong));

        loop {

            println!("[{}] inicio", self.id);
            let mut scrum_master = LeaderElection::new(self.id);

            *self.enabled.write().unwrap() = true;

            loop {

                if scrum_master.am_i_leader() {
                    println!("[{}] soy SM", self.id);
                    thread::sleep(Duration::from_millis(thread_rng().gen_range(5000, 10000)));
                    println!("[{}] me tomo vacaciones", self.id);
                    *self.enabled.write().unwrap() = false;
                    break;
                } else {
                    let leader_id = scrum_master.get_leader_id();
                    println!("[{}] pido trabajo al SM {}", self.id, leader_id);
                    self.socket.send_to("PING".as_bytes(), id_to_dataaddr(leader_id)).unwrap();
                    if let Ok(addr) = pong.recv_timeout(TIMEOUT) {
                        println!("[{}] recibí trabajo de {}", self.id, addr);
                        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000)));
                    } else {
                        // por simplicidad consideramos que cualquier error necesita un lider nuevo
                        println!("[{}] SM caido, disparo elección", self.id);
                        scrum_master.find_new()
                    }
                }
            }

            scrum_master.stop();

            thread::sleep(Duration::from_secs(30));

        }
    }

    fn receiver(self: &Arc<Self>, got_pong: Sender<SocketAddr>) {
        const PING: [u8; 4] = [b'P', b'I', b'N', b'G'];
        const PONG: [u8; 4] = [b'P', b'O', b'N', b'G'];

        let mut buf = [0; 4];
        loop {
            match self.socket.recv_from(&mut buf) {
                Ok((size, from)) => {
                    match buf {
                        PING => {
                            println!("[{}] PING de {}", self.id, from);
                            if *self.enabled.read().unwrap() {
                                self.socket.send_to(&PONG, from).unwrap();
                            } else {
                                println!("[{}] ignorado", self.id)
                            }
                        }
                        PONG => {
                            got_pong.send(from).unwrap();
                        }
                        _ => println!("[{}] mensaje desconocido desde {}: {:?}", self.id, from, buf)
                    }
                }
                Err(e) => println!("[{}] error leyendo socket {}", self.id, e)
            }
        }
    }
}