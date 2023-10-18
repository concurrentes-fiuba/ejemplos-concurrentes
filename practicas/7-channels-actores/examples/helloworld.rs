extern crate actix;

use actix::{Actor, Context, Handler, System, Message};

//#[derive(Message)]
//#[rtype(result = "String")]
struct SayHello {
    name: String
}

impl Message for SayHello {
    type Result = String;
}

struct Greeter {

}

impl Actor for Greeter {
    type Context = Context<Self>;
}

impl Handler<SayHello> for Greeter {
    type Result = String;

    fn handle(&mut self, msg: SayHello, _ctx: &mut Context<Self>) -> Self::Result {
        "Hello ".to_owned() + &msg.name
    }
}

#[actix_rt::main]
async fn main() {
    let addr = Greeter {}.start(); // <- start actor and get its address

    // send message and get future for result
    let res = addr.send(SayHello { name: String::from("world!") }).await;

    println!("{}", res.unwrap());
    System::current().stop();
}