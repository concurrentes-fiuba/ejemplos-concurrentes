#[derive(Debug)]
struct Persona {
    edad: u16
}

fn main() {
    let p: &Persona;
    {
        let ariel = Persona { edad: 37 };
        p = &ariel;
    }
    println!("{:?}", p);
}