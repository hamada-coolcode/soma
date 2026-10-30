import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

text = text.replace("let self_ty = table.intern", "let _self_ty = table.intern")
text = text.replace("env: &mut HashMap<String, u32>,", "_env: &mut HashMap<String, u32>,")
text = text.replace("let res = check_expr::check(table, inner)", "let _res = check_expr::check(table, inner)")

with open("src/types/check_item.rs", "w") as f:
    f.write(text)
