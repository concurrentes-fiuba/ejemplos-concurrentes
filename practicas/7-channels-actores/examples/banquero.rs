extern crate actix;

use std::collections::HashSet;
use actix::{Actor, Context, Handler, System, Message, Addr, AsyncContext, WrapFuture, Recipient, ActorFutureExt, ResponseActFuture};
use rand::{thread_rng, Rng};
use actix::clock::sleep;
use std::time::Duration;

const INVERSORES:usize = 5;

#[derive(Message)]
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
    devoluciones: HashSet<usize>
}

struct Inversor {
    id: usize,
}

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

        if !self.devoluciones.contains(&msg.0) {
            self.plata += msg.1;
            self.devoluciones.insert(msg.0);

            if self.devoluciones.len() == self.inversores.len() {
                println!("[BANQUERO] fin de la semana, resultado final {}", self.plata);
                _ctx.address().try_send(Semana(self.plata)).unwrap();
            }
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
                let resultado = msg.amount * thread_rng().gen_range(0.5, 1.5);
                println!("[INV {}] devuelvo {}", me.id, resultado);
                msg.sender.try_send(ResultadoInversion(me.id, resultado)).unwrap();
            }))
    }
}

fn main() {
    let system = System::new();
    system.block_on(async {
        let mut inversores = vec!();

        for id in 0..INVERSORES {
            inversores.push(Inversor { id }.start().recipient())
        }


        Banquero { plata: 0.0, inversores, devoluciones: HashSet::with_capacity(INVERSORES) }.start()
            .do_send(Semana(1000.0));
    });

    system.run().unwrap();

}