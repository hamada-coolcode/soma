use crate::ast::expr::{BinOp, Block, Expr, ExprKind, Stmt, StmtKind, UnOp};
use crate::ast::item::{Item, ItemKind};
use crate::ast::ty::{TypeKind, TypeRepr};
use crate::diag::{Diagnostic, ErrorKind};
use crate::lexer::token::FPart;
use std::collections::HashSet;
use std::fmt::Write as _;

struct Ctx {
    out: String,
    declared_vars: HashSet<String>,
}

impl Ctx {
    fn new() -> Self {
        Self {
            out: String::new(),
            declared_vars: HashSet::new(),
        }
    }

    fn push(&mut self, s: &str) {
        self.out.push_str(s);
    }

    fn format_type(&self, ty: &TypeRepr) -> Result<String, Diagnostic> {
        match &ty.kind {
            TypeKind::Named { name, generics } => {
                if !generics.is_empty() {
                    return Err(Diagnostic::new(
                        ErrorKind::Other,
                        "E1001: Generics unsupported",
                        ty.span.clone(),
                    ));
                }
                match name.text.as_str() {
                    "int" | "uint" => Ok("long long".to_string()),
                    "float" => Ok("double".to_string()),
                    "bool" => Ok("bool".to_string()),
                    "string" => Ok("const char*".to_string()),
                    other => Ok(format!("{}*", other)), // Object pointer
                }
            }
            TypeKind::Never => Ok("void".to_string()),
            _ => Err(Diagnostic::new(
                ErrorKind::Other,
                "E1001: Unsupported type",
                ty.span.clone(),
            )),
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<(), Diagnostic> {
        match &expr.kind {
            ExprKind::Int(v) => write!(self.out, "{}LL", v).unwrap(),
            ExprKind::Float(v) => write!(self.out, "{}", v).unwrap(),
            ExprKind::Bool(b) => write!(self.out, "{}", if *b { "true" } else { "false" }).unwrap(),
            ExprKind::Null => self.push("NULL"),
            ExprKind::Name(n) => {
                if n.text == "output" {
                    self.push("soma_print");
                } else {
                    self.push(&n.text);
                }
            },
            ExprKind::SelfExpr => self.push("self"),
            ExprKind::Str(s) => {
                let escaped = s.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n");
                write!(self.out, "\"{}\"", escaped).unwrap();
            }
            ExprKind::FStr(parts) => {
                if parts.is_empty() {
                    self.push("\"\"");
                } else {
                    for _ in 0..(parts.len() - 1) {
                        self.push("soma_str_concat(");
                    }
                    for (i, part) in parts.iter().enumerate() {
                        if i > 0 {
                            self.push(", ");
                        }
                        match part {
                            FPart::Text(s) => {
                                let escaped = s.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n");
                                write!(self.out, "\"{}\"", escaped).unwrap();
                            }
                            FPart::Expr(_) => {
                                return Err(Diagnostic::new(
                                    ErrorKind::Other,
                                    "E1001: FStr with Expr unsupported",
                                    expr.span.clone(),
                                ));
                            }
                        }
                        if i > 0 {
                            self.push(")");
                        }
                    }
                }
            }
            ExprKind::Call { callee, args } => {
                // io.term.output
                if let ExprKind::Field { base, name } = &callee.kind {
                    if let ExprKind::Field { base: io_base, name: term_name } = &base.kind {
                        if let ExprKind::Name(io_name) = &io_base.kind {
                            if io_name.text == "io" && term_name.text == "term" {
                                if name.text == "output" {
                                    self.push("soma_print(");
                                    if args.is_empty() {
                                        self.push("\"\"");
                                    } else {
                                        self.emit_expr(&args[0].value)?;
                                    }
                                    self.push(")");
                                    return Ok(());
                                } else if name.text == "input" {
                                    self.push("soma_getline()");
                                    return Ok(());
                                }
                            }
                        }
                    }
                }

                // if it's `output(...)`
                if let ExprKind::Name(n) = &callee.kind {
                    if n.text == "output" {
                        self.push("soma_print(");
                        if args.is_empty() {
                            self.push("\"\"");
                        } else {
                            self.emit_expr(&args[0].value)?;
                        }
                        self.push(")");
                        return Ok(());
                    }
                }

                // Check method call
                if let ExprKind::Field { base, name } = &callee.kind {
                    self.push(&name.text);
                    self.push("(");
                    self.emit_expr(base)?;
                    for arg in args {
                        self.push(", ");
                        self.emit_expr(&arg.value)?;
                    }
                    self.push(")");
                    return Ok(());
                }

                self.emit_expr(callee)?;
                self.push("(");
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.push(", ");
                    }
                    self.emit_expr(&arg.value)?;
                }
                self.push(")");
            }
            ExprKind::Field { base, name } => {
                self.emit_expr(base)?;
                write!(self.out, "->{}", name.text).unwrap();
            }
            ExprKind::Index { base, index } => {
                self.emit_expr(base)?;
                self.push("[");
                self.emit_expr(index)?;
                self.push("]");
            }
            ExprKind::Unary { op, operand } => {
                let op_str = match op {
                    UnOp::Neg => "-",
                    UnOp::Not => "!",
                };
                write!(self.out, "({}", op_str).unwrap();
                self.emit_expr(operand)?;
                self.push(")");
            }
            ExprKind::Binary { op, lhs, rhs } => {
                self.push("(");
                self.emit_expr(lhs)?;
                let op_str = match op {
                    BinOp::Add => " + ",
                    BinOp::Sub => " - ",
                    BinOp::Mul => " * ",
                    BinOp::Div => " / ",
                    BinOp::Rem => " % ",
                    BinOp::Eq => " == ",
                    BinOp::Ne => " != ",
                    BinOp::Lt => " < ",
                    BinOp::Gt => " > ",
                    BinOp::Le => " <= ",
                    BinOp::Ge => " >= ",
                    BinOp::And => " && ",
                    BinOp::Or => " || ",
                };
                self.push(op_str);
                self.emit_expr(rhs)?;
                self.push(")");
            }
            ExprKind::Propagate(_) => {
                self.push("/* TODO(task 26): Error unwinding unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: Error unwinding unsupported",
                    expr.span.clone(),
                ));
            }
            ExprKind::Lambda { .. } => {
                self.push("/* TODO(task 26): Closures unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: Closures unsupported",
                    expr.span.clone(),
                ));
            }
            ExprKind::Comprehension { .. } => {
                self.push("/* TODO(task 26): Comprehension unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: Comprehension unsupported",
                    expr.span.clone(),
                ));
            }
            _ => {
                self.push("/* TODO(task 26): Unsupported expression */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    format!("E1001: Unsupported expression: {:?}", expr.kind),
                    expr.span.clone(),
                ));
            }
        }
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt, indent: usize) -> Result<(), Diagnostic> {
        let ind = "    ".repeat(indent);
        self.push(&ind);
        match &stmt.kind {
            StmtKind::Expr(e) => {
                self.emit_expr(e)?;
                self.push(";\n");
            }
            StmtKind::Assign { target, value } => {
                if let ExprKind::Name(n) = &target.kind {
                    let var_name = n.text.clone();
                    if self.declared_vars.insert(var_name.clone()) {
                        write!(self.out, "__auto_type {} = ", var_name).unwrap();
                    } else {
                        write!(self.out, "{} = ", var_name).unwrap();
                    }
                    self.emit_expr(value)?;
                    self.push(";\n");
                } else {
                    self.emit_expr(target)?;
                    self.push(" = ");
                    self.emit_expr(value)?;
                    self.push(";\n");
                }
            }
            StmtKind::Return(e) => {
                if let Some(e) = e {
                    self.push("return ");
                    self.emit_expr(e)?;
                    self.push(";\n");
                } else {
                    self.push("return;\n");
                }
            }
            StmtKind::While { cond, body } => {
                self.push("while (");
                self.emit_expr(cond)?;
                self.push(") ");
                self.emit_block(body, indent, false)?;
                self.push("\n");
            }
            StmtKind::For { .. } => {
                self.push("/* TODO(task 26): For loop unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: For loops desugared usually but unsupported here",
                    stmt.span.clone(),
                ));
            }
            StmtKind::Break => self.push("break;\n"),
            StmtKind::Continue => self.push("continue;\n"),
            StmtKind::Match { .. } => {
                self.push("/* TODO(task 26): Match unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: Match unsupported here",
                    stmt.span.clone(),
                ));
            }
            StmtKind::Loop { .. } => {
                self.push("/* TODO(task 26): Loop unsupported */");
                return Err(Diagnostic::new(
                    ErrorKind::Other,
                    "E1001: Loop unsupported here",
                    stmt.span.clone(),
                ));
            }
            StmtKind::Item(_) => {} // Nested items
        }
        Ok(())
    }

    fn emit_block(&mut self, block: &Block, indent: usize, is_main: bool) -> Result<(), Diagnostic> {
        self.push("{\n");
        for stmt in &block.stmts {
            self.emit_stmt(stmt, indent + 1)?;
        }
        if is_main {
            let ind = "    ".repeat(indent + 1);
            self.push(&ind);
            self.push("return 0;\n");
        }
        let ind = "    ".repeat(indent);
        self.push(&ind);
        self.push("}");
        Ok(())
    }
}

