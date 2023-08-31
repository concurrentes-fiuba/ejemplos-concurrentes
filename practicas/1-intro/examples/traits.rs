struct NumeroImaginario(f64, f64);

impl NumeroImaginario {
    fn modulo(&self) -> f64 {
        (self.0*self.0 + self.1*self.1).sqrt()
    }
}

trait MagnitudVectorial {
    fn norma(&self) -> f64;
}

impl MagnitudVectorial for NumeroImaginario {
    fn norma(&self) -> f64 {
        self.modulo()
    }
}

fn max(v1:&MagnitudVectorial, v2:&MagnitudVectorial) -> f64 {
    v1.norma().max(v2.norma())
}

fn main() {
    println!("{}", max(&NumeroImaginario(3.0, 4.0), &NumeroImaginario(4.0, 5.0)));
}