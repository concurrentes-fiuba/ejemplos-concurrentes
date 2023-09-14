extern crate rand;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use std::thread::JoinHandle;
use rand::{Rng, thread_rng};


/**
Al tiempo el señor fallece.
Los hijos deciden que los inversores sigan trabajando el dinero con algunas condiciones:
- Ellos no se hacen cargo de nada, los inversores solos toman dinero de la cuenta y lo devuelven al final de la semana
- Cada inversor puede reinvetir el capital y hasta 50% de la ganancia de la semana anterior, o bien todo el capital en caso de haber perdido.
- Las inversiones deberán ser menos riesgosas, pudiendo dejar de -10% a +10%
*/

const INVERSORES: u32 = 5;
const SALDO_INICIAL: f64 = 100000.0;

fn main() {
    let cuenta = Arc::new(RwLock::new(SALDO_INICIAL));

    let inversores: Vec<JoinHandle<()>> = (0..INVERSORES)
        .map(|id| {
            let cuenta_local = cuenta.clone();
            thread::spawn(move || inversor(id, SALDO_INICIAL / (INVERSORES as f64), cuenta_local))
        })
        .collect();

    inversores.into_iter()
        .flat_map(|x| x.join())
        .for_each(drop)

}

fn inversor(id:u32, inicial:f64, cuenta:Arc<RwLock<f64>>) {
    let mut capital = inicial;
    while capital > 5.0 {
        println!("[INVERSOR {}] inicio semana {}", id, capital);
        if let Ok(mut saldo) = cuenta.write() {
            *saldo -= capital;
        }
        thread::sleep(Duration::from_millis(1000));
        let resultado = capital * thread_rng().gen_range(0.9, 1.1);
        if let Ok(mut money_guard) = cuenta.write() {
            *money_guard += resultado;
        }
        println!("[INVERSOR {}] resultado {}", id, resultado);
        if (resultado > capital) {
            capital += (resultado - capital) * 0.5;
        } else {
            capital = resultado;
        }
    }

}
