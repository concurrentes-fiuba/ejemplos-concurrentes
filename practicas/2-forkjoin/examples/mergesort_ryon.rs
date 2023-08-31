use std::thread;
use std::time::{Duration, Instant};

fn main() {

    let data = [7, 3, 2, 16, 24, 4, 11, 9];

    rayon::ThreadPoolBuilder::new().build_global();

    let start = Instant::now();
    let merged = mergesort(&data);
    println!("{:?}", start.elapsed());

    println!("{:?}", merged);
}

fn mergesort(data: &[i32]) -> Vec<i32> {

    thread::sleep(Duration::from_secs(2));

    let mid = data.len() / 2;
    if mid == 0 {
        return data.to_vec();
    }

    let left_data = &data[..mid];
    let right_data = &data[mid..];

    let (left, right) = rayon::join(|| mergesort(left_data), || mergesort(right_data));

    merge(left, right)

}

fn merge(left: Vec<i32>, right: Vec<i32>) -> Vec<i32> {
    let mut left_index = 0;
    let mut right_index = 0;
    let mut ret_index = 0;
    let mut ret = vec![0; left.len() + right.len()];

    while left_index < left.len() && right_index < right.len() {
        if left[left_index] <= right[right_index] {
            ret[ret_index] = left[left_index];
            ret_index += 1;
            left_index += 1;
        } else {
            ret[ret_index] = right[right_index];
            ret_index += 1;
            right_index += 1;
        }
    }

    if left_index < left.len() {
        ret[ret_index..].copy_from_slice(&left[left_index..]);
    }
    if right_index < right.len() {
        ret[ret_index..].copy_from_slice(&right[right_index..]);
    }

    ret
}