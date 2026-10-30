fn main() {
    let mut numbers = vec![5, 2, 9, 1, 7, 3];

    // closure as comparator
    numbers.sort_by(|a, b| a.cmp(b));
    println!("sorted asc:  {:?}", numbers);

    numbers.sort_by(|a, b| b.cmp(a));
    println!("sorted desc: {:?}", numbers);

    // higher-order: map / filter / reduce
    let doubled: Vec<i32> = numbers.iter().map(|x| x * 2).collect();
    let evens: Vec<i32> = numbers.iter().filter(|x| *x % 2 == 0).cloned().collect();
    let sum: i32 = numbers.iter().fold(0, |acc, x| acc + x);

    println!("doubled: {:?}", doubled);
    println!("evens:   {:?}", evens);
    println!("sum:     {}", sum);

    // closure capturing environment
    let factor = 3;
    let scale = |x: i32| x * factor;
    println!("scale(5): {}", scale(5));

    // pass a closure to a function
    println!("apply(5, double): {}", apply(5, |x| x * 2));
}

fn apply(x: i32, f: impl Fn(i32) -> i32) -> i32 {
    f(x)
}
