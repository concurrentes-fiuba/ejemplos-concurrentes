use std::net::SocketAddr;

use actix::{Actor, ActorFutureExt, Addr, Context, Handler, Message, StreamHandler, WrapFuture};
use actix_async_handler::async_handler;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::LinesStream;

struct TcpSender {
    write: Option<WriteHalf<TcpStream>>,
}

#[derive(Message)]
#[rtype(result = "()")]
struct TcpMessage(String);

impl Actor for TcpSender {
    type Context = Context<Self>;
}

#[async_handler]
impl Handler<TcpMessage> for TcpSender {
    type Result = ();

    async fn handle(&mut self, msg: TcpMessage, ctx: &mut Self::Context) -> Self::Result {
        let mut write = self.write.take()
            .expect("No debería poder llegar otro mensaje antes de que vuelva por usar AtomicResponse");

        let ret_write = async move {
            write
                .write_all(msg.0.as_bytes()).await
                .expect("should have sent");
            write
        }.await;

        self.write = Some(ret_write);

    }
}


struct HelloServer {
    addr: SocketAddr,
    tcp_sender: Addr<TcpSender>,
}

impl Actor for HelloServer {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for HelloServer {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            println!("[{:?}] Hello {}", self.addr, line);
            self.tcp_sender.try_send(TcpMessage(format!("Hello {}\n", line))).expect("mailbox full?");
        } else {
            println!("[{:?}] Failed to read line {:?}", self.addr, read);
        }
    }
}

#[actix_rt::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();

    println!("Esperando conexiones!");

    while let Ok((stream, addr)) = listener.accept().await {
        println!("[{:?}] Cliente conectado", addr);

        HelloServer::create(|ctx| {
            let (read, write_half) = split(stream);
            HelloServer::add_stream(LinesStream::new(BufReader::new(read).lines()), ctx);
            let write = Some(write_half);
            let tcp_sender = TcpSender { write }.start();
            HelloServer { addr, tcp_sender }
        });
    }
}