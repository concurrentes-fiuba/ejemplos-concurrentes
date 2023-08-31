use std::fs::File;
use std::ops::Mul;
use Message::Fire;

#[derive(Debug)]
enum Palo {
    Oro,
    Copa,
    Espada,
    Basto,
}

enum Message {
    Fire,
    Move { x: i32, y: i32 },
    Say(String),
}

fn print_message(m:Message) {
    match m {
        Message::Fire => println!("Fire"),
        Message::Move{ x:_, y} if y > 10 => println!("Corre hacia arriba"),
        Message::Move{ x, y} => println!("Se mueve hacia {}, {}", x, y),
        Message::Say(msg) => println!("Dice: {}", msg),
    };
}

fn main() {
    let mut palo: Palo = Palo::Oro;
    println!("{:?}", palo);
    palo = Palo::Copa;
    println!("{:?}", palo);

    print_message(Message::Fire);
    print_message(Message::Move { x:3, y:2 });
    print_message(Message::Move { x:3, y:20 });
    print_message(Message::Say("hello".to_string()));
    print_message(Message::Say("chau".to_string()));
}
