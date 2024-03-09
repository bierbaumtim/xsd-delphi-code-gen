use std::collections::HashMap;

pub trait TypeRegistryItem {
    fn get_qualified_name(&self) -> String;
}

/// Stores all types that have been parsed
///
/// This is used to resolve types that are referenced by other types
pub struct TypeRegistry<T>
where
    T: TypeRegistryItem + Sized,
{
    pub types: HashMap<String, T>,
    gen_type_count: i64,
}

impl<T> TypeRegistry<T>
where
    T: TypeRegistryItem + Sized,
{
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
            gen_type_count: 0,
        }
    }

    /// Registers a custom type
    pub fn register_type(&mut self, custom_type: T) {
        let name = custom_type.get_qualified_name();

        self.types.entry(name).or_insert(custom_type);
    }

    /// Generates a unique type name for a anonymous type
    pub fn generate_type_name(&mut self) -> String {
        let name = format!("__Custom_Type_{}__", self.gen_type_count);

        self.gen_type_count += 1;

        name
    }
}
