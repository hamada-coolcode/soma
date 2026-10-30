use crate::ast::expr::{Expr, ExprKind, Arg, BinOp, UnOp, Block, StmtKind};
use crate::ast::item::Name;
use crate::diag::{Diagnostic, ErrorKind, Span};
use crate::resolve::Scope;
use std::collections::HashMap;

use super::ty::{TypeId, TypeTable, Type};
use super::unify::unify;

/// Infer the type of an expression (owned by task 11).
pub fn check(table: &mut TypeTable, expr: &Expr) -> Result<TypeId, Diagnostic> {
    let mut scope = Scope::new();
    let mut env = HashMap::new();
    let mut checker = ExprChecker {
        table,
        scope: &mut scope,
        env: &mut env,
    };
    checker.check(expr)
}

pub struct ExprChecker<'a> {
    pub table: &'a mut TypeTable,
    pub scope: &'a mut Scope,
    pub env: &'a mut HashMap<String, TypeId>,
}

impl<'a> ExprChecker<'a> {
    pub fn new(table: &'a mut TypeTable, scope: &'a mut Scope, env: &'a mut HashMap<String, TypeId>) -> Self {
        Self { table, scope, env }
    }

    pub fn check(&mut self, expr: &Expr) -> Result<TypeId, Diagnostic> {
        match &expr.kind {
            ExprKind::Int(_) => Ok(self.table.intern(Type::Int)),
            ExprKind::Float(_) => Ok(self.table.intern(Type::Float)),
            ExprKind::Bool(_) => Ok(self.table.intern(Type::Bool)),
            ExprKind::Str(_) | ExprKind::FStr(_) => Ok(self.table.intern(Type::Str)),
            ExprKind::Null => Ok(self.table.intern(Type::Null)),
            ExprKind::Name(name) => self.check_name(name, expr.span),
            ExprKind::SelfExpr => self.check_name(&Name { text: "self".to_string(), span: expr.span }, expr.span),
            ExprKind::List(exprs) => self.check_list(exprs, expr.span),
            ExprKind::Index { base, index } => self.check_index(base, index, expr.span),
            ExprKind::Field { base, name } => self.check_field(base, name, expr.span),
            ExprKind::Call { callee, args } => self.check_call(callee, args, expr.span),
            ExprKind::Unary { op, operand } => self.check_unary(*op, operand, expr.span),
            ExprKind::Binary { op, lhs, rhs } => self.check_binary(*op, lhs, rhs, expr.span),
            ExprKind::Range { from, to } => self.check_range(from.as_deref(), to.as_deref(), expr.span),
            ExprKind::Lambda { params, body } => self.check_lambda(params, body, expr.span),
            ExprKind::If { cond, then, elifs, else_ } => self.check_if(cond, then, elifs, else_.as_ref(), expr.span),
            ExprKind::Comprehension { expr: iter_expr, var, iter } => self.check_comprehension(iter_expr, var, iter, expr.span),
            ExprKind::Propagate(inner) => self.check_propagate(inner, expr.span),
        }
    }

    fn check_name(&mut self, name: &Name, span: Span) -> Result<TypeId, Diagnostic> {
        if self.scope.lookup(&name.text).is_none() {
            return Err(Diagnostic::new(ErrorKind::Type, format!("[E0402] Unknown name '{}'", name.text), span));
        }

        if let Some(&ty) = self.env.get(&name.text) {
            Ok(ty)
        } else {
            // Missing from env but present in scope implies it's a global or un-typed yet.
            // But per task requirements we should not unwrap, instead return missing type diagnostic if un-typed.
            Err(Diagnostic::new(ErrorKind::Type, format!("[E0403] Missing type for name '{}'", name.text), span))
        }
    }

    fn check_list(&mut self, exprs: &[Expr], _span: Span) -> Result<TypeId, Diagnostic> {
        let elem_ty = self.table.fresh_var();
        for e in exprs {
            let ty = self.check(e)?;
            if let Err(err) = unify(self.table, elem_ty, ty) {
                return Err(Diagnostic::new(ErrorKind::Type, format!("[E0401] Type mismatch: {}", err.message), e.span));
            }
        }
        Ok(self.table.intern(Type::List(elem_ty)))
    }

