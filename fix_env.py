import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

text = text.replace("_env: &mut HashMap<String, u32>,", "env: &mut HashMap<String, u32>,")

with open("src/types/check_item.rs", "w") as f:
    f.write(text)
