use std::ops::Add;

#[derive(Debug)]
struct Persona {
    nombre: String,
    apellido: String
}

#[derive(Debug)]
struct NumeroImaginario(f64, f64);

impl NumeroImaginario {
    // "Método estático"
    fn new(r:f64, i:f64) -> NumeroImaginario {
        NumeroImaginario(r, i)
    }

    fn modulo(&self) -> f64 {
        (self.0*self.0 + self.1*self.1).sqrt()
    }
}

impl Add for NumeroImaginario {
    type Output = NumeroImaginario;

    fn add(self, rhs: Self) -> Self::Output {
        NumeroImaginario(self.0 + rhs.0, self.1 + rhs.1)
    }
}

fn main() {
    let nombre = String::from("ariel");
    let ariel = Persona {
        nombre,
        apellido: String::from("scarpinelli")
    };

    println!("{:?}", ariel);
    println!("{}", NumeroImaginario::new(3.0, 4.0).modulo());

    println!("{:?}", NumeroImaginario::new(1.0, 2.0) + NumeroImaginario::new(1.0, 2.0));

}