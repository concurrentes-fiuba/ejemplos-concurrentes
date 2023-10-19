use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:12345").unwrap();

    println!("Esperando conexiones!");

    for opt_stream in listener.incoming() {
        if let Ok(mut stream) = opt_stream {
            let addr = stream.peer_addr().unwrap();
            println!("[{:?}] Cliente conectado", addr);
            thread::spawn(move || receiver(stream, addr));
        } else {
            println!("Error al conectar")
        }

    }
}

fn receiver(mut stream: TcpStream, addr: SocketAddr) {
    let mut reader = BufReader::new(stream.try_clone().expect(""));
    loop {
        let mut buffer = String::new();
        if let Ok(read) = reader.read_line(&mut buffer) {
            if read > 0 {
                // Aca proceso el mensaje
                println!("[{:?}] Hello {}", addr, buffer);
                stream.write_all(format!("Hello {}", buffer).as_bytes())
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