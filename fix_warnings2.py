import sys

def main():
    with open('src/types/check_expr.rs', 'r') as f:
        c = f.read()

    c = c.replace("use crate::ast::expr::{Expr, ExprKind, Arg, BinOp, UnOp, Block, Stmt, StmtKind};", "use crate::ast::expr::{Expr, ExprKind, Arg, BinOp, UnOp, Block, StmtKind};")
    c = c.replace("let mut arm_ty = self.table.fresh_var();", "let arm_ty = self.table.fresh_var();")
    c = c.replace("let scrutinee_ty = self.check(scrutinee)?;", "let _scrutinee_ty = self.check(scrutinee)?;")
    c = c.replace("let old_span = last_span;", "let _old_span = last_span;")

    with open('src/types/check_expr.rs', 'w') as f:
        f.write(c)

if __name__ == "__main__":
    main()
