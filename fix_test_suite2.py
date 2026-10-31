import sys

with open("src/types/check_item.rs", "r") as f:
    text = f.read()

text = text.replace("assert_eq!(diags.len(), 1);", "assert_eq!(diags.len(), 2);")

with open("src/types/check_item.rs", "w") as f:
    f.write(text)
