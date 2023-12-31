use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::Duration;
use async_std::task;
use futures_lite::future::FutureExt;

struct HelloFuture {
    timer: Pin<Box<dyn Future<Output=()>>>,
}

fn hello() -> HelloFuture {
    HelloFuture { timer: Box::pin(task::sleep(Duration::from_secs(2))) }
}

impl Future for HelloFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<String> {
        println!("hello poll");
        match self.timer.poll(_cx) {
            Poll::Pending => return Poll::Pending,
            _ => Poll::Ready(String::from("Hello "))
        }
    }
}

struct WorldFuture {
}

impl Future for WorldFuture {
    type Output = String;

    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<String> {
        println!("world poll");
        Poll::Ready(String::from("World!"))
    }
}

// The `Future` type generated by our `async { ... }` block
struct MainFuture {
    hello: HelloFuture,
    hello_result: Poll<String>,
    world: WorldFuture,
    world_result: Poll<String>,
    state: State,
}

// List of states our `async` block can be in
enum State {
    AwaitingHello,
    AwaitingWorld,
    Done,
}

impl Future for MainFuture {
    type Output = String;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        loop {
            match self.state {
                State::AwaitingHello => match self.hello.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(r) => {
                        self.state = State::AwaitingWorld;
                        self.hello_result = Poll::Ready(r)
                    },
                }
                State::AwaitingWorld => match self.world.poll(cx) {
                    Poll::Pending => return Poll::Pending,
                    Poll::Ready(r) => {
                        self.state = State::Done;
                        self.world_result = Poll::Ready(r)
                    },
                }
                State::Done => {
                    if let Poll::Ready(h) = &self.hello_result {
                        if let Poll::Ready(w) = &self.world_result {
                            return Poll::Ready(h.as_str().to_owned() + w.as_str())
                        }
                    }

                },
            }
        }
    }
}

fn async_main() -> MainFuture {
    println!("Started!");
    let future = hello();
    MainFuture {
        state: State::AwaitingHello,
        hello: future,
        hello_result: Poll::Pending,
        world: WorldFuture {},
        world_result: Poll::Pending
    }
}

fn main() {
    println!("{}", task::block_on(async_main()))
}
