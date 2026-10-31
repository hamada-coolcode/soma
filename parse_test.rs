use soma::driver;

fn main() {
    let src = "fn main\\n  x = f\\\"{x}\\\"\\n";
    let p = driver::parse_source("test", src);
    println!("{:#?}", p);
}
