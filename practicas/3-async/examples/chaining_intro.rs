use std::time::Duration;
use async_std::task;

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
    let hello1 = hello();
    let world1 = world();
    hello1.await + world1.await.as_str()
}

fn main() {
    println!("{}", task::block_on(async_main()))
}