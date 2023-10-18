extern crate actix;

use std::{thread, io};
use std::time::{Duration, SystemTime};

use actix::{Actor, ActorFutureExt, Context, Handler, Message, ResponseActFuture, SyncArbiter, System, WrapFuture, SyncContext};
use actix::clock::sleep;
use std::io::Read;

#[derive(Message)]
#[rtype(result = "()")]
struct Sleep(u64);

struct Sleepyhead {
    id: usize
}

impl Actor for Sleepyhead {
    type Context = Context<Self>; //SyncContext<Self>;
}

impl Handler<Sleep> for Sleepyhead {
    type Result = ();//ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Sleep, _ctx: &mut <Sleepyhead as Actor>::Context) -> Self::Result  {
        println!("[{}] durmiendo por {}", self.id, msg.0);
        thread::sleep(Duration::from_secs(msg.0 * 10));
        println!("[{}] desperté de {}", self.id, msg.0);
        // Box::pin(sleep(Duration::from_secs(msg.0))
        //     .into_actor(self)
        //     .map(move |_result, me, _ctx| {
        //         println!("[{}] desperté de {}", me.id, msg.0);
        //     }))
    }
}


#[actix_rt::main]
async fn main() {
    console_subscriber::init();

    println!("Enter para empezar");
    io::stdin().read(&mut [0u8]).unwrap();

    let addr = Sleepyhead { id: 1}.start(); // <- start actor and get its address
    // let addr = SyncArbiter::start(1, || Sleepyhead { id: 1 });
    // let addr = SyncArbiter::start(2, || Sleepyhead { id: 1 });

    let other = Sleepyhead { id: 2 }.start(); // <- start actor and get its address
    // let other = SyncArbiter::start(1, || Sleepyhead { id: 2 });

    let now = SystemTime::now();

    addr.try_send(Sleep(3)).unwrap();
    println!("mandé 3 al 1");

    other.try_send(Sleep(2)).unwrap();
    println!("mandé 2 al 2");

    // wait for response
    addr.send(Sleep(2)).await.unwrap();

    println!("terminé. tardé {}", now.elapsed().unwrap().as_secs());


    println!("Enter para terminar");
    io::stdin().read(&mut [0u8]).unwrap();

    System::current().stop();
}