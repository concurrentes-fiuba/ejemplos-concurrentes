extern crate rand;

use std::thread;
use std::time::Duration;
use rand::{thread_rng, Rng};
use std::thread::JoinHandle;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender, SendError};

struct AfipNotifier {
    send: Sender<f64>
}

impl AfipNotifier {
    fn notify(&self, msg:f64) -> Result<(), SendError<f64>> {
        self.send.send(msg)
    }
}

fn main() {
    let mut plata = 1000.0;

    const INVERSORES: i32 = 10;

    let (devolucion_send, devolucion_receive) = mpsc::channel();

    let (afip_send, afip_receive) = mpsc::channel();

    thread::spawn(move || loop { println!("[AFIP] {} ", afip_receive.recv().unwrap()) });

    let inversores: Vec<(Sender<f64>, JoinHandle<()>)> = (0..INVERSORES)
        .map(|id| {
            let (inversor_send, inversor_receive) = mpsc::channel();
            let devolucion_send_inversor = devolucion_send.clone();
            let afip_send_inversor = afip_send.clone();
            let t = thread::spawn(move || inversor(id, inversor_receive, devolucion_send_inversor, AfipNotifier { send: afip_send_inversor }));
            (inversor_send, t)
        })
        .collect();


    while plata > 5.0 {
        let prestamo = plata / (INVERSORES as f64);
        for (inversor, _) in &inversores {
            inversor.send(prestamo).unwrap();
        }

        let mut plata_semana = 0.0;
        for _ in &inversores {
            plata_semana += devolucion_receive.recv().unwrap();
        }
        println!("[Banquero] final de semana {}", plata_semana);
        plata = plata_semana
    }

    let _:Vec<()> = inversores.into_iter()
        .flat_map(|(_,h)| h.join())
        .collect();
}

fn inversor(id: i32, prestamo: Receiver<f64>, devolucion: Sender<f64>, afip: AfipNotifier) {
    loop {
        let plata_inicial = prestamo.recv().unwrap();
        println!("[Inversor {}] me dan {}", id, plata_inicial);
        thread::sleep(Duration::from_secs(2));
        let resultado = plata_inicial * thread_rng().gen_range(0.5, 1.5);
        println!("[Inversor {}] devuelvo {}", id, resultado);
        devolucion.send(resultado);
        afip.notify(resultado);
    }
}