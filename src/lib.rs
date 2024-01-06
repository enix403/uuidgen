pub(crate) mod uuid;
pub(crate) mod uuid_v4;
pub(crate) mod hash_based;
pub mod time_based;
pub mod inspect;

pub mod gen {
    pub use crate::time_based::v1;
    pub use crate::uuid_v4::v4;
    pub use crate::hash_based::{v3, v5};
}

pub use uuid::{Uuid, wellknown};
pub use gen::*;
