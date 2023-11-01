use rand::{thread_rng, Rng};
use rand::prelude::ThreadRng;

fn main() {
    println!("{}", play(|| thread_rng().gen()))
}

fn play(rng: fn() -> f64) -> String {
    get_message(flip_coin(rng))
}

fn flip_coin(rng: fn() -> f64) -> bool {
    let random: f64 = rng();
    random >= 0.5
}

fn get_message(won: bool) -> String {
    if won {
        "Ganaste".to_string()
    } else {
        "Perdiste".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_winner_message() {
        assert_eq!("Ganaste", get_message(true));
    }

    #[test]
    fn test_get_loser_message() {
        assert_eq!("Perdiste", get_message(false));
    }

    #[test]
    fn test_flip_true() {
        assert_eq!(true, flip_coin(|| 0.7));
    }

    #[test]
    fn test_flip_false() {
        assert_eq!(false, flip_coin(|| 0.3));
    }

    #[test]
    fn test_play_win() {
        assert_eq!("Ganaste", play(|| 0.7));
    }

    #[test]
    fn test_play_lose() {
        assert_eq!("Perdiste", play(|| 0.3));
    }
}
