pub type TypeId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Null,
    Never,
    Unit,
    Named { name: String, args: Vec<TypeId> },
    List(TypeId),
    Array(TypeId, usize),
    Optional(TypeId),
    Fn { params: Vec<TypeId>, ret: TypeId },
    ErrorUnion { err: TypeId, ok: TypeId },
    Var(u32),
}

#[derive(Debug, Clone, Default)]
pub struct TypeTable {
    types: Vec<Type>,
    vars: Vec<Option<Type>>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn intern(&mut self, ty: Type) -> TypeId {
        self.types.push(ty);
        self.types.len() - 1
    }

    pub fn ty(&self, id: TypeId) -> &Type {
        &self.types[id]
    }

    pub fn resolve(&self, id: TypeId) -> Type {
        self.types[id].clone()
    }

    pub fn fresh_var(&mut self) -> TypeId {
        let n = self.vars.len() as u32;
        self.vars.push(None);
        self.intern(Type::Var(n))
    }

    pub fn var(&self, v: u32) -> Option<&Type> {
        self.vars[v as usize].as_ref()
    }

    pub fn set_var(&mut self, v: u32, ty: Type) {
        self.vars[v as usize] = Some(ty);
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }
}
