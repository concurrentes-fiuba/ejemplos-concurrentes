extern crate actix;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient, ResponseActFuture, System, WrapFuture};
use actix::clock::sleep;
use actix::dev::fut::future::Map;
use actix::fut::result;
use rand::{Rng, thread_rng};

const N: usize = 5;

type Neighbours = HashMap<ChopstickId, Addr<Philosopher>>;

#[derive(Message)]
#[rtype(result = "()")]
struct SetNeighbours(Neighbours);

#[derive(Message)]
#[rtype(result = "()")]
struct Hungry;

#[derive(Message)]
#[rtype(result = "()")]
struct ChopstickRequest(ChopstickId);

#[derive(Message)]
#[rtype(result = "()")]
struct ChopstickResponse(ChopstickId);

#[derive(Message)]
#[rtype(result = "()")]
struct EatingDone;

#[derive(PartialEq)]
enum ChopstickState {
    DontHave,
    Dirty,
    Clean,
    Requested
}

#[derive(PartialEq, Eq, Hash, Copy, Clone)]
struct ChopstickId(usize);

struct Philosopher {
    id: usize,
    chopsticks: HashMap<ChopstickId, ChopstickState>,
    neighbours: Neighbours
}

impl Philosopher {
    fn sleep<WakeupMessage>(&self, msg: WakeupMessage) -> ResponseActFuture<Self, ()>
        where
            Self: Handler<WakeupMessage>,
            WakeupMessage: Message + Send + 'static,
            WakeupMessage::Result: Send,
    {
        println!("[{}] pensando", self.id);
        Box::pin(sleep(Duration::from_millis(thread_rng().gen_range(2000, 5000)))
            .into_actor(self)
            .map(move |_result, _me, ctx| {
                ctx.address().try_send(msg).unwrap();
            }))
    }

    fn eat_if_ready(&self) -> ResponseActFuture<Self, ()> {
        if self.chopsticks.iter().all(|(_id, state)| *state != ChopstickState::DontHave) { // si los tengo todos
            println!("[{}] comiendo", self.id);
            self.sleep(EatingDone)
        } else {
            println!("[{}] aun no puedo comer", self.id);
            Box::pin(std::future::ready(()).into_actor(self))
        }
    }

}

impl Actor for Philosopher {
    type Context = Context<Self>;
}

impl Handler<SetNeighbours> for Philosopher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: SetNeighbours, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi a mis vecinos", self.id);
        self.neighbours = msg.0;
        self.sleep(Hungry)
    }
}

impl Handler<Hungry> for Philosopher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: Hungry, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] por comer", self.id);
        for (chopstick_id, state) in self.chopsticks.iter() {
            if *state == ChopstickState::DontHave {
                println!("[{}] pido palito {}", self.id, chopstick_id.0);
                self.neighbours.get(chopstick_id).unwrap().try_send(ChopstickRequest(*chopstick_id)).unwrap();
            }
        }

        self.eat_if_ready()
    }
}


impl Handler<ChopstickRequest> for Philosopher {
    type Result = ();

    fn handle(&mut self, msg: ChopstickRequest, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] me piden palito {}", self.id, msg.0.0);
        let chopstick = msg.0;
        let chopstick_state = &self.chopsticks.get(&chopstick);
        match chopstick_state {
            Some(ChopstickState::Dirty) => {
                println!("[{}] se lo doy ahora", self.id);
                self.neighbours.get(&chopstick).unwrap().try_send(ChopstickResponse(msg.0));
                self.chopsticks.insert(chopstick, ChopstickState::DontHave);
            },
            Some(ChopstickState::Clean) => {
                println!("[{}] se lo doy cuando termine", self.id);
                self.chopsticks.insert(chopstick, ChopstickState::Requested);
            },
            _ => {
                println!("[{}] no deberia pasar", self.id);
            }
        }
    }
}

impl Handler<ChopstickResponse> for Philosopher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: ChopstickResponse, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi palito {}", self.id, msg.0.0);
        self.chopsticks.insert(msg.0,ChopstickState::Clean);
        self.eat_if_ready()
    }
}

impl Handler<EatingDone> for Philosopher {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, _msg: EatingDone, _ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] terminé de comer", self.id);
        for (chopstick, mut state) in self.chopsticks.iter_mut() {
            if *state == ChopstickState::Requested {
                println!("[{}] entrego palito {}", self.id, chopstick.0);
                self.neighbours.get(chopstick).unwrap().try_send(ChopstickResponse(*chopstick)).unwrap();
                *state = ChopstickState::DontHave
            } else {
                println!("[{}] marco como sucio palito {}", self.id, chopstick.0);
                *state = ChopstickState::Dirty
            }
        }
        self.sleep(Hungry)
    }
}

fn main() {
    let system = System::new();
    system.block_on(async {
        let mut philosophers = vec!();

        for id in 0..N {
            // Deadlock avoidance forcing the initial state
            philosophers.push(Philosopher {
                id,
                chopsticks: HashMap::from([
                    (ChopstickId(id), if id == 0 { ChopstickState::Dirty } else { ChopstickState::DontHave }),
                    (ChopstickId((id + 1) % N), if id == N-1 { ChopstickState::DontHave } else { ChopstickState::Dirty })
                ]),
                neighbours: HashMap::with_capacity(2)
            }.start())
        }


        for id in 0..N {
            let prev = if id == 0 { N - 1 } else { id - 1 };
            let next = (id + 1) % N;
            philosophers[id].try_send(SetNeighbours(HashMap::from([
                (ChopstickId(id), philosophers[prev].clone()),
                (ChopstickId(next), philosophers[next].clone())
            ]))).unwrap();
        }
    });

    system.run().unwrap();
}
