use std::net::SocketAddr;

use actix::{Actor, ActorFutureExt, Context, ContextFutureSpawner, StreamHandler, WrapFuture};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio_stream::wrappers::LinesStream;

struct HelloServer {
    write: Option<WriteHalf<TcpStream>>,
    addr: SocketAddr,
}

impl HelloServer {

    fn send(&mut self, msg: String, ctx: &mut <HelloServer as Actor>::Context) {
        let mut write = self.write.take()
            .expect("No debería poder llegar otro mensaje antes de que vuelva por usar ctx.wait");
        async move {
            write
                .write_all(msg.as_bytes()).await
                .expect("should have sent");
            write
        }.into_actor(self)
            .map(|write, this, _| this.write = Some(write))
            .wait(ctx)
    }
}
impl Actor for HelloServer {
    type Context = Context<Self>;
}

impl StreamHandler<Result<String, std::io::Error>> for HelloServer {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            println!("[{:?}] Hello {}", self.addr, line);
            self.send(format!("Hello {}\n", line), ctx);
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
            HelloServer { addr, write }
        });
    }
}