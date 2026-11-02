rustc test_parse.rs --extern soma=target/debug/libsoma.rlib --edition 2021 -L dependency=target/debug/deps
./test_parse
