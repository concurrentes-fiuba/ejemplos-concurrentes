use std::sync::Arc;
use std::thread;


fn main() {

    let five = Arc::new(5);

    for _ in 0..10 {
        let five_clone = five.clone();

        thread::spawn(move || {
            println!("{:?}", five_clone);
        });
    }
}


/*

fn main() {

    let mi_arc = Arc::new(vec![1.0, 2.0, 3.0]);

    for _ in 0..10 {
        //let mi_arc_clone = mi_arc.clone();
        let mi_arc_clone = Arc::clone(&mi_arc);

        thread::spawn(move || {
            mi_arc_clone.push(5.0);
            println!("{:?}", mi_arc_clone);
        });
    }
}
*/