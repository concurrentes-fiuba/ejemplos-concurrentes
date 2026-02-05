extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, Addr, AsyncContext, WrapFuture, Recipient, ActorFutureExt};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;
use actix_async_handler::async_handler;

const WORKERS:usize = 5;

#[derive(Message)]
#[rtype(result = "()")]
struct ProcessSignal {
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
    workers: Vec<Recipient<ProcessSignal>>,
    results: HashSet<usize>,
    next_epoch_recipient: Option<Recipient<Epoch>>
}

struct Worker {
    id: usize,
}

#[cfg(not(test))]
fn gen_range(l: f64, h: f64) -> f64 {
    thread_rng().gen_range(l, h)
}

#[cfg(test)]
use tests::mock_gen_range as gen_range;

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
            worker.try_send(ProcessSignal { amount: signal_local, sender: _ctx.address().recipient()}).unwrap();
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
                self.next_epoch_recipient.as_ref().unwrap_or(&_ctx.address().recipient())
                    .try_send(Epoch(self.signal)).unwrap();
            }
        }

    }
}

#[async_handler]
impl Handler<ProcessSignal> for Worker {
    type Result = ();

    fn handle(&mut self, msg: ProcessSignal, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[WORKER {}] recibo {}", self.id, msg.amount);
        sleep(Duration::from_millis(thread_rng().gen_range(500, 1500))).await;
        let resultado = msg.amount * thread_rng().gen_range(0., 1.);
        println!("[WORKER {}] devuelvo {}", self.id, resultado);
        msg.sender.try_send(Result(self.id, resultado)).unwrap();
    }
}

async fn setup(next_epoch_recipient: Option<Recipient<Epoch>>) -> Addr<Coordinator> {

    let workers = (0..WORKERS).into_iter()
        .map(|id| Worker { id }.start().recipient())
        .collect::<Vec<Recipient<ProcessSignal>>>();

    let addr = Coordinator { signal: 0., workers, results: HashSet::with_capacity(WORKERS), next_epoch_recipient }.start();

    addr.send(Epoch(100.)).await;

    addr

}

fn main() {
    let system = System::new();
    system.block_on(setup(None));

    system.run().unwrap();

}

#[cfg(test)]
mod tests {
    use super::*;
    use ntest::timeout;
    use ntest::assert_about_eq;
    use actix::actors::mocker::Mocker;
    use std::sync::{Mutex, Arc};
    use std::cell::RefCell;

    pub(crate) fn mock_gen_range(l: f64, h: f64) -> f64 {
        (l + h) * 0.7
    }

    #[actix_rt::test]
    async fn test_scenario_coordinador_envia() {

        // Given coordinador y 3 workers

        let (tx, mut rx) = tokio::sync::mpsc::channel(3);
        let mut workers = vec!();


        for id in 0..3 {
            let mut my_tx = tx.clone();
            workers.push(Mocker::<ProcessSignal>::mock(Box::new(move |_msg, _ctx| {
                println!("[{}] recibi {:?}", id, _msg);
                my_tx.try_send(_msg);
                Box::new(Some(()))
            })).start().recipient());
        }

        let sut = Coordinator {
            signal: 0.,
            workers,
            results: HashSet::new(),
            next_epoch_recipient: None
        }.start();

        // When inicia proces

        sut.send(Epoch(900.)).await;

        // Then cada worker recibió 1/3 de la señal.

        for _ in 0..3 {
            assert_eq!(300., rx.recv().await.unwrap().downcast_ref::<ProcessSignal>().unwrap().amount)
        }

    }

    #[actix_rt::test]
    async fn test_scenario_coordinador_envia_promesa() {

        // Given coordinador y 3 workers

        let mut promises = vec!();
        let mut workers = vec!();


        for _ in 0..3 {
            let (tx, rx) = futures_channel::oneshot::channel();
            let mut tx_once = Some(tx);
            promises.push(rx);
            workers.push(Mocker::<ProcessSignal>::mock(Box::new(move |_msg, _ctx| {
                tx_once.take().expect("should be called just once").send(_msg);
                Box::new(Some(()))
            })).start().recipient());
        }

        let sut = Coordinator {
            signal: 0.,
            workers,
            results: HashSet::new(),
            next_epoch_recipient: None
        }.start();

        // When inicia proceso

        sut.send(Epoch(900.)).await;

        // Then cada worker recibió 1/3 de la señal.

        for mut p in promises {
            assert_eq!(300., p.await.unwrap().downcast_ref::<ProcessSignal>().unwrap().amount)
        }

    }


    #[test]
    #[timeout(5000)]
    fn test_integration() {

        let system = System::new();
        system.block_on(async {
            let retorno = Mocker::<Epoch>::mock(Box::new(move |_msg, _ctx| {
                System::current().stop();
                assert_about_eq!(_msg.downcast_ref::<Epoch>().unwrap().0, 1400.);
                Box::new(Some(()))
            })).start();

            setup(Some(retorno.recipient())).await;
        });

        system.run().unwrap();

    }

}