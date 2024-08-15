extern crate actix;

use std::{thread, io};
use std::time::{Duration, SystemTime};

use actix::{Actor, ActorFutureExt, Context, Handler, Message, SyncArbiter, System, SyncContext};
use std::io::Read;
use actix_async_handler::async_handler;

#[derive(Message)]
#[rtype(result = "()")]
struct Sleep(u64);

struct Sleepyhead {
    id: usize
}

impl Actor for Sleepyhead {
    type Context = Context<Self>; //SyncContext<Self>;
}

#[async_handler]
impl Handler<Sleep> for Sleepyhead {
    type Result = ();

    async fn handle(&mut self, msg: Sleep, _ctx: &mut <Sleepyhead as Actor>::Context) -> Self::Result  {
        println!("[{}] durmiendo por {}", self.id, msg.0);
        tokio::time::sleep(Duration::from_secs(msg.0)).await;
        println!("[{}] desperté de {}", self.id, msg.0);
    }
}

// impl Handler<Sleep> for Sleepyhead {
//     type Result = ();
//
//     fn handle(&mut self, msg: Sleep, _ctx: &mut <Sleepyhead as Actor>::Context) -> Self::Result  {
//         println!("[{}] durmiendo por {}", self.id, msg.0);
//         thread::sleep(Duration::from_secs(msg.0 * 10));
//         println!("[{}] desperté de {}", self.id, msg.0);
//     }
// }

#[actix_rt::main]
async fn main() {
    // console_subscriber::init();

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