#[cfg(feature = "std")]
use ink::primitives::Key;
#[cfg(feature = "std")]
use ink::storage::traits::StorageLayout;
#[cfg(feature = "std")]
use ink_metadata::layout::{
    Layout,
    LayoutKey,
    LeafLayout,
};
use primitive_types::U256;
use scale::{
    Decode,
    Encode,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct WrappedU256(U256);

impl From<WrappedU256> for U256 {
    fn from(value: WrappedU256) -> Self {
        value.0
    }
}

impl From<U256> for WrappedU256 {
    fn from(value: U256) -> Self {
        WrappedU256(value)
    }
}

#[cfg(feature = "std")]
impl StorageLayout for WrappedU256 {
    fn layout(key: &Key) -> Layout {
        Layout::Leaf(LeafLayout::from_key::<Self>(LayoutKey::from(key)))
    }
}

macro_rules! construct_from {
    ( $( $type:ident ),* ) => {
        $(
            impl TryFrom<WrappedU256> for $type {
                type Error = &'static str;
                #[inline]
                fn try_from(value: WrappedU256) -> Result<Self, Self::Error> {
                    Self::try_from(value.0)
                }
            }

            impl From<$type> for WrappedU256 {
                fn from(value: $type) -> WrappedU256 {
                    WrappedU256(U256::from(value))
                }
            }
        )*
    };
}

construct_from!(u8, u16, u32, u64, usize, i8, i16, i32, i64);
