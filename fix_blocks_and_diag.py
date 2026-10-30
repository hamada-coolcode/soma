import sys

def main():
    with open('src/types/check_expr.rs', 'r') as f:
        c = f.read()

    # 1. Block statements checking
    replace_block = """fn check_block(&mut self, block: &Block, expected_ty: TypeId) -> Result<(), Diagnostic> {
        let mut last_ty = self.table.intern(Type::Unit);
        let mut last_span = block.span;

        for stmt in &block.stmts {
            last_span = stmt.span;
            match &stmt.kind {
                StmtKind::Expr(e) => {
                    last_ty = self.check(e)?;
                }
                StmtKind::Assign { target, value } => {
                    let target_ty = self.check(target)?;
                    let value_ty = self.check(value)?;
                    if let Err(err) = unify(self.table, target_ty, value_ty) {
                        return Err(Diagnostic::new(ErrorKind::Type, format!("[E0401] Assignment type mismatch: {}", err.message), stmt.span));
                    }
                    last_ty = self.table.intern(Type::Unit);
                }
                StmtKind::While { cond, body } => {
                    let cond_ty = self.check(cond)?;
                    let t_bool = self.table.intern(Type::Bool);
                    if unify(self.table, cond_ty, t_bool).is_err() {
                        return Err(Diagnostic::new(ErrorKind::Type, "[E0401] While condition must be bool", cond.span));
                    }
                    let dummy_ty = self.table.intern(Type::Unit);
                    self.check_block(body, dummy_ty)?;
                    last_ty = self.table.intern(Type::Unit);
                }
                StmtKind::For { var, iter, body } => {
                    let iter_ty = self.check(iter)?;
                    let elem_ty = self.table.fresh_var();
                    let list_ty = self.table.intern(Type::List(elem_ty));
                    if unify(self.table, iter_ty, list_ty).is_err() {
                        return Err(Diagnostic::new(ErrorKind::Type, "[E0401] For iter must be a list", iter.span));
                    }

                    self.scope.push();
                    use crate::resolve::{Symbol, SymbolKind};
                    self.scope.define(Symbol { name: var.text.clone(), kind: SymbolKind::Local, span: var.span, mutable: true });
                    let old = self.env.insert(var.text.clone(), elem_ty);

                    let dummy_ty = self.table.intern(Type::Unit);
                    self.check_block(body, dummy_ty)?;

                    self.scope.pop();
                    if let Some(v) = old {
                        self.env.insert(var.text.clone(), v);
                    } else {
                        self.env.remove(&var.text);
                    }
                    last_ty = self.table.intern(Type::Unit);
                }
                StmtKind::Loop { body } => {
                    let dummy_ty = self.table.intern(Type::Unit);
                    self.check_block(body, dummy_ty)?;
                    last_ty = self.table.intern(Type::Unit);
                }
                StmtKind::Match { scrutinee, arms } => {
                    let scrutinee_ty = self.check(scrutinee)?;
                    let mut arm_ty = self.table.fresh_var();
                    for arm in arms {
                        // Very simplified pattern checking
                        let body_ty = self.table.fresh_var();

                        let old_span = last_span;

                        // We check the block by wrapping Stmt in a Block manually
                        // wait, arm.body is Stmt, so we just check it
                        let block_mock = Block { stmts: vec![arm.body.clone()], span: arm.span };
                        self.check_block(&block_mock, body_ty)?;

                        if let Err(err) = unify(self.table, arm_ty, body_ty) {
                            return Err(Diagnostic::new(ErrorKind::Type, format!("[E0401] Match arm type mismatch: {}", err.message), arm.span));
                        }
                        last_ty = arm_ty;
                    }
                }
                StmtKind::Return(expr_opt) => {
                    if let Some(e) = expr_opt {
                        self.check(e)?;
                    }
                    last_ty = self.table.intern(Type::Never);
                }
                StmtKind::Break | StmtKind::Continue => {
                    last_ty = self.table.intern(Type::Never);
                }
                StmtKind::Item(_) => {
                    last_ty = self.table.intern(Type::Unit);
                }
            }
        }

        if let Err(err) = unify(self.table, expected_ty, last_ty) {
            return Err(Diagnostic::new(ErrorKind::Type, format!("[E0401] Block type mismatch: {}", err.message), last_span));
        }
        Ok(())
    }"""

    # Let's replace the whole check_block
    import re
    c = re.sub(r"fn check_block.*?Ok\(\(\)\)\n    }", replace_block, c, flags=re.DOTALL)

    with open('src/types/check_expr.rs', 'w') as f:
        f.write(c)

if __name__ == "__main__":
    main()
