use std::rc::Rc;

fn main() {

    let mi_vector = Rc::new(vec![1.0, 2.0, 3.0]);
    let a = mi_vector.clone();

    println!("{:?}", mi_vector);
    println!("{:?}", a);

    //mi_vector.push(5.0);
}