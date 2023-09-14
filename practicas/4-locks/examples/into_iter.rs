#[derive(Debug)]
struct NonCopy(i32);

fn main() {

    let v = vec![NonCopy(1), NonCopy(2), NonCopy(3)];

    v.iter().for_each(|v| println!("{:?}", v));

    v.into_iter().for_each(|v| println!("{:?}", v));

    //println!("{:?}", v)
}