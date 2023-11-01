use std::ops::Add;

fn add<T: Add<Output = T>>(l: T, r: T) -> T {
    l + r
}

fn main() {
    println!("{:?}", add(2, 3));
    println!("{:?}", add(2.1, 3.2));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_adds_i32s() {
        assert_eq!(5, add(3_i32, 2));
    }

    #[test]
    fn it_adds_i64s() {
        assert_eq!(5, add(3_i64, 2));
    }

    #[test]
    fn it_adds_f64s() {
        let actual = add(3.2_f64, 2.1);
        assert!((5.3 - actual).abs() < 1e-10);
    }
}