use std::sync::RwLock;

fn main() {
    let lock = RwLock::new(1);
    let mut dataw = lock.write().expect("failed to write");
    *dataw = 2;
     
    let datar = lock.read().expect("failed to read");
    println!("El valor encontrado es: {}", *datar);
}