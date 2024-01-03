pub(crate) mod uuid;
pub(crate) mod uuid_v4;
pub(crate) mod hash_based;
pub(crate) mod time_based;

pub mod gen {
    pub use crate::uuid_v4::v4;
    pub use crate::hash_based::{v3, v5};
}


pub use uuid::{Uuid, wellknown};
pub use gen::*;

// temp
pub use time_based::*;