    fn check_index(&mut self, base: &Expr, index: &Expr, _span: Span) -> Result<TypeId, Diagnostic> {
        let base_ty = self.check(base)?;
        let index_ty = self.check(index)?;

        let t_int = self.table.intern(Type::Int);
        if let Err(_) = unify(self.table, index_ty, t_int) {
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Index must be an integer", index.span));
        }

        let elem_ty = self.table.fresh_var();
        let list_ty = self.table.intern(Type::List(elem_ty));
        if let Err(_) = unify(self.table, base_ty, list_ty) {
            // Arrays are also valid, but type checker uses List for checking.
            // Wait, we can just return elem_ty anyway if it unified.
            // Actually, arrays don't easily unify with list unless we explicitly handle it,
            // but we'll try List(elem_ty).
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Cannot index into non-list type", base.span));
        }

        Ok(elem_ty)
    }

    fn check_field(&mut self, base: &Expr, name: &Name, span: Span) -> Result<TypeId, Diagnostic> {
        let base_ty = self.check(base)?;
        let resolved = self.table.resolve(base_ty);
        if let Type::Named { name: type_name, .. } = resolved {
            let field_key = format!("{}.{}", type_name, name.text);
            if let Some(&ty) = self.env.get(&field_key) {
                return Ok(ty);
            }

            // Collect did you mean
            let mut candidates = Vec::new();
            let prefix = format!("{}.", type_name);
            for key in self.env.keys() {
                if key.starts_with(&prefix) {
                    candidates.push(key.strip_prefix(&prefix).unwrap().to_string());
                }
            }

            let suggestion = self.find_best_match(&name.text, &candidates);
            let msg = if let Some(best) = suggestion {
                format!("[E0404] Unknown field '{}' on '{}'. Did you mean '{}'?", name.text, type_name, best)
            } else {
                format!("[E0404] Unknown field '{}' on '{}'", name.text, type_name)
            };
            return Err(Diagnostic::new(ErrorKind::Type, msg, span));
        }
        Err(Diagnostic::new(ErrorKind::Type, format!("[E0401] Cannot access field on type '{}'", self.table.display(&resolved)), base.span))
    }

    fn check_call(&mut self, callee: &Expr, args: &[Arg], span: Span) -> Result<TypeId, Diagnostic> {
        let callee_ty = self.check(callee)?;

        let mut arg_tys = Vec::new();
        for arg in args {
            arg_tys.push(self.check(&arg.value)?);
        }

        let ret_ty = self.table.fresh_var();
        let expected_fn_ty = self.table.intern(Type::Fn { params: arg_tys.clone(), ret: ret_ty });

        if let Err(_) = unify(self.table, callee_ty, expected_fn_ty) {
            let resolved = self.table.resolve(callee_ty);
            match resolved {
                Type::Fn { params, .. } => {
                    if params.len() != arg_tys.len() {
                        return Err(Diagnostic::new(ErrorKind::Type, format!("[E0405] Wrong arity: expected {} arguments, got {}", params.len(), arg_tys.len()), span));
                    }
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in arguments", span));
                },
                _ => return Err(Diagnostic::new(ErrorKind::Type, "[E0406] Not callable", callee.span)),
            }
        }

        Ok(ret_ty)
    }

    fn check_unary(&mut self, op: UnOp, operand: &Expr, _span: Span) -> Result<TypeId, Diagnostic> {
        let op_ty = self.check(operand)?;
        let (expected_ty, ret_ty) = match op {
            UnOp::Not => (self.table.intern(Type::Bool), self.table.intern(Type::Bool)),
            UnOp::Neg => {
                let resolved = self.table.resolve(op_ty);
                let is_valid = match resolved {
                    Type::Int | Type::Float => true,
                    Type::Var(_) => {
                        // Default to Int for free vars in unary minus
                        let t_int = self.table.intern(Type::Int);
                        let _ = unify(self.table, op_ty, t_int);
                        true
                    }
                    _ => false,
                };
                if !is_valid {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Invalid operand for '-'", operand.span));
                }
                return Ok(op_ty);
            }
        };

        if let Err(_) = unify(self.table, op_ty, expected_ty) {
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in unary operation", operand.span));
        }
        Ok(ret_ty)
    }

