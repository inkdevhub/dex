use primitive_types::U256;

#[cfg(feature = "std")]
use ink_metadata::layout::{
    CellLayout,
    Layout,
    LayoutKey,
};
use ink_primitives::KeyPtr;
use ink_storage::traits::{
    ExtKeyPtr,
    PackedLayout,
    SpreadAllocate,
    SpreadLayout,
};

#[cfg(feature = "std")]
use ink_storage::traits::StorageLayout;
use scale::{
    Decode,
    Encode,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct WrappedU256(U256);

impl SpreadLayout for WrappedU256 {
    const FOOTPRINT: u64 = 4;
    const REQUIRES_DEEP_CLEAN_UP: bool = true;
    fn pull_spread(ptr: &mut ink_primitives::KeyPtr) -> Self {
        let slice: [u64; 4] = SpreadLayout::pull_spread(ptr);
        Self(U256(slice))
    }

    fn push_spread(&self, ptr: &mut ink_primitives::KeyPtr) {
        SpreadLayout::push_spread(&self.0 .0, ptr);
    }

    fn clear_spread(&self, ptr: &mut ink_primitives::KeyPtr) {
        SpreadLayout::clear_spread(&self.0 .0, ptr);
    }
}

impl PackedLayout for WrappedU256 {
    fn pull_packed(&mut self, at: &ink_primitives::Key) {
        self.0 .0.pull_packed(at);
    }
    fn push_packed(&self, at: &ink_primitives::Key) {
        self.0 .0.push_packed(at);
    }
    fn clear_packed(&self, at: &ink_primitives::Key) {
        self.0 .0.clear_packed(at);
    }
}

impl SpreadAllocate for WrappedU256 {
    fn allocate_spread(ptr: &mut KeyPtr) -> Self {
        ptr.next_for::<WrappedU256>();
        WrappedU256::default()
    }
}

#[cfg(feature = "std")]
impl StorageLayout for WrappedU256 {
    fn layout(key_ptr: &mut KeyPtr) -> Layout {
        Layout::Cell(CellLayout::new::<WrappedU256>(LayoutKey::from(
            key_ptr.advance_by(1),
        )))
    }
}

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
