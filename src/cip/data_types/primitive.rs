macro_rules! impl_cip_primitive {
    ($name:ident, u8) => { impl_cip_primitive!(@gen $name, u8, get_u8, put_u8); };
    ($name:ident, i8) => { impl_cip_primitive!(@gen $name, i8, get_i8, put_i8); };
    ($name:ident, $type:ident) => {
        paste::paste! {
            impl_cip_primitive!(@gen $name, $type, [<get_ $type _le>], [<put_ $type _le>]);
        }
    };

    (@gen $name:ident, $type:ty, $get_fn:ident, $put_fn:ident) => {
        #[repr(transparent)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
        pub struct $name($type);

        impl $name {
            pub const LEN: usize = std::mem::size_of::<$type>();
            const BITS: u8 = (Self::LEN * 8) as u8;

            pub fn new(value: $type) -> Self {
                Self(value)
            }

            pub fn value(&self) -> $type {
                self.0
            }

            pub fn get_bit(&self, index: u8) -> bool {
                if index >= Self::BITS {
                    return false;
                }

                (self.0 as u64 & (1u64 << index)) != 0
            }

            pub fn set_bit(&mut self, index: u8) {
                if index < Self::BITS {
                    self.0 |= (1u64 << index) as $type;
                }
            }

            pub fn clear_bit(&mut self, index: u8) {
                if index < Self::BITS {
                    self.0 &= !(1u64 << index) as $type;
                }
            }

            pub fn toggle_bit(&mut self, index: u8) {
                if index < Self::BITS {
                    self.0 ^= (1u64 << index) as $type;
                }
            }

            pub fn get_bits(&self, start: u8, end: u8) -> $type {
                if start > end || start >= Self::BITS { return 0; }

                let len = (end - start + 1).min(Self::BITS - start);
                let mask = (1u64 << len) - 1;

                ((self.0 as u64 >> start) & mask) as $type
            }

            pub fn set_bits(&mut self, start: u8, end: u8, value: $type) {
                if start > end || start >= Self::BITS { return; }

                let len = (end - start + 1).min(Self::BITS - start);
                let mask = (1u64 << len) - 1;
                let clear_mask = !(mask << start);

                self.0 = ((self.0 as u64 & clear_mask) | ((value as u64 & mask) << start)) as $type;
            }

            pub fn clear_bits(&mut self, start: u8, end: u8) {
                if start > end || start >= Self::BITS { return; }

                let len = (end - start + 1).min(Self::BITS - start);
                let mask = (1u64 << len) - 1;

                self.0 &= !(mask << start) as $type;
            }

            pub fn toggle_bits(&mut self, start: u8, end: u8) {
                if start > end || start >= Self::BITS { return; }

                let len = (end - start + 1).min(Self::BITS - start);
                let mask = (1u64 << len) - 1;

                self.0 ^= (mask << start) as $type;
            }
        }

        impl FromBytes for $name {
            fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
                if buffer.remaining() < Self::LEN {
                    return Err(BinaryError::Truncated {
                        expected: Self::LEN,
                        actual: buffer.remaining(),
                    });
                }
                let value = buffer.$get_fn();
                Ok(Self(value))
            }
        }

        impl ToBytes for $name {
            fn encode<T: BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
                if buffer.remaining_mut() < Self::LEN {
                    return Err(BinaryError::BufferTooSmall {
                        expected: Self::LEN,
                        actual: buffer.remaining_mut(),
                    });
                }
                buffer.$put_fn(self.0);
                Ok(())
            }

            fn encoded_len(&self) -> usize { Self::LEN }
        }
    };
}
