use crate::ast::expr::Arm;
use crate::diag::{Diagnostic, ErrorKind};
use super::ty::{TypeId, TypeTable, Type};
use super::unify::unify;

/// Pattern typing and exhaustiveness (owned by task 14).
pub fn check_matches() -> Vec<Diagnostic> {
    Vec::new()
}

pub struct VariantDef {
    pub name: String,
    pub payload: Vec<TypeId>,
}

/// Checks a match expression.
/// - `scrutinee_ty`: The resolved type of the scrutinee.
/// - `arms`: The arms of the match.
/// - `arm_body_tys`: The types of the arm bodies (must match `arms.len()`).
/// - `variants`: The definitions of the variants for the scrutinee type (in declaration order).
pub fn check_match_expr(
    table: &mut TypeTable,
    scrutinee_ty: TypeId,
    arms: &[Arm],
    arm_body_tys: &[TypeId],
    variants: &[VariantDef],
) -> Result<TypeId, Vec<Diagnostic>> {
    let mut diags = Vec::new();

    // E0500: Check if the scrutinee type is valid
    let scrutinee_resolved = table.resolve(scrutinee_ty);
    match scrutinee_resolved {
        Type::Bool | Type::Optional(_) | Type::ErrorUnion { .. } | Type::Named { .. } => {}
        _ => {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                "E0500",
                arms.first().map(|a| a.span).unwrap_or_default(),
            ).with_help("Scrutinee must be an object/enum-like type, an error union, T?, or bool"));
            return Err(diags);
        }
    }

    // Track missing variants
    let mut missing_variants: Vec<String> = variants.iter().map(|v| v.name.clone()).collect();
    let mut seen_catch_all = false;
    let mut seen_variants: Vec<String> = Vec::new();

    for (i, arm) in arms.iter().enumerate() {
        let pattern_name = &arm.pattern.name.text;

        // E0504: arms after a `_` are unreachable
        if seen_catch_all {
            diags.push(Diagnostic::new(
                ErrorKind::Type,
                "E0504",
                arm.span,
            ).with_help("Unreachable arm"));
            continue;
        }

        if pattern_name == "_" {
            // E0503: `_` is a catch-all, legal only in the last position
            if i != arms.len() - 1 {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    "E0503",
                    arm.pattern.span,
                ).with_help("Catch-all `_` must be in the last position"));
            }
            seen_catch_all = true;
            missing_variants.clear(); // Catch-all covers everything remaining
        } else {
            // Duplicate arm check
            if seen_variants.contains(pattern_name) {
                // Actually the task doesn't specify an error code for duplicate arm, but it's part of pattern shape E0502 or unreachable E0504?
                // Let's use E0504 unreachable arm for duplicate arms since it's unreachable, or E0502.
                // Let's emit E0504 unreachable arm for duplicate as well. Wait, "arms after a _ are unreachable — report E0504 for each". Duplicate arm is also unreachable.
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    "E0504",
                    arm.pattern.span,
                ).with_help(format!("Duplicate arm `{}` is unreachable", pattern_name)));
            } else {
                seen_variants.push(pattern_name.clone());
            }

            // Find variant
            if let Some(pos) = missing_variants.iter().position(|v| v == pattern_name) {
                missing_variants.remove(pos);
            }

            // E0502: wrong arity or wrong pattern shape
            let expected_payload = variants.iter().find(|v| v.name == *pattern_name).map(|v| &v.payload);
            if let Some(payload) = expected_payload {
                if payload.len() != arm.pattern.binds.len() {
                    diags.push(Diagnostic::new(
                        ErrorKind::Type,
                        "E0502",
                        arm.pattern.span,
                    ).with_help(format!("Wrong arity: variant `{}` expects {} bindings", pattern_name, payload.len())));
                }
            } else {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    "E0502",
                    arm.pattern.span,
                ).with_help(format!("Wrong pattern shape: variant `{}` not found", pattern_name)));
            }
        }
    }

    // E0501: missing variants
    if !missing_variants.is_empty() {
        // Find them in declaration order
        let mut ordered_missing = Vec::new();
        for v in variants {
            if missing_variants.contains(&v.name) {
                ordered_missing.push(v.name.clone());
            }
        }
        let missing_str = ordered_missing.join(", ");
        diags.push(Diagnostic::new(
            ErrorKind::Type,
            format!("E0501: {}", missing_str),
            arms.first().map(|a| a.span).unwrap_or_default(),
        ).with_help("Missing variants"));
    }

    // E0505: unify arm bodies
    let mut unified_ty = table.fresh_var();
    if !arm_body_tys.is_empty() {
        unified_ty = arm_body_tys[0];
        for (i, &body_ty) in arm_body_tys.iter().enumerate().skip(1) {
            if let Err(_) = unify(table, unified_ty, body_ty) {
                diags.push(Diagnostic::new(
                    ErrorKind::Type,
                    format!("E0505: {} vs {}", table.display(&table.resolve(unified_ty)), table.display(&table.resolve(body_ty))),
                    arms[i].span, // Ideally the body's span, but arm's span is acceptable
                ).with_help("Arm body type mismatch"));
            }
        }
    }

    if diags.is_empty() {
        Ok(unified_ty)
    } else {
        Err(diags)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::expr::Pattern;
    use crate::ast::item::Name;
    use crate::diag::Span;
    use crate::ast::expr::Stmt;
    use crate::ast::expr::StmtKind;
    use crate::ast::expr::Expr;
    use crate::ast::expr::ExprKind;

    fn mk_span() -> Span { Span::new(0, 1) }
    fn mk_name(text: &str) -> Name { Name { text: text.to_string(), span: mk_span() } }

    fn mk_arm(pat_name: &str, binds: usize) -> Arm {
        let mut bind_names = Vec::new();
        for i in 0..binds {
            bind_names.push(mk_name(&format!("v{}", i)));
        }
        Arm {
            pattern: Pattern {
                name: mk_name(pat_name),
                binds: bind_names,
                span: mk_span(),
            },
            body: Stmt { kind: StmtKind::Expr(Expr { kind: ExprKind::Null, span: mk_span() }), span: mk_span() },
            span: mk_span(),
        }
    }

    #[test]
    fn test_exhaustive_ok() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("true", 0), mk_arm("false", 0)];
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        assert!(res.is_ok());
    }

    #[test]
    fn test_missing_one_variant() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("true", 0)];
        let body_tys = vec![table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0501"));
        assert!(diags[0].message.contains("false"));
    }

    #[test]
    fn test_missing_several() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![];
        let body_tys = vec![];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0501"));
        assert!(diags[0].message.contains("true, false"));
    }

    #[test]
    fn test_duplicate_arm() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("true", 0), mk_arm("true", 0), mk_arm("false", 0)];
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Int), table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0504"));
    }

    #[test]
    fn test_wrong_payload_arity() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("true", 1), mk_arm("false", 0)]; // `true` arm binds 1 payload
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] }, // expects 0 payload
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0502"));
    }

    #[test]
    fn test_catch_all_not_last() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("_", 0), mk_arm("true", 0)];
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 2);
        assert!(diags.iter().any(|d| d.message.contains("E0503")));
        assert!(diags.iter().any(|d| d.message.contains("E0504")));
    }

    #[test]
    fn test_unreachable_arm() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let _arms = vec![mk_arm("true", 0), mk_arm("false", 0), mk_arm("_", 0)];
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Int), table.intern(Type::Int)];
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        // Actually, arms after `_` are unreachable. E0504
        let arms2 = vec![mk_arm("true", 0), mk_arm("_", 0), mk_arm("false", 0)];
        let res = check_match_expr(&mut table, scrutinee_ty, &arms2, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 2); // 1 for not last (_), 1 for unreachable arm (false)
        assert!(diags.iter().any(|d| d.message.contains("E0504")));
    }

    #[test]
    fn test_unifying_arms() {
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Bool);
        let arms = vec![mk_arm("true", 0), mk_arm("false", 0)];
        let body_tys = vec![table.intern(Type::Int), table.intern(Type::Float)]; // Mismatch
        let variants = vec![
            VariantDef { name: "true".to_string(), payload: vec![] },
            VariantDef { name: "false".to_string(), payload: vec![] },
        ];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0505"));
    }

    #[test]
    fn test_non_exhaustive_no_variants() {
        // test E0500
        let mut table = TypeTable::new();
        let scrutinee_ty = table.intern(Type::Int); // Invalid scrutinee type
        let arms = vec![];
        let body_tys = vec![];
        let variants = vec![];

        let res = check_match_expr(&mut table, scrutinee_ty, &arms, &body_tys, &variants);
        let diags = res.unwrap_err();
        assert_eq!(diags.len(), 1);
        assert!(diags[0].message.contains("E0500"));
    }
}
