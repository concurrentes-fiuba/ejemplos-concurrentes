use std::net::SocketAddr;

use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, split};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main(){
    let listener = TcpListener::bind("127.0.0.1:12345").await.unwrap();

    println!("Esperando conexiones!");

    loop {
        if let Ok((mut stream, addr)) = listener.accept().await {
            println!("[{:?}] Cliente conectado", addr);

            tokio::spawn(async move {
                receiver(stream, addr).await;
            });
        } else {
            println!("Error al conectar")
        }

    }
}

async fn receiver(mut stream: TcpStream, addr: SocketAddr) {
    let (r, mut w) = split(stream);
    let mut reader = BufReader::new(r);
    loop {
        let mut buffer = String::new();
        if let Ok(read) = reader.read_line(&mut buffer).await {
            if read > 0 {
                println!("[{:?}] Hello {}", addr, buffer);
                w.write_all(format!("Hello {}", buffer).as_bytes()).await
                    .expect("");
            } else {
                println!("[{:?}] Goodbye!", addr);
                break;
            }
        } else {
            println!("[{:?}] Error leyendo socket!", addr);
            break;
        }
    }
}