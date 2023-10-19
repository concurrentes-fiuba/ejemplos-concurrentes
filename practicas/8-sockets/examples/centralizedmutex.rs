use std::net::{TcpListener, TcpStream};
use std::io::{BufReader, BufRead, Write};
use std::thread;
use std::sync::{Mutex, Arc, Condvar};
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::any::Any;
use std_semaphore::Semaphore;


struct DistMutex {
    writer: TcpStream,
    reader: BufReader<TcpStream>
}

impl DistMutex {
    fn new(id:u32) -> DistMutex {
        let mut stream = TcpStream::connect("127.0.0.1:12345").unwrap();
        let mut ret = DistMutex {
            writer: stream.try_clone().unwrap(),
            reader: BufReader::new(stream)
        };

        ret.writer.write_all((id.to_string() + "\n").as_bytes() );

        ret
    }

    fn acquire(&mut self) {
        self.writer.write_all("acquire\n".as_bytes()).unwrap();

        let mut buffer = String::new();
        self.reader.read_line(&mut buffer);
    }

    fn release(&mut self) {
        self.writer.write_all("release\n".as_bytes()).unwrap();
    }
}

const CLIENTS: u32 = 3;

fn main() {
    let coordinator = thread::spawn(coordinator);
    for id in 0..CLIENTS {
        thread::spawn(move || { client(id) });
    }
    coordinator.join();
}

fn client(id:u32) {

    let mut mutex = DistMutex::new(id);
    println!("[{}] conectado", id);

    let mut count = 0;

    loop {
        println!("[{}] durmiendo", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000u64..3000)));
        println!("[{}] pidiendo lock", id);

        mutex.acquire();
        println!("[{}] tengo el lock", id);
        thread::sleep(Duration::from_millis(thread_rng().gen_range(1000u64..3000)));

        count += 1;
        if count > 2 {
            break;
        }

        println!("[{}] libero el lock", id);
        mutex.release();
    }

    println!("[{}] salí", id);
}

fn coordinator() {

    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    let mutex = Arc::new(Semaphore::new(1));

    for stream in listener.incoming() {
        let tcp_stream = stream.unwrap();
        let mut writer = tcp_stream.try_clone().unwrap();
        let mut reader = BufReader::new(tcp_stream);
        let local_mutex = mutex.clone();
        let mut id = String::new();
        reader.read_line(&mut id);
        id = id.replace("\n", "");
        println!("[COORDINATOR] Cliente conectado {}", id);
        thread::spawn(move || receiver(writer, reader, local_mutex, id));
    }

}

fn receiver(mut writer: TcpStream, mut reader: BufReader<TcpStream>, local_mutex: Arc<Semaphore>, id: String) {
    let mut mine = false;

    loop {
        let mut buffer = String::new();
        reader.read_line(&mut buffer);
        match buffer.as_str() {
            "acquire\n" => {
                println!("[COORDINATOR] pide lock {}", id);
                if !mine {
                    local_mutex.acquire();
                    mine = true;
                    writer.write_all("OK\n".as_bytes()).unwrap();
                    println!("[COORDINATOR] le dí lock a {}", id);
                } else {
                    println!("[COORDINATOR] ERROR: ya lo tiene");
                }
            }
            "release\n" => {
                println!("[COORDINATOR] libera lock {}", id);
                if mine {
                    local_mutex.release();
                    mine = false;
                } else {
                    println!("[COORDINATOR] ERROR: no lo tiene!")
                }
            }
            "" => {
                println!("[COORDINATOR] desconectado {}", id);
                break;
            }
            _ => {
                println!("[COORDINATOR] ERROR: mensaje desconocido de {}", id);
                break;
            }
        }
    }
    if mine {
        println!("[COORDINATOR] ERROR: {} tenia el lock. Liberación forzosa", id);
        local_mutex.release();
    }
}