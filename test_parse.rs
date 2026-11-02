fn main() {
    let src = "test\n  \"pass 1\"\n    assert = 1\n";
    let parsed = soma::driver::parse_source("test", src).unwrap();
    println!("{:?}", parsed.items);
}
