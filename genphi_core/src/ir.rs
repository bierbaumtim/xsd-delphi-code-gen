pub mod type_id_provider;
pub mod type_registry;
pub mod types;

/// Unique identifier for a type.
pub type IrTypeId = usize;

pub trait IrLookupType {
    fn set_id(&mut self, id: IrTypeId);
    fn get_id(&self) -> IrTypeId;
    fn get_lookup_name(&self) -> &String;
}
