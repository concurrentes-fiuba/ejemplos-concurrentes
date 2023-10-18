extern crate actix;

use actix::{Actor, Context, Handler, System, Message};

#[derive(Message)]
#[rtype(result = "i32")]
struct Add(i32);

#[derive(Message)]
#[rtype(result = "i32")]
struct Sub(i32);

struct Calc {
    current: i32
}

impl Actor for Calc {
    type Context = Context<Self>;
}

impl Handler<Add> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Add, _ctx: &mut Context<Self>) -> Self::Result {
        println!("add {}", msg.0);
        self.current += msg.0;
        self.current
    }
}

impl Handler<Sub> for Calc {
    type Result = i32;

    fn handle(&mut self, msg: Sub, _ctx: &mut Context<Self>) -> Self::Result {
        println!("sub {}", msg.0);
        self.current -= msg.0;
        self.current
    }
}

#[actix_rt::main]
async fn main() {
    let addr = Calc { current: 0 }.start(); // <- start actor and get its address

    // fire and forget
    addr.do_send(Add(20));
    println!("do_send done");

    // fire and forget, but check for errors like full mailbox
    addr.try_send(Add(15)).unwrap();
    println!("try_send done");

    // wait for response
    let res = addr.send(Add(5)).await;
    println!("{}", res.unwrap());

    // wait for response
    let res = addr.send(Sub(3)).await;

    println!("{}", res.unwrap());
    System::current().stop();
}