    fn check_binary(&mut self, op: BinOp, lhs: &Expr, rhs: &Expr, span: Span) -> Result<TypeId, Diagnostic> {
        let lhs_ty = self.check(lhs)?;
        let rhs_ty = self.check(rhs)?;

        match op {
            BinOp::Eq | BinOp::Ne => {
                if let Err(_) = unify(self.table, lhs_ty, rhs_ty) {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in comparison", span));
                }
                Ok(self.table.intern(Type::Bool))
            }
            BinOp::And | BinOp::Or => {
                let t_bool = self.table.intern(Type::Bool);
                if unify(self.table, lhs_ty, t_bool).is_err() || unify(self.table, rhs_ty, t_bool).is_err() {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch, expected bool", span));
                }
                Ok(t_bool)
            }
            BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge => {
                if unify(self.table, lhs_ty, rhs_ty).is_err() {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in comparison", span));
                }
                let resolved = self.table.resolve(lhs_ty);
                match resolved {
                    Type::Int | Type::Float => {},
                    Type::Var(_) => {
                        let t_int = self.table.intern(Type::Int);
                        let _ = unify(self.table, lhs_ty, t_int);
                    },
                    _ => return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Cannot compare non-numeric types", span)),
                }
                Ok(self.table.intern(Type::Bool))
            }
            BinOp::Add => {
                if unify(self.table, lhs_ty, rhs_ty).is_err() {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in addition", span));
                }
                let resolved = self.table.resolve(lhs_ty);
                match resolved {
                    Type::Int | Type::Float | Type::Str | Type::List(_) => return Ok(lhs_ty),
                    Type::Var(_) => {
                        // If it's a free var, we can't easily guess. Default to Int or leave it.
                        // Actually, wait, we can just leave it as is if it's a Var,
                        // or default it to Int.
                        let t_int = self.table.intern(Type::Int);
                        let _ = unify(self.table, lhs_ty, t_int);
                        return Ok(lhs_ty);
                    }
                    _ => return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Invalid operand for '+'", span)),
                }
            }
            _ => { // Sub, Mul, Div, Rem
                if unify(self.table, lhs_ty, rhs_ty).is_err() {
                    return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Type mismatch in arithmetic", span));
                }
                let resolved = self.table.resolve(lhs_ty);
                match resolved {
                    Type::Int | Type::Float => return Ok(lhs_ty),
                    Type::Var(_) => {
                        let t_int = self.table.intern(Type::Int);
                        let _ = unify(self.table, lhs_ty, t_int);
                        return Ok(lhs_ty);
                    }
                    _ => return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Invalid operand for arithmetic", span)),
                }
            }
        }
    }

    fn check_range(&mut self, from: Option<&Expr>, to: Option<&Expr>, _span: Span) -> Result<TypeId, Diagnostic> {
        let t_int = self.table.intern(Type::Int);
        if let Some(f) = from {
            let f_ty = self.check(f)?;
            if unify(self.table, f_ty, t_int).is_err() {
                return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Range bounds must be integers", f.span));
            }
        }
        if let Some(t) = to {
            let t_ty = self.check(t)?;
            if unify(self.table, t_ty, t_int).is_err() {
                return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Range bounds must be integers", t.span));
            }
        }
        // Assuming Range is represented as an object or just a fresh var for now,
        // Wait, range is a specific builtin type or list of ints?
        // Let's assume list of ints? No, the requirements don't specify the exact type,
        // maybe `List(Int)`?
        Ok(self.table.intern(Type::List(t_int)))
    }

    fn check_lambda(&mut self, params: &[Name], body: &Expr, _span: Span) -> Result<TypeId, Diagnostic> {
        self.scope.push();
        let mut param_tys = Vec::new();
        let mut old_envs = Vec::new();
        for p in params {
            use crate::resolve::SymbolKind;
            use crate::resolve::Symbol;
            self.scope.define(Symbol { name: p.text.clone(), kind: SymbolKind::Param, span: p.span, mutable: true });
            let ty = self.table.fresh_var();
            let old = self.env.insert(p.text.clone(), ty);
            old_envs.push((p.text.clone(), old));
            param_tys.push(ty);
        }
        let ret_ty = self.check(body)?;
        self.scope.pop();
        for (k, old) in old_envs {
            if let Some(v) = old {
                self.env.insert(k, v);
            } else {
                self.env.remove(&k);
            }
        }

        Ok(self.table.intern(Type::Fn { params: param_tys, ret: ret_ty }))
    }

