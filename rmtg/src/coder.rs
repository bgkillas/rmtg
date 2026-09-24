use crate::events::life_counter::CommanderCounter;
use crate::net::Peer;
use bevy_p2p::bitcode::{self, Decode, Encode};
use core::gca;
use enum_map::EnumMap;
pub trait FixedSize: Sized + Copy {
    #[rustc_always_gca]
    const SIZE: usize;
}
#[derive(Encode, Decode)]
#[repr(transparent)]
pub struct DataCoder<T: FixedSize> {
    pub data: [u8; gca!(T::SIZE)],
}
macro_rules! coder {
    ($ty:ty) => {
        impl FixedSize for $ty {
            const SIZE: usize = core::gca!(const { size_of::<$ty>() });
        }
        impl From<&$ty> for DataCoder<$ty> {
            fn from(value: &$ty) -> Self {
                unsafe { std::mem::transmute_copy(value) }
            }
        }
        impl From<DataCoder<$ty>> for $ty {
            fn from(value: DataCoder<$ty>) -> Self {
                unsafe { std::mem::transmute(value) }
            }
        }
    };
}
coder!(EnumMap<Peer, EnumMap<CommanderCounter, i32>>);
