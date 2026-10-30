fn parse_and_sum(values: &[&str]) -> Result<i32, String> {
    let mut total = 0;
    for v in values {
        let n: i32 = v.parse().map_err(|e| format!("bad int '{}': {}", v, e))?;
        total += n;
    }
    Ok(total)
}

fn safe_div(a: i32, b: i32) -> Result<i32, String> {
    if b == 0 {
        Err("division by zero".to_string())
    } else {
        Ok(a / b)
    }
}

fn main() {
    let good = ["10", "20", "30"];
    let bad = ["10", "oops", "30"];

    match parse_and_sum(&good) {
        Ok(total) => {
            println!("sum = {}", total);
            match safe_div(total, 3) {
                Ok(q) => println!("sum / 3 = {}", q),
                Err(e) => println!("error: {}", e),
            }
            match safe_div(total, 0) {
                Ok(q) => println!("sum / 0 = {}", q),
                Err(e) => println!("error: {}", e),
            }
        }
        Err(e) => println!("error: {}", e),
    }

    match parse_and_sum(&bad) {
        Ok(total) => println!("sum = {}", total),
        Err(e) => println!("error: {}", e),
    }
}