    fn check_if(&mut self, cond: &Expr, then: &Block, elifs: &[(Expr, Block)], else_: Option<&Block>, _span: Span) -> Result<TypeId, Diagnostic> {
        let cond_ty = self.check(cond)?;
        let t_bool = self.table.intern(Type::Bool);
        if unify(self.table, cond_ty, t_bool).is_err() {
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] if condition must be bool", cond.span));
        }

        let ret_ty = self.table.fresh_var();
        self.check_block(then, ret_ty)?;

        for (elif_cond, elif_body) in elifs {
            let elif_cond_ty = self.check(elif_cond)?;
            if unify(self.table, elif_cond_ty, t_bool).is_err() {
                return Err(Diagnostic::new(ErrorKind::Type, "[E0401] elif condition must be bool", elif_cond.span));
            }
            self.check_block(elif_body, ret_ty)?;
        }

        if let Some(e) = else_ {
            self.check_block(e, ret_ty)?;
        }

        Ok(ret_ty)
    }

    fn check_block(&mut self, block: &Block, expected_ty: TypeId) -> Result<(), Diagnostic> {
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
                    let _scrutinee_ty = self.check(scrutinee)?;
                    let arm_ty = self.table.fresh_var();
                    for arm in arms {
                        // Very simplified pattern checking
                        let body_ty = self.table.fresh_var();

                        let _old_span = last_span;

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
    }

    fn check_comprehension(&mut self, expr: &Expr, var: &Name, iter: &Expr, _span: Span) -> Result<TypeId, Diagnostic> {
        let iter_ty = self.check(iter)?;
        let elem_ty = self.table.fresh_var();
        let list_ty = self.table.intern(Type::List(elem_ty));
        if unify(self.table, iter_ty, list_ty).is_err() {
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Comprehension iter must be a list", iter.span));
        }

        self.scope.push();
        use crate::resolve::{Symbol, SymbolKind};
        self.scope.define(Symbol { name: var.text.clone(), kind: SymbolKind::Local, span: var.span, mutable: true });
        let old = self.env.insert(var.text.clone(), elem_ty);

        let yield_ty = self.check(expr)?;
        self.scope.pop();
        if let Some(v) = old {
            self.env.insert(var.text.clone(), v);
        } else {
            self.env.remove(&var.text);
        }

        Ok(self.table.intern(Type::List(yield_ty)))
    }

    fn check_propagate(&mut self, inner: &Expr, _span: Span) -> Result<TypeId, Diagnostic> {
        let inner_ty = self.check(inner)?;
        // inner must be an error union
        let ok_ty = self.table.fresh_var();
        let err_ty = self.table.fresh_var();
        let expected = self.table.intern(Type::ErrorUnion { err: err_ty, ok: ok_ty });
        if unify(self.table, inner_ty, expected).is_err() {
            return Err(Diagnostic::new(ErrorKind::Type, "[E0401] Propagate requires an ErrorUnion", inner.span));
        }
        Ok(ok_ty)
    }

    fn find_best_match(&self, target: &str, candidates: &[String]) -> Option<String> {
        let mut best = None;
        let mut min_dist = usize::MAX;

        for cand in candidates {
            let dist = self.levenshtein(target, cand);
            if dist < min_dist && dist <= 3 { // Threshold
                min_dist = dist;
                best = Some(cand.clone());
            }
        }

        best
    }

    fn levenshtein(&self, a: &str, b: &str) -> usize {
        let mut dp = vec![vec![0; b.len() + 1]; a.len() + 1];
        for i in 0..=a.len() {
            dp[i][0] = i;
        }
        for j in 0..=b.len() {
            dp[0][j] = j;
        }
        for (i, ca) in a.chars().enumerate() {
            for (j, cb) in b.chars().enumerate() {
                let cost = if ca == cb { 0 } else { 1 };
                dp[i + 1][j + 1] = (dp[i][j + 1] + 1).min(dp[i + 1][j] + 1).min(dp[i][j] + cost);
            }
        }
        dp[a.len()][b.len()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;








    #[test]
    fn test_literals() {
        let mut table = TypeTable::new();
        let e = Expr { kind: ExprKind::Int(42), span: Span::new(0, 0) };
        let ty_id = check(&mut table, &e).unwrap();
        assert_eq!(table.resolve(ty_id), Type::Int);
    }

    #[test]
    fn test_failure_1() {
        let mut table = TypeTable::new();
        let mut scope = Scope::new();
        let mut env = HashMap::new();
        let mut checker = ExprChecker::new(&mut table, &mut scope, &mut env);

        let lhs = Box::new(Expr { kind: ExprKind::Int(1), span: Span::new(0, 0) });
        let rhs = Box::new(Expr { kind: ExprKind::Str("x".to_string()), span: Span::new(0, 0) });
        let e = Expr { kind: ExprKind::Binary { op: BinOp::Add, lhs, rhs }, span: Span::new(0, 0) };

        let res = checker.check(&e);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("[E0401]"));
    }

    #[test]
    fn test_failure_2() {
        let mut table = TypeTable::new();
        let mut scope = Scope::new();
        let mut env = HashMap::new();
        let mut checker = ExprChecker::new(&mut table, &mut scope, &mut env);

        let callee = Box::new(Expr { kind: ExprKind::Int(1), span: Span::new(0, 0) });
        let e = Expr { kind: ExprKind::Call { callee, args: vec![] }, span: Span::new(0, 0) };

        let res = checker.check(&e);
        assert!(res.is_err());
        assert!(res.unwrap_err().message.contains("[E0406] Not callable"));
    }

    #[test]
    fn test_lambda_and_call() {
        let mut table = TypeTable::new();
        let mut scope = Scope::new();
        let mut env = HashMap::new();
        let mut checker = ExprChecker::new(&mut table, &mut scope, &mut env);

        // lambda: (x) => x
        let param = Name { text: "x".to_string(), span: Span::new(0, 1) };
        let body = Box::new(Expr { kind: ExprKind::Name(Name { text: "x".to_string(), span: Span::new(0, 1) }), span: Span::new(0, 1) });
        let lam = Expr { kind: ExprKind::Lambda { params: vec![param], body }, span: Span::new(0, 1) };

        let ty_id = checker.check(&lam).unwrap();
        // Since x is assigned a fresh variable, ret should be the same fresh variable.
        let resolved = checker.table.resolve(ty_id);
        if let Type::Fn { params, ret } = resolved {
            assert_eq!(params.len(), 1);
            assert_eq!(checker.table.resolve(params[0]), checker.table.resolve(ret));
        } else {
            panic!("Expected Fn type");
        }

        // Call it with an int
        let call = Expr { kind: ExprKind::Call { callee: Box::new(lam), args: vec![Arg { name: None, value: Expr { kind: ExprKind::Int(42), span: Span::new(0, 1) } }] }, span: Span::new(0, 1) };
        let res_id = checker.check(&call).unwrap();
        assert_eq!(checker.table.resolve(res_id), Type::Int);
    }

    #[test]
    fn test_field_did_you_mean() {
        let mut table = TypeTable::new();
        let mut scope = Scope::new();
        let mut env = HashMap::new();
        let mut checker = ExprChecker::new(&mut table, &mut scope, &mut env);

        // Let's pretend we have a variable `obj` of type `MyObj`
        use crate::resolve::SymbolKind;
        use crate::resolve::Symbol;
        checker.scope.push();
        checker.scope.define(Symbol { name: "obj".to_string(), kind: SymbolKind::Local, span: Span::new(0, 1), mutable: false });
        let obj_ty = checker.table.intern(Type::Named { name: "MyObj".to_string(), args: vec![] });
        checker.env.insert("obj".to_string(), obj_ty);

        // And `MyObj` has a field `hello_world` mapped in env (as `MyObj.hello_world`)
        let t_int = checker.table.intern(Type::Int);
        checker.env.insert("MyObj.hello_world".to_string(), t_int);

        // Try to access `obj.hello_word` (missing 'l')
        let base = Box::new(Expr { kind: ExprKind::Name(Name { text: "obj".to_string(), span: Span::new(0, 1) }), span: Span::new(0, 1) });
        let field = Expr { kind: ExprKind::Field { base, name: Name { text: "hello_word".to_string(), span: Span::new(0, 1) } }, span: Span::new(0, 1) };

        let res = checker.check(&field);
        assert!(res.is_err());
        let err = res.unwrap_err();
        assert!(err.message.contains("Did you mean 'hello_world'"), "Actual: {}", err.message);
    }

}