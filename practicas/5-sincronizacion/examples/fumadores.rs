extern crate rand;
extern crate std_semaphore;
extern crate num_derive;
extern crate num_traits;

use std::sync::Arc;
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

    let smokers:Vec<JoinHandle<()>> =  (0..N)
        .map(|i|  {
            let agent_sem_smoker = agent_sem.clone();
            let ingredient_sems_smoker = ingredient_sems.clone();
            thread::spawn(move || loop {
                let me = Ingredients::from_usize(i).unwrap();
                for ing_id in 0..N {
                    if ing_id != i {
                        let ing = Ingredients::from_usize(ing_id).unwrap();
                        println!("[Fumador {:?}] Esperando {:?}", me, ing);
                        ingredient_sems_smoker[ing_id].acquire();
                        println!("[Fumador {:?}] Obtuve {:?}", me, ing);
                    }
                }
                println!("[Fumador {:?}] Fumando", me);
                thread::sleep(Duration::from_secs(2));
                agent_sem_smoker.release();
                println!("[Fumador {:?}] Terminé", me);
            })
        })
        .collect();

    let _:Vec<()> = smokers.into_iter()
        .flat_map(|x| x.join())
        .collect();

    agent.join().unwrap();
}