pub fn emit(items: &[Item]) -> String {
    emit_checked(items).unwrap_or_else(|_| String::new())
}

pub fn emit_checked(items: &[Item]) -> Result<String, Diagnostic> {
    let mut ctx = Ctx::new();

    if items.is_empty() {
        ctx.push("#include <stdio.h>\n");
        ctx.push("#include <stdlib.h>\n");
        ctx.push("#include <string.h>\n");
        ctx.push("#include <stdbool.h>\n");
        ctx.push("\n");
        ctx.push("static char arena[1024 * 1024];\n");
        ctx.push("static size_t arena_offset = 0;\n");
        ctx.push("void* soma_alloc(size_t size) {\n");
        ctx.push("    if (arena_offset + size > sizeof(arena)) { exit(1); }\n");
        ctx.push("    void* ptr = &arena[arena_offset];\n");
        ctx.push("    arena_offset += size;\n");
        ctx.push("    return ptr;\n");
        ctx.push("}\n");
        ctx.push("const char* soma_str_concat(const char* a, const char* b) {\n");
        ctx.push("    size_t len_a = strlen(a);\n");
        ctx.push("    size_t len_b = strlen(b);\n");
        ctx.push("    char* res = (char*)soma_alloc(len_a + len_b + 1);\n");
        ctx.push("    strcpy(res, a);\n");
        ctx.push("    strcat(res, b);\n");
        ctx.push("    return res;\n");
        ctx.push("}\n");
        ctx.push("const char* soma_int_to_str(long long v) {\n");
        ctx.push("    char* res = (char*)soma_alloc(32);\n");
        ctx.push("    snprintf(res, 32, \"%lld\", v);\n");
        ctx.push("    return res;\n");
        ctx.push("}\n");
        ctx.push("#define soma_print(x) _Generic((x), char*: printf(\"%s\\n\", x), const char*: printf(\"%s\\n\", x), long long: printf(\"%lld\\n\", x), double: printf(\"%f\\n\", x), bool: printf(\"%s\\n\", (x)?\"true\":\"false\"), default: printf(\"%p\\n\", (void*)x))\n");
        ctx.push("#define soma_to_str(x) _Generic((x), char*: (const char*)x, const char*: x, long long: soma_int_to_str(x))\n");
        ctx.push("\n");
        ctx.push("int main(void) {\n    return 0;\n}\n");
        return Ok(ctx.out);
    }

    // 1. Prelude
    ctx.push("#include <stdio.h>\n");
    ctx.push("#include <stdlib.h>\n");
    ctx.push("#include <string.h>\n");
    ctx.push("#include <stdbool.h>\n");
    ctx.push("\n");

    ctx.push("static char arena[1024 * 1024];\n");
    ctx.push("static size_t arena_offset = 0;\n");
    ctx.push("void* soma_alloc(size_t size) {\n");
    ctx.push("    if (arena_offset + size > sizeof(arena)) { exit(1); }\n");
    ctx.push("    void* ptr = &arena[arena_offset];\n");
    ctx.push("    arena_offset += size;\n");
    ctx.push("    return ptr;\n");
    ctx.push("}\n");
    ctx.push("const char* soma_str_concat(const char* a, const char* b) {\n");
    ctx.push("    size_t len_a = strlen(a);\n");
    ctx.push("    size_t len_b = strlen(b);\n");
    ctx.push("    char* res = (char*)soma_alloc(len_a + len_b + 1);\n");
    ctx.push("    strcpy(res, a);\n");
    ctx.push("    strcat(res, b);\n");
    ctx.push("    return res;\n");
    ctx.push("}\n");
    ctx.push("const char* soma_int_to_str(long long v) {\n");
    ctx.push("    char* res = (char*)soma_alloc(32);\n");
    ctx.push("    snprintf(res, 32, \"%lld\", v);\n");
    ctx.push("    return res;\n");
    ctx.push("}\n");
    ctx.push("#define soma_print(x) _Generic((x), char*: printf(\"%s\\n\", x), const char*: printf(\"%s\\n\", x), long long: printf(\"%lld\\n\", x), double: printf(\"%f\\n\", x), bool: printf(\"%s\\n\", (x)?\"true\":\"false\"), default: printf(\"%p\\n\", (void*)x))\n");
    ctx.push("#define soma_to_str(x) _Generic((x), char*: (const char*)x, const char*: x, long long: soma_int_to_str(x))\n");

    // Gather methods for _Generic macros
    let mut methods: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

    // 2. Forward Declarations
    for item in items {
        match &item.kind {
            ItemKind::Object(obj) => {
                writeln!(ctx.out, "typedef struct {0} {0};", obj.name.text).unwrap();
                writeln!(ctx.out, "struct {} {{", obj.name.text).unwrap();
                for field in &obj.fields {
                    let ty_str = ctx.format_type(&field.ty)?;
                    writeln!(ctx.out, "    {} {};", ty_str, field.name.text).unwrap();
                }
                ctx.push("};\n");

                for method in &obj.methods {
                    let m_name = format!("{}_{}", obj.name.text, method.name.text);
                    methods.entry(method.name.text.clone()).or_default().push(format!("{}*: {}", obj.name.text, m_name));
                    let ret_ty = method.ret.as_ref().map_or(Ok("void".to_string()), |t| ctx.format_type(t))?;
                    write!(ctx.out, "{} {}(", ret_ty, m_name).unwrap();
                    for (i, param) in method.params.iter().enumerate() {
                        if i > 0 { ctx.push(", "); }
                        let param_ty = ctx.format_type(&param.ty)?;
                        write!(ctx.out, "{} {}", param_ty, param.name.text).unwrap();
                    }
                    if method.params.is_empty() {
                        ctx.push("void");
                    }
                    ctx.push(");\n");
                }
            }
            ItemKind::Fn(func) => {
                if func.name.text == "main" {
                    ctx.push("int main(void);\n");
                } else {
                    let ret_ty = func.ret.as_ref().map_or(Ok("void".to_string()), |t| ctx.format_type(t))?;
                    write!(ctx.out, "{} {}(", ret_ty, func.name.text).unwrap();
                    for (i, param) in func.params.iter().enumerate() {
                        if i > 0 { ctx.push(", "); }
                        let param_ty = ctx.format_type(&param.ty)?;
                        write!(ctx.out, "{} {}", param_ty, param.name.text).unwrap();
                    }
                    if func.params.is_empty() {
                        ctx.push("void");
                    }
                    ctx.push(");\n");
                }
            }
            _ => {}
        }
    }

    // Macro definitions
    for (m_name, variants) in methods {
        writeln!(ctx.out, "#define {}(x, ...) _Generic((x), {})(x, ##__VA_ARGS__)", m_name, variants.join(", ")).unwrap();
    }
    ctx.push("\n");

    // 3. Definitions
    for item in items {
        if let ItemKind::Fn(func) = &item.kind {
            if func.name.text == "main" {
                ctx.push("int main(void) ");
                ctx.declared_vars.clear();
                ctx.emit_block(&func.body, 0, true)?;
                ctx.push("\n\n");
            } else {
                let ret_ty = func.ret.as_ref().map_or(Ok("void".to_string()), |t| ctx.format_type(t))?;
                write!(ctx.out, "{} {}(", ret_ty, func.name.text).unwrap();
                for (i, param) in func.params.iter().enumerate() {
                    if i > 0 { ctx.push(", "); }
                    let param_ty = ctx.format_type(&param.ty)?;
                    write!(ctx.out, "{} {}", param_ty, param.name.text).unwrap();
                }
                if func.params.is_empty() {
                    ctx.push("void");
                }
                ctx.push(") ");
                ctx.declared_vars.clear();
                // Populate param names
                for param in &func.params {
                    ctx.declared_vars.insert(param.name.text.clone());
                }
                ctx.emit_block(&func.body, 0, false)?;
                ctx.push("\n\n");
            }
        } else if let ItemKind::Object(obj) = &item.kind {
            for method in &obj.methods {
                let ret_ty = method.ret.as_ref().map_or(Ok("void".to_string()), |t| ctx.format_type(t))?;
                let m_name = format!("{}_{}", obj.name.text, method.name.text);
                write!(ctx.out, "{} {}(", ret_ty, m_name).unwrap();
                for (i, param) in method.params.iter().enumerate() {
                    if i > 0 { ctx.push(", "); }
                    let param_ty = ctx.format_type(&param.ty)?;
                    write!(ctx.out, "{} {}", param_ty, param.name.text).unwrap();
                }
                if method.params.is_empty() {
                    ctx.push("void");
                }
                ctx.push(") ");
                ctx.declared_vars.clear();
                for param in &method.params {
                    ctx.declared_vars.insert(param.name.text.clone());
                }
                ctx.emit_block(&method.body, 0, false)?;
                ctx.push("\n\n");
            }
        }
    }

    Ok(ctx.out)
}
