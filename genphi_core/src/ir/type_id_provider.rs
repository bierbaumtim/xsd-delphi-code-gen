use std::sync::{
    LazyLock,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};

use crate::ir::IrTypeId;

pub const UNRESOLVED_TYPE_ID: IrTypeId = 0;

static IR_TYPE_ID: LazyLock<AtomicUsize> = LazyLock::new(|| AtomicUsize::new(0));
static TYPE_ID_PROVIDER_INITIALIZED: AtomicBool = AtomicBool::new(false);
pub static TYPE_ID_PROVIDER: LazyLock<TypeIdProvider> = LazyLock::new(TypeIdProvider::new);

pub struct TypeIdProvider;

impl TypeIdProvider {
    fn new() -> Self {
        if TYPE_ID_PROVIDER_INITIALIZED.load(Ordering::SeqCst) {
            panic!("TypeIdProvider already initialized");
        }

        IR_TYPE_ID.store(0, Ordering::SeqCst);
        TYPE_ID_PROVIDER_INITIALIZED.store(true, Ordering::SeqCst);

        Self
    }

    pub fn next_id(&self) -> IrTypeId {
        let last_used_id = IR_TYPE_ID.fetch_add(1, Ordering::SeqCst);

        let new_id = IR_TYPE_ID.load(Ordering::SeqCst);

        if last_used_id == new_id || new_id == 0 {
            panic!("Ran out of IDs");
        }

        new_id as IrTypeId
    }
}
