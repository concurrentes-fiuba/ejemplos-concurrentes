use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Message, StreamHandler};
use actix_async_handler::async_handler;
use rand::{thread_rng, Rng};
use std::io::BufRead;
use std::net::SocketAddr;
use std::process::Command;
use std::time::Duration;
use std::env;
use tokio::io::{split, AsyncBufReadExt, AsyncWriteExt, BufReader, ReadHalf, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::time::sleep;
use tokio_stream::wrappers::LinesStream;

struct Coordinator {
    queue: Vec<Addr<CoordinatorClient>>,
    current: Option<Addr<CoordinatorClient>>
}

#[derive(Message)]
#[rtype(result = "()")]
struct Acquire(Addr<CoordinatorClient>);

#[derive(Message)]
#[rtype(result = "()")]
struct Release(Addr<CoordinatorClient>);

#[derive(Message)]
#[rtype(result = "()")]
struct Acquired();

impl Actor for Coordinator {
    type Context = Context<Self>;
}

impl Handler<Acquire> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Acquire, ctx: &mut Self::Context) -> Self::Result {
        let sender = msg.0;
        if self.current.is_none() {
            self.current = Some(sender.clone());
            sender.try_send(Acquired()).expect("can't notify acquired");
        } else {
            self.queue.push(sender);
        }
    }
}

impl Handler<Release> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Release, ctx: &mut Self::Context) -> Self::Result {
        if self.current.is_none() || !self.current.clone().unwrap().eq(&msg.0) {
            println!("[COORDINATOR] intenta liberar sin adquirir")
        } else {
            self.current = self.queue.pop();
            if self.current.is_some() {
                self.current.clone().unwrap().try_send(Acquired()).expect("can't acquire");
            }
        }
    }
}

struct CoordinatorClient {
    id: String,
    coordinator: Addr<Coordinator>,
    addr: SocketAddr,
    write: Option<WriteHalf<tokio::net::TcpStream>>,
}

impl Actor for CoordinatorClient {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for CoordinatorClient {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        match read {
            Ok(buffer) => {
                match buffer.as_str() {
                    "acquire" => {
                        println!("[COORDINATOR] pide lock {}", self.id);
                        // TODO: send ACK
                        self.coordinator.try_send(Acquire(ctx.address())).expect("can't acquire");
                    }
                    "release" => {
                        println!("[COORDINATOR] libera lock {}", self.id);
                        self.coordinator.try_send(Release(ctx.address())).expect("can't release");
                    }
                    _ => {
                        println!("[COORDINATOR] ERROR: mensaje desconocido de {}", self.id);
                    }
                }
            }
            Err(_) => {
                self.finished(ctx);
            }
        }
    }
    fn finished(&mut self, ctx: &mut Self::Context) {
        self.coordinator.try_send(Release(ctx.address())).expect("can't release");
        ctx.stop();
    }
}

#[async_handler]
impl Handler<Acquired> for CoordinatorClient {
    type Result = ();

    async fn handle(&mut self, msg: Acquired, ctx: &mut Self::Context) -> Self::Result {
        let mut write = self.write.take()
            .expect("non atomic!?");

        let ret_write = async move {
            write
                .write_all("acquired\n".as_bytes()).await
                .expect("can't send");
            write
        }.await;

        self.write = Some(ret_write);

    }
}


struct Client {
    id: String
}

struct DistMutex {
    writer: WriteHalf<TcpStream>,
    reader: BufReader<ReadHalf<TcpStream>>,
}

impl DistMutex {
    async fn new(id: u32) -> DistMutex {
        let mut stream = TcpStream::connect("127.0.0.1:12345").await.expect("unable to connect");
        let (reader, writer) = split(stream);
        let mut ret = DistMutex {
            writer,
            reader: BufReader::new(reader)
        };

        ret.writer.write_all((id.to_string() + "\n").as_bytes()).await.expect("failed handshake");

        ret
    }

    async fn acquire(&mut self) {
        self.writer.write_all("acquire\n".as_bytes()).await.expect("unable to request acquire");

        let mut buffer = String::new();
        self.reader.read_line(&mut buffer).await.expect("failed to await response");
    }

    async fn release(&mut self) {
        self.writer.write_all("release\n".as_bytes()).await.expect("unable to request release");
    }
}

const CLIENTS: u32 = 3;

#[actix_rt::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        // procesos independientes. se ejecutan aca para compartir la consola y que sea mas
        // sencillo demostrar.

        for id in 0..CLIENTS {
            Command::new("cargo")
                .arg("run")
                .arg("--example")
                .arg("centralizedmutex")
                .arg("--")
                .arg(id.to_string())
                .spawn()
                .expect("failed to start child");
        }

        coordinator().await
    } else {
        let id = args[1].parse().expect("can't parse id");
        println!("soy proceso {}", id);
        client(id).await
    }
}

async fn coordinator() {
    let listener = TcpListener::bind("127.0.0.1:12345").await.expect("unable to bind");

    println!("[COORDINATOR] Esperando conexiones!");

    let coordinator = Coordinator { queue: vec!(), current: None }.start();

    while let Ok((stream, addr)) = listener.accept().await {
        let (read, write_half) = split(stream);
        let mut reader = BufReader::new(read);

        let mut id = String::new();
        reader.read_line(&mut id).await.expect("unable to read incoming connection id");
        id = id.replace("\n", "");

        println!("[COORDINATOR] Cliente conectado {}", id);

        CoordinatorClient::create(|ctx| {
            CoordinatorClient::add_stream(LinesStream::new(reader.lines()), ctx);
            let write = Some(write_half);
            CoordinatorClient { id, coordinator: coordinator.clone(), addr, write }
        });
    }
}

async fn client(id: u32) {

    let mut mutex = DistMutex::new(id).await;
    println!("[{}] conectado", id);

    let mut count = 0;

    loop {
        println!("[{}] durmiendo", id);
        sleep(Duration::from_millis(thread_rng().gen_range(1000u64..3000))).await;
        println!("[{}] pidiendo lock", id);

        mutex.acquire().await;
        println!("[{}] tengo el lock", id);
        sleep(Duration::from_millis(thread_rng().gen_range(1000u64..3000))).await;

        count += 1;
        if count > 2 {
            break;
        }

        println!("[{}] libero el lock", id);
        mutex.release().await;
    }

    println!("[{}] salí", id);
}