use std::thread;

fn worker(id: i32) -> i64 {
    let mut sum: i64 = 0;
    for j in 0..1_000_000 {
        sum += j;
    }
    println!("worker {} sum = {}", id, sum);
    sum
}

fn main() {
    let handles: Vec<_> = (0..4).map(|i| thread::spawn(move || worker(i))).collect();

    let total: i64 = handles.into_iter().map(|h| h.join().unwrap()).sum();
    println!("total = {}", total);
}
