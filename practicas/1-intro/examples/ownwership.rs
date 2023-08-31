#[derive(Debug)]
// probar comentar el Copy
// #[derive(Copy, Clone)]
struct Persona {
    edad: u16
}

fn envejecer_ref(p:&mut Persona) {
    p.edad += 20;
    println!("{:?}", p)
}

fn envejecer(mut p:Persona) {
    p.edad += 20;
    println!("{:?}", p)
}

fn main() {
    let mut ariel = Persona { edad: 37 };
    println!("{:?}", ariel);
    //envejecer(ariel);
    //envejecer_ref(&mut ariel);
    println!("{:?}", ariel);
}