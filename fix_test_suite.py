import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

text = text.replace("let diags = check_items(&items);", "let _diags = check_items(&items);")
text = text.replace("env: &mut HashMap<String, u32>,", "_env: &mut HashMap<String, u32>,")
text = text.replace("check_fn(&mut table, &mut env, f, &mut diags);", "check_fn(&mut table, &mut env, f, &mut diags);")

with open("src/types/check_item.rs", "w") as f:
    f.write(text)
