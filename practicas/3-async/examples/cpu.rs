use std::future::Future;
use std::time::SystemTime;

use async_std::task;
use futures::future::join;

async fn fibonacci(n: u32) -> u32 {
    if n <= 1 {
        n
    } else {
        Box::pin(fibonacci(n - 1)).await + Box::pin(fibonacci(n - 2)).await
    }
}

async fn measure<Fut: Future>(f: Fut) {
    let start = SystemTime::now();
    f.await;
    println!("{:?}", SystemTime::now().duration_since(start));
}

fn main() {
    task::block_on(async {
        measure(fibonacci(34)).await;
        measure(fibonacci(36)).await;
        measure(join(fibonacci(34), fibonacci(36))).await;
    });
}