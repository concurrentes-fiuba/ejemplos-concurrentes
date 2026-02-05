extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, AsyncContext, Recipient, ActorFutureExt};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;
use actix_async_handler::async_handler;

const WORKERS:usize = 5;

#[derive(Message)]
#[rtype(result = "()")]
struct Process {
    amount: f64,
    sender: Recipient<Result>
}

#[derive(Message)]
#[rtype(result = "()")]
struct Result(usize, f64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Epoch(f64);

struct Coordinator {
    signal: f64,
    workers: Vec<Recipient<Process>>,
    results: HashSet<usize>
}

struct Worker {
    id: usize,
}

impl Actor for Coordinator {
    type Context = Context<Self>;
}

impl Actor for Worker {
    type Context = Context<Self>;
}

impl Handler<Epoch> for Coordinator {
    type Result = ();

    fn handle(&mut self, _msg: Epoch, _ctx: &mut Context<Self>) -> Self::Result {
        let signal = _msg.0;
        let signal_local = signal / self.workers.len() as f64;
        self.signal = 0.;
        self.results.clear();

        println!("[COORDINADOR] empieza con señal {}", signal);

        for worker in self.workers.iter() {
            worker.try_send(Process { amount: signal_local, sender: _ctx.address().recipient()}).unwrap();
        }


    }
}

impl Handler<Result> for Coordinator {
    type Result = ();

    fn handle(&mut self, msg: Result, _ctx: &mut Context<Self>) -> Self::Result {

        println!("[COORDINADOR] recibí resultado de worker {}", msg.0);

        if !self.results.contains(&msg.0) {
            self.signal += msg.1;
            self.results.insert(msg.0);

            if self.results.len() == self.workers.len() {
                println!("[COORDINADOR] fin de la epoch, resultado final {}", self.signal);
                _ctx.address().try_send(Epoch(self.signal)).unwrap();
            }
        }

    }
}

#[async_handler]
impl Handler<Process> for Worker {
    type Result = ();

    fn handle(&mut self, msg: Process, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[WORKER {}] recibo {}", self.id, msg.amount);
        sleep(Duration::from_millis(thread_rng().gen_range(500, 1500))).await;
        let resultado = msg.amount * thread_rng().gen_range(0., 1.);
        println!("[WORKER {}] devuelvo {}", self.id, resultado);
        msg.sender.try_send(Result(self.id, resultado)).unwrap();
    }
}

fn main() {
    let system = System::new();
    system.block_on(async {
        let mut workers = vec!();

        for id in 0..WORKERS {
            workers.push(Worker { id }.start().recipient())
        }


        Coordinator { signal: 0.0, workers, results: HashSet::with_capacity(WORKERS) }.start()
            .do_send(Epoch(1000.0));
    });

    system.run().unwrap();

}