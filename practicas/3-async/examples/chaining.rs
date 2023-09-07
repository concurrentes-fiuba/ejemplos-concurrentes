use std::time::Duration;

use async_std::task;
use futures::join;

async fn hello() -> String {
    println!("before hello");
    task::sleep(Duration::from_secs(2)).await;
    println!("after hello");
    String::from("Hello")
}

async fn world() -> String {
    println!("before world");
    task::sleep(Duration::from_secs(1)).await;
    println!("after world");
    String::from(" World!")
}

async fn async_main() -> String {
    let (h, w) = join!(hello(), world());
    h + w.as_str()
}

fn main() {
    println!("{}", task::block_on(async_main()))
}