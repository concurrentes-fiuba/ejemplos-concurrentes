extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, Addr, AsyncContext, WrapFuture, Recipient, ActorFutureExt, ResponseActFuture};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;

const INVERSORES:usize = 5;

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Invertir {
    amount: f64,
    sender: Recipient<ResultadoInversion>
}

#[derive(Message)]
#[rtype(result = "()")]
struct ResultadoInversion(usize, f64);

#[derive(Message, Debug)]
#[rtype(result = "()")]
struct Semana(f64);

struct Banquero {
    plata: f64,
    inversores: Vec<Recipient<Invertir>>,
    devoluciones: HashSet<usize>,
    retorno: Option<Recipient<Semana>>
}

struct Inversor {
    id: usize,
}

#[cfg(not(test))]
fn gen_range(l: f64, h: f64) -> f64 {
    thread_rng().gen_range(l, h)
}

#[cfg(test)]
use tests::mock_gen_range as gen_range;

impl Actor for Banquero {
    type Context = Context<Self>;
}

impl Actor for Inversor {
    type Context = Context<Self>;
}

impl Handler<Semana> for Banquero {
    type Result = ();

    fn handle(&mut self, _msg: Semana, _ctx: &mut Context<Self>) -> Self::Result {
        let plata = _msg.0;
        let amount = plata / self.inversores.len() as f64;
        self.plata = 0.;
        self.devoluciones.clear();

        println!("[BANQUERO] empieza la semana con {}", plata);

        for inversor in self.inversores.iter() {
            inversor.try_send(Invertir { amount, sender: _ctx.address().recipient()}).unwrap();
        }


    }
}

impl Handler<ResultadoInversion> for Banquero {
    type Result = ();

    fn handle(&mut self, msg: ResultadoInversion, _ctx: &mut Context<Self>) -> Self::Result {

        println!("[BANQUERO] recibí resultado de la inversion {}", msg.0);

        self.plata += msg.1;
        self.devoluciones.insert(msg.0);

        if self.devoluciones.len() == self.inversores.len() {
            println!("[BANQUERO] fin de la semana, resultado final {}", self.plata);
            self.retorno.as_ref().unwrap_or(&_ctx.address().recipient())
                .try_send(Semana(self.plata)).unwrap();
        }

    }
}

impl Handler<Invertir> for Inversor {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Invertir, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[INV {}] recibo inversion por {}", self.id, msg.amount);
        Box::pin(sleep(Duration::from_millis(thread_rng().gen_range(500, 1500)))
            .into_actor(self)
            .map(move |_result, me, _ctx| {
                let resultado = msg.amount * gen_range(0.5, 1.5);
                println!("[INV {}] devuelvo {}", me.id, resultado);
                msg.sender.try_send(ResultadoInversion(me.id, resultado)).unwrap();
            }))
    }
}

async fn setup(retorno: Option<Recipient<Semana>>) -> Addr<Banquero> {

    let inversores = (0..INVERSORES).into_iter()
        .map(|id| Inversor { id }.start().recipient())
        .collect::<Vec<Recipient<Invertir>>>();

    let addr = Banquero { plata: 0., inversores, devoluciones: HashSet::with_capacity(INVERSORES), retorno }.start();

    addr.send(Semana(1000.)).await;

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
    async fn test_scenario_banquero_envia() {

        // Given banquero y 3 inversores

        let (tx, mut rx) = tokio::sync::mpsc::channel(3);
        let mut inversores = vec!();


        for id in 0..3 {
            let mut my_tx = tx.clone();
            inversores.push(Mocker::<Invertir>::mock(Box::new(move |_msg, _ctx| {
                println!("[{}] recibi {:?}", id, _msg);
                my_tx.try_send(_msg);
                Box::new(Some(()))
            })).start().recipient());
        }

        let sut = Banquero {
            plata: 0.,
            inversores,
            devoluciones: HashSet::new(),
            retorno: None
        }.start();

        // When inicia semana

        sut.send(Semana(900.)).await;

        // Then cada inversor recibió 1/3 de la plata.

        for _ in 0..3 {
            assert_eq!(300., rx.recv().await.unwrap().downcast_ref::<Invertir>().unwrap().amount)
        }

    }

    #[actix_rt::test]
    async fn test_scenario_banquero_envia_promesa() {

        // Given banquero y 3 inversores

        let mut promises = vec!();
        let mut inversores = vec!();


        for _ in 0..3 {
            let (tx, rx) = futures_channel::oneshot::channel();
            let mut tx_once = Some(tx);
            promises.push(rx);
            inversores.push(Mocker::<Invertir>::mock(Box::new(move |_msg, _ctx| {
                tx_once.take().expect("should be called just once").send(_msg);
                Box::new(Some(()))
            })).start().recipient());
        }

        let sut = Banquero {
            plata: 0.,
            inversores,
            devoluciones: HashSet::new(),
            retorno: None
        }.start();

        // When inicia semana

        sut.send(Semana(900.)).await;

        // Then cada inversor recibió 1/3 de la plata.

        for mut p in promises {
            assert_eq!(300., p.await.unwrap().downcast_ref::<Invertir>().unwrap().amount)
        }

    }


    #[test]
    #[timeout(5000)]
    fn test_integration() {

        let system = System::new();
        system.block_on(async {
            let retorno = Mocker::<Semana>::mock(Box::new(move |_msg, _ctx| {
                System::current().stop();
                assert_about_eq!(_msg.downcast_ref::<Semana>().unwrap().0, 1400.);
                Box::new(Some(()))
            })).start();

            setup(Some(retorno.recipient())).await;
        });

        system.run().unwrap();

    }

}
