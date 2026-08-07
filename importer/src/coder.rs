use crate::card::{Color, MainType, SubType, SuperType};
use bevy::math::Vec3;
use bitcode::{Decode, Encode};
use core::direct_const_arg;
use enumset::EnumSet;
use std::mem;
pub trait FixedSize: Sized + Copy {
    type const SIZE: usize;
}
#[derive(Encode, Decode)]
#[repr(transparent)]
pub struct DataCoder<T: FixedSize> {
    pub data: [u8; direct_const_arg!(T::SIZE)],
}
macro_rules! coder {
    ($ty:ty) => {
        impl FixedSize for $ty {
            type const SIZE: usize = const { size_of::<$ty>() };
        }
        impl From<&$ty> for DataCoder<$ty> {
            fn from(value: &$ty) -> Self {
                unsafe { mem::transmute_copy(value) }
            }
        }
        impl From<DataCoder<$ty>> for $ty {
            fn from(value: DataCoder<$ty>) -> Self {
                unsafe { mem::transmute(value) }
            }
        }
    };
}
coder!(EnumSet<SuperType>);
coder!(EnumSet<MainType>);
coder!(EnumSet<SubType>);
coder!(EnumSet<Color>);
coder!(Vec3);
