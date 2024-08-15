extern crate actix;

use std::time::Duration;

use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, System, WrapFuture};
use actix_async_handler::async_handler;
use rand::{Rng, thread_rng};
use tokio::time::sleep;

#[derive(Message)]
#[rtype(result = "i32")]
struct Add(i32);

#[derive(Message)]
#[rtype(result = "i32")]
struct Sub(i32);

#[derive(Message)]
#[rtype(result = "i32")]
struct Get();

struct Calc {
    current: i32,
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

impl Handler<Get> for Calc {
    type Result = i32;

    fn handle(&mut self, _msg: Get, _ctx: &mut Context<Self>) -> Self::Result {
        self.current
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Produce();

struct Producer {
    id: i32,
    calc: Addr<Calc>,
}

impl Actor for Producer {
    type Context = Context<Self>;
}

#[async_handler]
impl Handler<Produce> for Producer {
    type Result = ();

    fn handle(&mut self, msg: Produce, ctx: &mut Context<Self>) -> Self::Result {
        sleep(Duration::from_millis(thread_rng().gen_range(500, 1500))).await;
        let amount = thread_rng().gen_range(-100, 100);
        println!("[Producer {}] - sending {}", self.id, amount);
        self.calc.try_send(Add(amount)).unwrap();
        ctx.address().try_send(Produce()).unwrap();
    }
}

#[derive(Message)]
#[rtype(result = "()")]
struct Report();

struct Reporter {
    id: i32,
    calc: Addr<Calc>,
}

impl Actor for Reporter {
    type Context = Context<Self>;
}

#[async_handler]
impl Handler<Report> for Reporter {
    type Result = ();

    async fn handle(&mut self, msg: Report, ctx: &mut Context<Self>) -> Self::Result {
        sleep(Duration::from_millis(thread_rng().gen_range(1000, 3000))).await;
        let result = self.calc.send(Get()).await;
        println!("[Reporter {}] - {}", self.id, result.expect("no vino!"));
        ctx.address().try_send(Report()).unwrap();
    }
}

fn main() {
    let system = System::new();
    system.block_on(async {
        let calc = Calc { current: 0 }.start();

        let producer1 = Producer { id: 1, calc: calc.clone() }.start();
        producer1.try_send(Produce()).unwrap();
        let producer2 = Producer { id: 2, calc: calc.clone() }.start();
        producer2.try_send(Produce()).unwrap();
        let producer3 = Producer { id: 3, calc: calc.clone() }.start();
        producer3.try_send(Produce()).unwrap();

        let reporter1 = Reporter { id: 1, calc: calc.clone() }.start();
        reporter1.try_send(Report()).unwrap();
        let reporter2 = Reporter { id: 2, calc: calc.clone() }.start();
        reporter2.try_send(Report()).unwrap();
    });

    system.run().unwrap();
}