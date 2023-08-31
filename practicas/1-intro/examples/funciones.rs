fn sumar_uno(a: i32) -> i32 {
    a + 1
}

// Closure como parámetro
fn map42(mapper: fn(i32) -> i32) -> i32 {
    mapper(42)
}

fn main() {

    // Closure
    let plus_one = |a| { a + 1 };

    println!("{}", sumar_uno(3));
    println!("{}", map42(plus_one))
}
