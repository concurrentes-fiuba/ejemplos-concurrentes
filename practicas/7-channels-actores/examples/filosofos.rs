extern crate actix;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use actix::{Actor, ActorFutureExt, Addr, AsyncContext, Context, Handler, Message, Recipient, System, WrapFuture};
use actix::clock::sleep;
use actix::dev::fut::future::Map;
use actix::fut::result;
use actix_async_handler::async_handler;
use rand::{Rng, thread_rng};

const N: usize = 5;

type Neighbours = HashMap<ChopstickId, Addr<Philosopher>>;

#[derive(Message)]
#[rtype(result = "()")]
struct SetNeighbours(Neighbours);

#[derive(Message)]
#[rtype(result = "()")]
struct Think;

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
struct TryToEat;

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

impl Actor for Philosopher {
    type Context = Context<Self>;
}

impl Handler<SetNeighbours> for Philosopher {
    type Result = ();

    fn handle(&mut self, msg: SetNeighbours, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi a mis vecinos", self.id);
        self.neighbours = msg.0;
        ctx.address().try_send(Think).unwrap();
    }
}

#[async_handler]
impl Handler<Think> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: Think, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] pensando", self.id);
        sleep(Duration::from_millis(thread_rng().gen_range(2000, 5000))).await;
        ctx.address().try_send(Hungry).unwrap();
    }
}

#[async_handler]
impl Handler<TryToEat> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: TryToEat, ctx: &mut Context<Self>) -> Self::Result {
        if self.chopsticks.iter().all(|(_id, state)| *state != ChopstickState::DontHave) { // si los tengo todos
            println!("[{}] comiendo", self.id);
            sleep(Duration::from_millis(thread_rng().gen_range(2000, 5000))).await;
            ctx.address().try_send(EatingDone).unwrap();
        } else {
            println!("[{}] aun no puedo comer", self.id);
        }
    }
}

#[async_handler]
impl Handler<Hungry> for Philosopher {
    type Result = ();

    async fn handle(&mut self, _msg: Hungry, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] por comer", self.id);
        for (chopstick_id, state) in self.chopsticks.iter() {
            if *state == ChopstickState::DontHave {
                println!("[{}] pido palito {}", self.id, chopstick_id.0);
                self.neighbours.get(chopstick_id).unwrap().try_send(ChopstickRequest(*chopstick_id)).unwrap();
            }
        }

        ctx.address().try_send(TryToEat).unwrap();
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
                self.neighbours.get(&chopstick).unwrap().try_send(ChopstickResponse(msg.0)).unwrap();
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

#[async_handler]
impl Handler<ChopstickResponse> for Philosopher {
    type Result = ();

    async fn handle(&mut self, msg: ChopstickResponse, ctx: &mut Context<Self>) -> Self::Result {
        println!("[{}] recibi palito {}", self.id, msg.0.0);
        self.chopsticks.insert(msg.0,ChopstickState::Clean);
        ctx.address().try_send(TryToEat).unwrap();
    }
}

#[async_handler]
impl Handler<EatingDone> for Philosopher {
    type Result = ();

    async fn handle(&mut self, _msg: EatingDone, ctx: &mut Context<Self>) -> Self::Result {
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
        ctx.address().try_send(Think).unwrap();
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