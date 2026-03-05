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
        #[allow(dead_code)]
        pub struct $name($type);

        #[allow(dead_code)]
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

#[cfg(test)]
mod tests {
    use crate::common::binary::{BinaryError, FromBytes, ToBytes};
    use bytes::{Buf, BufMut, Bytes, BytesMut};

    impl_cip_primitive!(TestByte, u8);
    impl_cip_primitive!(TestWord, u16);

    #[test]
    fn bit_manipulation_single_bits_behave_correctly() {
        let mut val = TestByte::new(0); // 0000 0000
        assert!(!val.get_bit(0));

        val.set_bit(0); // 0000 0001
        assert!(val.get_bit(0));
        assert_eq!(val.value(), 1);

        val.set_bit(7); // 1000 0001
        assert!(val.get_bit(7));
        assert_eq!(val.value(), 0x81);

        val.toggle_bit(0); // 1000 0000
        assert!(!val.get_bit(0));
        assert_eq!(val.value(), 0x80);

        val.clear_bit(7); // 0000 0000
        assert!(!val.get_bit(7));
        assert_eq!(val.value(), 0);
    }

    #[test]
    fn bit_manipulation_ranges_behave_correctly() {
        let mut val = TestByte::new(0);

        val.set_bits(0, 3, 0xF); // 0000 1111
        assert_eq!(val.value(), 0x0F);

        assert_eq!(val.get_bits(1, 2), 0x03); // bits 1-2 are 11 (3)

        val.clear_bits(0, 1); // 0000 1100
        assert_eq!(val.value(), 0x0C);

        val.toggle_bits(2, 4); // 0001 1000 -> 0x10 (binary 0000 1100 -> toggle bits 2,3,4 -> 0001 0000)
        assert_eq!(val.value(), 0x10);

        val.set_bits(0, 7, 0xAA); // 1010 1010
        assert_eq!(val.value(), 0xAA);
    }

    #[test]
    fn bit_manipulation_out_of_bounds_handled_safely() {
        let mut val = TestByte::new(0x55);

        assert!(!val.get_bit(8));

        val.set_bit(8);
        assert_eq!(val.value(), 0x55);

        val.clear_bit(8);
        assert_eq!(val.value(), 0x55);

        val.toggle_bit(8);
        assert_eq!(val.value(), 0x55);

        assert_eq!(val.get_bits(8, 10), 0);

        val.set_bits(8, 10, 0x07);
        assert_eq!(val.value(), 0x55);

        val.clear_bits(8, 10);
        assert_eq!(val.value(), 0x55);

        val.toggle_bits(8, 10);
        assert_eq!(val.value(), 0x55);
    }

    #[test]
    fn serialization_symmetry_uint_roundtrip_correctly() {
        let raw_bytes: [u8; 2] = [0x34, 0x12];

        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let decoded = TestWord::decode(&mut cursor).expect("Failed to decode");

        assert_eq!(decoded.value(), 0x1234);

        let mut buffer = BytesMut::with_capacity(decoded.encoded_len());
        decoded.encode(&mut buffer).expect("Failed to encode");
        assert_eq!(
            buffer.as_ref(),
            &raw_bytes,
            "Inconsistent encode/decode symmetry"
        );
    }

    #[test]
    fn serialization_error_truncated_buffer_fails() {
        let raw_bytes: [u8; 1] = [0x34];
        let mut cursor = Bytes::copy_from_slice(&raw_bytes);
        let result = TestWord::decode(&mut cursor);

        assert!(matches!(
            result,
            Err(BinaryError::Truncated { expected: 2, .. })
        ));
    }

    #[test]
    fn serialization_error_buffer_too_small_fails() {
        let val = TestWord::new(0x1234);
        let mut buf = [0u8; 1];
        let mut buffer = &mut buf[..];
        let result = val.encode(&mut buffer);

        assert!(matches!(
            result,
            Err(BinaryError::BufferTooSmall { expected: 2, .. })
        ));
    }
}
