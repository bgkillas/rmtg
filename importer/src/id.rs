use bitcode::{Decode, Encode};
use uuid::Uuid;
#[derive(Debug, Default, PartialEq, Clone, Copy, Encode, Decode, Eq, Hash)]
pub struct Id {
    #[bitcode(with = "IdCoder")]
    pub id: Uuid,
}
#[derive(Debug, Default, PartialEq, Clone, Copy, Encode, Decode, Eq, Hash)]
struct IdCoder {
    bytes: u128,
}
impl From<&Uuid> for IdCoder {
    fn from(value: &Uuid) -> Self {
        Self {
            bytes: value.as_u128(),
        }
    }
}
impl From<Uuid> for Id {
    fn from(id: Uuid) -> Self {
        Self { id }
    }
}
impl From<IdCoder> for Uuid {
    fn from(value: IdCoder) -> Self {
        Self::from_u128(value.bytes)
    }
}
