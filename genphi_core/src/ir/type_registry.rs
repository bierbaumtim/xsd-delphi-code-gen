use std::collections::HashMap;

use crate::ir::{
    IrTypeId,
    type_id_provider::TYPE_ID_PROVIDER,
    types::{DelphiClass, DelphiEnum},
};

pub struct TypeRegistry {
    classes: HashMap<IrTypeId, DelphiClass>,
    enums: HashMap<IrTypeId, DelphiEnum>,
}

impl TypeRegistry {
    pub fn new() -> Self {
        Self {
            classes: HashMap::new(),
            enums: HashMap::new(),
        }
    }

    pub fn build_internal_name(name: &str, source: &str) -> String {
        // Replace spaces and special characters with underscores
        let sanitized_name = name.replace([' ', '-', '.', ':'], "_");
        format!("{}_{}", source, sanitized_name)
    }

    pub fn generate_name(base: &str) -> (IrTypeId, String) {
        let id = TYPE_ID_PROVIDER.next_id();
        let name = format!("Gen{base}_{id}");

        (id, name)
    }

    pub fn register_class(&mut self, class: DelphiClass) -> IrTypeId {
        let id = TYPE_ID_PROVIDER.next_id();

        self.classes.insert(id, class);

        id
    }

    pub fn register_class_with_id(&mut self, id: IrTypeId, class: DelphiClass) {
        self.classes.insert(id, class);
    }

    pub fn get_class(&self, id: &IrTypeId) -> Option<&DelphiClass> {
        self.classes.get(id)
    }

    pub fn get_class_mut(&mut self, id: &IrTypeId) -> Option<&mut DelphiClass> {
        self.classes.get_mut(id)
    }

    pub fn find_class_by_name(&self, name: &str, source: &str) -> Option<&DelphiClass> {
        let internal_name = Self::build_internal_name(name, source);

        self.classes
            .values()
            .find(|c| c.internal_name == internal_name)
    }

    pub fn find_class_type_id_by_name(&self, name: &str, source: &str) -> Option<IrTypeId> {
        let internal_name = Self::build_internal_name(name, source);

        self.classes
            .iter()
            .find(|(_, c)| c.internal_name == internal_name)
            .map(|(id, _)| *id)
    }

    pub fn register_enum(&mut self, enum_: DelphiEnum) -> IrTypeId {
        let id = TYPE_ID_PROVIDER.next_id();

        self.enums.insert(id, enum_);

        id
    }

    pub fn register_enum_with_id(&mut self, id: IrTypeId, enum_: DelphiEnum) {
        self.enums.insert(id, enum_);
    }

    pub fn get_enum(&self, id: &IrTypeId) -> Option<&DelphiEnum> {
        self.enums.get(id)
    }

    pub fn get_enum_mut(&mut self, id: &IrTypeId) -> Option<&mut DelphiEnum> {
        self.enums.get_mut(id)
    }

    pub fn find_enum_by_name(&self, name: &str, source: &str) -> Option<&DelphiEnum> {
        let internal_name = Self::build_internal_name(name, source);

        self.enums
            .values()
            .find(|e| e.internal_name == internal_name)
    }

    pub fn find_enum_type_id_by_name(&self, name: &str, source: &str) -> Option<IrTypeId> {
        let internal_name = Self::build_internal_name(name, source);

        self.enums
            .iter()
            .find(|(_, e)| e.internal_name == internal_name)
            .map(|(id, _)| *id)
    }
}
