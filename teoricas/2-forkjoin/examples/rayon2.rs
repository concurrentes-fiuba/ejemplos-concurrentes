use std::vec;
use rayon::prelude::*;

fn main() {
    let numeros = vec![1_i32, 3, 4, 6, 10];
    let resultado = suma_de_cuadrados(&numeros);
    println!("Resultado: {}", resultado);


    let res = join();
    println!("Resultados: {} y {}", res.0, res.1);
}

fn suma_de_cuadrados(input: &[i32]) -> i32 {
    input.par_iter().map(|&i| i * i).sum()
}



fn join() -> (i32, i32) {
    let numeros1 = vec![1_i32, 3, 4, 6, 10];
    let numeros2 = vec![1_i32, 2];
    rayon::join(
        || suma_de_cuadrados(&numeros1),
        || suma_de_cuadrados(&numeros2),
    )
}
