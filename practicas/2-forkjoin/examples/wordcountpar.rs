use std::collections::HashMap;
use std::{env, thread};
use std::fs::{File, read_dir};
use std::io::{BufRead, BufReader};

use rayon::prelude::{IntoParallelRefIterator, ParallelBridge, ParallelIterator};
use std::path::PathBuf;
use std::time::{Instant, Duration};

fn main() {

    let start = Instant::now();
    let result = read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/data")).unwrap()
        .flatten()
        .map(|d| d.path())
        .collect::<Vec<PathBuf>>()
        .par_iter()
        .flat_map(|path| {
            let file = File::open(path);
            let reader = BufReader::new(file.unwrap());
            reader.lines().par_bridge()
        })
        .map(|l| {
            let line = l.unwrap();
            let words = line.split(' ');
            thread::sleep(Duration::from_millis(100));
            let mut counts = HashMap::new();
            words.for_each(|w| *counts.entry(w.to_string()).or_insert(0) += 1);
            counts
        })
        .reduce(|| HashMap::new(), |mut acc, words| {
            words.iter().for_each(|(k, v)| *acc.entry(k.clone()).or_insert(0) += v);
            acc
        });
    println!("{:?}", start.elapsed());


    println!("{:?}", result);
}