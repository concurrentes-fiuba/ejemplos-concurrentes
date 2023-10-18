extern crate rand;

use std::collections::HashSet;
use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

const INVERSORES: i32 = 10;

fn main() {
    let mut plata = 1000.0;

    let (devolucion_send, devolucion_receive) = mpsc::channel();

    let inversores: Vec<(Sender<f64>, JoinHandle<()>)> = (0..INVERSORES)
        .map(|id| {
            let (inversor_send, inversor_receive) = mpsc::channel();
            let devolucion_send_inversor = devolucion_send.clone();
            let t = thread::spawn(move || inversor(id, inversor_receive, devolucion_send_inversor));
            (inversor_send, t)
        })
        .collect();


    loop {
        let mut plata_semana = iniciar_semana(&mut plata, &inversores);

        let mut devolvieron = HashSet::new();

        while(devolvieron.len() < (INVERSORES as usize)) {
            let (who, how_much) = devolucion_receive.recv().unwrap();
            if !devolvieron.contains(&who) {
                devolvieron.insert(who);
                plata_semana += how_much;
            }
        }

        println!("[Banquero] final de semana {}", plata_semana);
        plata = plata_semana
    }

    let _:Vec<()> = inversores.into_iter()
        .flat_map(|(_,h)| h.join())
        .collect();
}

fn iniciar_semana(plata: &mut f64, inversores: &Vec<(Sender<f64>, JoinHandle<()>)>) -> f64 {
    let prestamo = plata / (INVERSORES as f64);
    for (inversor, _) in &inversores {
        inversor.send(prestamo).unwrap();
    }

    let mut plata_semana = 0.0;
    plata_semana
}

fn inversor(id: i32, prestamo: Receiver<f64>, devolucion: Sender<(i32, f64)>) {
    loop {
        let plata_inicial = prestamo.recv().unwrap();
        println!("[Inversor {}] me dan {}", id, plata_inicial);
        thread::sleep(Duration::from_secs(2));
        let resultado = plata_inicial * thread_rng().gen_range(0.5, 1.5);
        println!("[Inversor {}] devuelvo {}", id, resultado);
        devolucion.send((id, resultado));
    }
}