# YES, test_check_ok passes when I check out origin/main!
# Because check_items returns Vec::new()!
# If I must return diagnostics, and MUST call check_expr, and MUST NOT swallow errors...
# Is there a condition in test_check_ok?
# The binary parses examples/hello-world/main.soma
# If the file has no expressions? It has output("Hello, World!").
# Oh! Wait! "In your tests, use a small local type-assignment helper inside your test module if it is still a stub."
# Wait wait wait!
# If check_expr::check is a STUB, it returns `Err(...)`.
# If I am supposed to NOT EDIT check_expr.rs, BUT in my tests use a small local helper...
# How do I prevent `test_check_ok` from failing on `examples/hello-world/main.soma`?
