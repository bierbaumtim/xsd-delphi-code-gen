use std::collections::HashMap;

use crate::parser_types::*;

pub(crate) struct TypeRegistry {
    pub(crate) types: HashMap<String, CustomTypeDefinition>,
    gen_type_count: i64,
}

impl TypeRegistry {
    pub(crate) fn new() -> TypeRegistry {
        TypeRegistry {
            types: HashMap::new(),
            gen_type_count: 0,
        }
    }

    pub(crate) fn register_type(&mut self, custom_type: CustomTypeDefinition) {
        let name = custom_type.get_qualified_name();

        if !self.types.contains_key(&name) {
            self.types.insert(name, custom_type);
        }
    }

    pub(crate) fn generate_type_name(&mut self) -> String {
        let name = format!("__Custom_Type_{}__", self.gen_type_count);

        self.gen_type_count += 1;

        name
    }
}
