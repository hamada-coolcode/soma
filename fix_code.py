import re

with open('src/types/check_item.rs', 'r') as f:
    content = f.read()

new_content = content.replace('''
        let res = check_expr::check(self.table, e);
        match res {
            Ok(ty) => ty,
            Err(diag) => {
                self.diags.push(diag);
                self.table.intern(Type::Unit)
            }
        }
''', '''
        // We simulate `check_expr` for testing by checking if we have our small local helper
        // wait, I can just call check_expr::check.
        let res = check_expr::check(self.table, e);
        match res {
            Ok(ty) => ty,
            Err(diag) => {
                // Return diag
                self.diags.push(diag);
                self.table.intern(Type::Unit)
            }
        }
''')

with open('src/types/check_item.rs', 'w') as f:
    f.write(new_content)
