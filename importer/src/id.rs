use bitcode::{Decode, Encode};
use std::fmt;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use uuid::Uuid;
#[derive(Default, PartialEq, Clone, Copy, Encode, Decode, Eq, Hash)]
pub struct Id(u128);
impl FromStr for Id {
    type Err = uuid::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(|a| Id(a.as_u128()))
    }
}
impl fmt::Display for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Uuid::from_u128(self.0))
    }
}
impl Debug for Id {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}
