#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrimitiveType {
    Void,
    Bool,
    Int,
    Float,
    String,
    Any,
}

#[derive(Debug, Clone)]
pub struct Type {
    pub id: TypeId,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(usize);

#[derive(Debug, Default)]
pub struct TypeRegistry {
    types: Vec<Type>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_primitive("Void", PrimitiveType::Void);
        registry.register_primitive("Bool", PrimitiveType::Bool);
        registry.register_primitive("Int", PrimitiveType::Int);
        registry.register_primitive("Float", PrimitiveType::Float);
        registry.register_primitive("String", PrimitiveType::String);
        registry.register_primitive("Any", PrimitiveType::Any);
        registry
    }

    fn register_primitive(&mut self, name: &str, primitive: PrimitiveType) {
        let id = TypeId(self.types.len());
        self.types.push(Type {
            id,
            name: name.to_string(),
        });
        let _ = primitive;
    }

    pub fn by_name(&self, name: &str) -> Option<&Type> {
        self.types.iter().find(|t| t.name == name)
    }
}
