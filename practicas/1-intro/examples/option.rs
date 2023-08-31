use std::fs::File;

fn dividir(num: f64, den: f64) -> Option<f64> {
    if den == 0.0 {
        None
    } else {
        Some(num/den)
    }
}

fn contar_lineas(path: &str) -> Result<u64, String> {
    let file = File::open(path);
    if !file.is_ok() {
        return Err(String::from("error al abrir archivo"))
    }
    Ok(42)
}

fn div_mul(n:f64, d:f64, m:f64) -> Option<f64> {
    let d = dividir(n, d)?;
    Some(d * m)
}

fn main() {
    println!("{:?}", dividir(3., 0.));
    println!("{:?}", dividir(3., 2.));
    println!("{:?}", dividir(3., 2.).map(|n| n*3.));

    if let Some(r) = dividir(3., 0.) {
        println!("El resultado es {}", r)
    } else {
        println!("else")
    }

    println!("{:?}", div_mul(3., 0., 2.));
    println!("{:?}", div_mul(3., 1., 2.));

    println!("{:?}", contar_lineas("wrong path"));
    println!("{:?}", contar_lineas(file!()));
    println!("{:?}", contar_lineas("wrong path").expect("file should exist!"))

}