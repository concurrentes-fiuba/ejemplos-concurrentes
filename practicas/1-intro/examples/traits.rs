struct NumeroImaginario(f64, f64);

impl NumeroImaginario {
    fn modulo(&self) -> f64 {
        (self.0*self.0 + self.1*self.1).sqrt()
    }
}

trait MagnitudVectorial {
    fn norma(&self) -> f64;
    fn max(&self, other: &MagnitudVectorial) -> f64 {
        self.norma().max(other.norma())
    }
}

impl MagnitudVectorial for NumeroImaginario {
    fn norma(&self) -> f64 {
        self.modulo()
    }

}


fn main() {
    println!("{}", NumeroImaginario(3.0, 4.0).max(&NumeroImaginario(4.0, 5.0)));
}