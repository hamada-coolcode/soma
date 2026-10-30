use soma::lexer::scan;
use soma::parse::recover::parse_file_recovering;
use std::fs;

// Deterministic xorshift64* generator
struct XorShift64Star {
    state: u64,
}

impl XorShift64Star {
    fn new(seed: u64) -> Self {
        assert_ne!(seed, 0);
        Self { state: seed }
    }

    fn next(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_range(&mut self, min: usize, max: usize) -> usize {
        assert!(max >= min);
        if min == max {
            return min;
        }
        let range = max - min;
        min + (self.next() as usize % range)
    }

    fn next_bytes(&mut self, len: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(len);
        for _ in 0..len {
            bytes.push(self.next() as u8);
        }
        bytes
    }
}

fn test_str(src: &str) {
    let res = scan(src);
    if let Ok(toks) = res {
        let _ = parse_file_recovering(&toks);
    }
}

#[test]
fn test_fuzz_random_bytes() {
    let seed = 42; // Fixed seed for determinism
    println!("Random byte fuzzing with seed: {}", seed);
    let mut rng = XorShift64Star::new(seed);

    for _ in 0..500 {
        let len = rng.next_range(0, 2000);
        let bytes = rng.next_bytes(len);
        let s = String::from_utf8_lossy(&bytes).into_owned();
        test_str(&s);
    }
}

#[test]
fn test_fuzz_structured_mutants() {
    let seed = 12345;
    println!("Structured fuzzing with seed: {}", seed);
    let mut rng = XorShift64Star::new(seed);

    let mut base_examples = Vec::new();
    let example_dirs = fs::read_dir("examples").unwrap();
    let mut example_dirs: Vec<_> = example_dirs.map(|r| r.unwrap()).collect();
    example_dirs.sort_by_key(|e| e.path());
    for entry in example_dirs {
        if entry.path().is_dir() {
            let mut soma_files: Vec<_> = fs::read_dir(entry.path()).unwrap().map(|r| r.unwrap()).collect();
            soma_files.sort_by_key(|e| e.path());
            for f in soma_files {
                if f.path().extension().map_or(false, |ext| ext == "soma") {
                    base_examples.push(fs::read_to_string(f.path()).unwrap());
                }
            }
        }
    }

    // Add some inline examples if we found none
    if base_examples.is_empty() {
        base_examples = vec![
            "fn main -> !\n  x = 1\n".to_string(),
            "object Config\n  a: int\n".to_string(),
            "fn f(x: int) -> int\n  return x + 1\n".to_string(),
            "import lib:std\n".to_string(),
        ];
    }

    for _ in 0..300 {
        let idx = rng.next_range(0, base_examples.len());
        let mut base = base_examples[idx].to_string();

        let mutation = rng.next_range(0, 6);
        match mutation {
            0 => { // truncate
                if !base.is_empty() {
                    let cut = rng.next_range(0, base.len());
                    if base.is_char_boundary(cut) { base.truncate(cut); }
                }
            }
            1 => { // delete char
                if !base.is_empty() {
                    let idx = rng.next_range(0, base.len());
                    if base.is_char_boundary(idx) {
                        base.remove(idx);
                    }
                }
            }
            2 => { // duplicate char
                if !base.is_empty() {
                    let idx = rng.next_range(0, base.len());
                    if let Some(c) = base[idx..].chars().next() {
                        base.insert(idx, c);
                    }
                }
            }
            3 => { // insert random ascii
                let mut idx = rng.next_range(0, base.len() + 1);
                while !base.is_char_boundary(idx) && idx > 0 {
                    idx -= 1;
                }
                if base.is_char_boundary(idx) {
                    let c = (rng.next_range(32, 127) as u8) as char;
                    base.insert(idx, c);
                }
            }
            4 => { // replace with random
                if !base.is_empty() {
                    let idx = rng.next_range(0, base.len());
                    if base.is_char_boundary(idx) {
                        if let Some(c) = base[idx..].chars().next() {
                            let len = c.len_utf8();
                            base.replace_range(idx..idx+len, &((rng.next_range(32, 127) as u8) as char).to_string());
                        }
                    }
                }
            }
            5 => { // unbalance delimiter
                let delimiters = ['(', ')', '[', ']', '{', '}'];
                if !base.is_empty() {
                    let idx = rng.next_range(0, base.len());
                    if base.is_char_boundary(idx) {
                        let delim = delimiters[rng.next_range(0, delimiters.len())];
                        base.insert(idx, delim);
                    }
                }
            }
            _ => {}
        }
        test_str(&base);

        // Check determinism
        let r1_lex = scan(&base);
        let r2_lex = scan(&base);
        assert_eq!(r1_lex, r2_lex);
    }
}

#[test]
fn test_corpus_fuzz() {
    let mut entries: Vec<_> = fs::read_dir("tests/corpus/fuzz")
        .unwrap()
        .map(|r| r.unwrap())
        .collect();

    // Sort for determinism
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        if entry.path().extension().map_or(false, |ext| ext == "soma") {
            let src = fs::read_to_string(entry.path()).unwrap();
            let lex_res = scan(&src);
            if let Ok(toks) = lex_res {
                let (_items, diags) = parse_file_recovering(&toks);
                assert!(!diags.is_empty(), "Expected diagnostics for malicious input: {:?}", entry.path());
            } else {
                // Lexer error, which is a diagnostic failure of some kind
                assert!(lex_res.is_err());
            }
        }
    }
}
