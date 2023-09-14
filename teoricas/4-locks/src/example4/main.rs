
use std::thread;
use std::fs::File;
use std::io::SeekFrom;
use std::io::prelude::*;


fn main() -> std::io::Result<()> {

    let mut file = File::create("file.txt")?;
    
    thread::Builder::new().name("thread-A".to_string()).spawn(move || {
        for _ in 1..1000 {
            let file_size = file.metadata().unwrap().len();
            file.seek(SeekFrom::Start(file_size)).unwrap();
            file.write_all(b"A").unwrap();
        }

    }).unwrap().join().ok();

    thread::Builder::new().name("thread-B".to_string()).spawn(move || {
        for _ in 1..1000 {
            let file_size = file.metadata().unwrap().len();
            file.seek(SeekFrom::Start(file_size)).unwrap();
            file.write_all(b"B").unwrap();
        }

    }).unwrap().join().ok();



    Ok(())    
}