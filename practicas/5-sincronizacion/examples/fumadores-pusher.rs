extern crate rand;
extern crate std_semaphore;
extern crate num_derive;
extern crate num_traits;

use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;

use rand::{thread_rng};
use rand::seq::SliceRandom;
use std_semaphore::Semaphore;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::thread::JoinHandle;

const N:usize = 3;

#[derive(Clone, Copy, Debug, FromPrimitive)]
enum Ingredients {
    Tobacco = 0,
    Paper,
    Fire
}

const ALL_INGREDIENTS: [Ingredients; N] = [Ingredients::Tobacco, Ingredients::Paper, Ingredients::Fire];

fn main() {


    let agent_sem = Arc::new(Semaphore::new(1));
    let ingredient_sems: Arc<Vec<Semaphore>> = Arc::new((0..N)
                                       .map(|_| Semaphore::new(0))
                                       .collect());

    let smoker_sems: Arc<Vec<Semaphore>> = Arc::new((0..N)
        .map(|_| Semaphore::new(0))
        .collect());

    let scoreboard = Arc::new(RwLock::new(vec!(false, false, false)));

    let agent_sem_a = agent_sem.clone();
    let ingredients_sem_a = ingredient_sems.clone();

    let agent = thread::spawn(move || loop {
        println!("[Agente] Esperando sem");
        agent_sem_a.acquire();

        let mut ings = ALL_INGREDIENTS.to_vec();
        ings.shuffle(&mut thread_rng());
        let selected_ings = &ings[0..N-1];
        for ing in selected_ings {
            println!("[Agente] Pongo {:?}", ing);
            ingredients_sem_a[*ing as usize].release();
        }
    });

    let pushers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let ingredient_sems_pusher = ingredient_sems.clone();
            let scoreboard_pusher = scoreboard.clone();
            let smoker_sems_pusher = smoker_sems.clone();
            thread::spawn(move || loop {
                let me = Ingredients::from_usize(i).unwrap();
                println!("[Pusher {:?}] Esperando ", me);
                ingredient_sems_pusher[i].acquire();
                println!("[Pusher {:?}] Mi ingrediente esta en la mesa ", me);
                if let Ok(mut scores) = scoreboard_pusher.write() {
                    scores[i] = true;
                    if !notify_smokers_if_possible(&mut *scores, &smoker_sems_pusher) {
                        println!("[Pusher {:?}] Lo pongo en el tablero", me);
                    } else {
                        println!("[Pusher {:?}] Despierto fumador", me);
                    }
                }
            })
        })
        .collect();

    let smokers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let agent_sem_smoker = agent_sem.clone();
            let smoker_sems_smoker = smoker_sems.clone();
            thread::spawn(move || loop {
                let me = Ingredients::from_usize(i).unwrap();
                println!("[Fumador {:?}] Esperando", me);
                smoker_sems_smoker[i].acquire();
                println!("[Fumador {:?}] Fumando", me);
                thread::sleep(Duration::from_secs(2));
                agent_sem_smoker.release();
                println!("[Fumador {:?}] Terminé", me);
            })
        })
        .collect();

    let _:Vec<()> = pushers.into_iter()
        .chain(smokers.into_iter())
        .flat_map(|x| x.join())
        .collect();

    agent.join().unwrap();
}

fn notify_smokers_if_possible(scores:&mut Vec<bool>, smoker_sems_pusher: &Arc<Vec<Semaphore>>) -> bool{
    if scores[Ingredients::Tobacco as usize] {
        if scores[Ingredients::Paper as usize] {
            smoker_sems_pusher[Ingredients::Fire as usize].release();
            scores[Ingredients::Paper as usize] = false;
            scores[Ingredients::Tobacco as usize] = false;
            true
        } else if scores[Ingredients::Fire as usize] {
            smoker_sems_pusher[Ingredients::Paper as usize].release();
            scores[Ingredients::Fire as usize] = false;
            scores[Ingredients::Tobacco as usize] = false;
            true
        } else {
            false
        }
    } else if scores[Ingredients::Paper as usize] && scores[Ingredients::Fire as usize] {
        smoker_sems_pusher[Ingredients::Tobacco as usize].release();
        scores[Ingredients::Paper as usize] = false;
        scores[Ingredients::Fire as usize] = false;
        true
    } else {
        false
    }
}
