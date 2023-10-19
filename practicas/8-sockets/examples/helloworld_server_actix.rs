use std::net::SocketAddr;
use std::sync::Arc;

use actix::{Actor, ActorContext, ActorFutureExt, AsyncContext, Context, ContextFutureSpawner, StreamHandler};
use actix::fut::wrap_future;
use tokio::io;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split, WriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_stream::wrappers::LinesStream;

struct HelloServer {
    // write: Option<WriteHalf<TcpStream>>,
    write: Arc<Mutex<WriteHalf<TcpStream>>>,
    addr: SocketAddr,
}

impl Actor for HelloServer {
    type Context = Context<Self>;
}

/*
impl StreamHandler<Result<String, std::io::Error>> for HelloServer {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            println!("[{:?}] Hello {}", self.addr, line);
            let mut write = self.write.take()
                .expect("No debería poder llegar otro mensaje antes de que vuelva por usar ctx.wait");
            wrap_future::<_, Self>(async move {
                write
                    .write_all(format!("Hello {}\n", line).as_bytes()).await
                    .expect("should have sent");
                write
            })
                .map(|write, this, _| this.write = Some(write))
                .wait(ctx);
        } else {
            println!("[{:?}] Failed to read line {:?}", self.addr, read);
        }
    }
}
 */

impl StreamHandler<Result<String, std::io::Error>> for HelloServer {
    fn handle(&mut self, read: Result<String, std::io::Error>, ctx: &mut Self::Context) {
        if let Ok(line) = read {
            println!("[{:?}] Hello {}", self.addr, line);
            let arc = self.write.clone();
            wrap_future::<_, Self>(async move {
                arc.lock().await
                    .write_all(format!("Hello {}\n", line).as_bytes()).await
                    .expect("should have sent")
            }).spawn(ctx);
        } else {
            println!("[{:?}] Failed to read line {:?}", self.addr, read);
        }
    }

    fn finished(&mut self, ctx: &mut Self::Context) {
        println!("[{:?}] desconectado", self.addr);
        ctx.stop();
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
            let write =  Arc::new(Mutex::new(write_half));
            //let write = Some(write_half);
            HelloServer { addr, write }
        });
    }
}