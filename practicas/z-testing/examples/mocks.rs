#[cfg(test)]
use mockall::automock;

use rand::{thread_rng, Rng};
use rand::prelude::ThreadRng;

#[cfg_attr(test, automock)]
trait RandomNumberGenerator {
    fn generate(&mut self) -> f64;
}

impl RandomNumberGenerator for ThreadRng {
    fn generate(&mut self) -> f64 {
        self.gen()
    }
}

fn flip_coin(random:&mut dyn RandomNumberGenerator) -> bool {
    random.generate() >= 0.5
}

fn main() {
    println!("{}", flip_coin(&mut thread_rng()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lose() {
        let mut mock = MockRandomNumberGenerator::new();
        mock.expect_generate()
            .returning(|| 0.3);
        assert_eq!(false, flip_coin(&mut mock));
    }

    #[test]
    fn test_win() {
        let mut mock = MockRandomNumberGenerator::new();
        mock.expect_generate()
            .returning(|| 0.7);
        assert_eq!(true, flip_coin(&mut mock));
    }
}

