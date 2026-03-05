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
    impl_cip_primitive!(TestDWord, u32);
    impl_cip_primitive!(TestLWord, u64);

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
    fn bit_manipulation_32_and_64_bits_behave_correctly() {
        let mut val32 = TestDWord::new(0); // 00000000 00000000 00000000 00000000
        val32.set_bit(31); // 10000000 00000000 00000000 00000000
        assert!(val32.get_bit(31));
        assert_eq!(val32.value(), 0x8000_0000);

        val32.set_bits(0, 7, 0xFF); // 10000000 00000000 00000000 11111111
        assert_eq!(val32.get_bits(0, 3), 0x0F);
        assert_eq!(val32.value(), 0x8000_00FF);

        let mut val64 = TestLWord::new(0); // 64 bits of zeros
        val64.set_bit(63); // 10000000 ... (63 zeros)
        assert!(val64.get_bit(63));
        assert_eq!(val64.value(), 0x8000_0000_0000_0000);

        val64.set_bits(0, 15, 0xFFFF); // 10000000 00000000 ... 11111111 11111111
        assert_eq!(val64.get_bits(0, 7), 0xFF);
        assert_eq!(val64.value(), 0x8000_0000_0000_FFFF);
    }

    #[test]
    fn bit_manipulation_16_bits_behave_correctly() {
        let mut val16 = TestWord::new(0); // 00000000 00000000
        val16.set_bit(15); // 10000000 00000000
        assert!(val16.get_bit(15));
        assert_eq!(val16.value(), 0x8000);

        val16.set_bits(0, 7, 0xFF); // 10000000 11111111
        assert_eq!(val16.get_bits(0, 3), 0x0F);
        assert_eq!(val16.value(), 0x80FF);
    }

    #[test]
    fn serialization_symmetry_uint_roundtrip_correctly() {
        // 16-bit
        let raw_bytes16: [u8; 2] = [0x34, 0x12];
        let mut cursor16 = Bytes::copy_from_slice(&raw_bytes16);
        let decoded16 = TestWord::decode(&mut cursor16).expect("Failed to decode u16");
        assert_eq!(decoded16.value(), 0x1234);
        let mut buffer16 = BytesMut::with_capacity(decoded16.encoded_len());
        decoded16
            .encode(&mut buffer16)
            .expect("Failed to encode u16");
        assert_eq!(buffer16.as_ref(), &raw_bytes16);

        // 32-bit
        let raw_bytes32: [u8; 4] = [0x78, 0x56, 0x34, 0x12];
        let mut cursor32 = Bytes::copy_from_slice(&raw_bytes32);
        let decoded32 = TestDWord::decode(&mut cursor32).expect("Failed to decode u32");
        assert_eq!(decoded32.value(), 0x12345678);
        let mut buffer32 = BytesMut::with_capacity(decoded32.encoded_len());
        decoded32
            .encode(&mut buffer32)
            .expect("Failed to encode u32");
        assert_eq!(buffer32.as_ref(), &raw_bytes32);

        // 64-bit
        let raw_bytes64: [u8; 8] = [0xEF, 0xCD, 0xAB, 0x89, 0x67, 0x45, 0x23, 0x01];
        let mut cursor64 = Bytes::copy_from_slice(&raw_bytes64);
        let decoded64 = TestLWord::decode(&mut cursor64).expect("Failed to decode u64");
        assert_eq!(decoded64.value(), 0x0123456789ABCDEF);
        let mut buffer64 = BytesMut::with_capacity(decoded64.encoded_len());
        decoded64
            .encode(&mut buffer64)
            .expect("Failed to encode u64");
        assert_eq!(buffer64.as_ref(), &raw_bytes64);
    }

    #[test]
    fn serialization_error_truncated_buffer_fails() {
        // u16
        let mut cursor = Bytes::copy_from_slice(&[0x34]);
        assert!(matches!(
            TestWord::decode(&mut cursor),
            Err(BinaryError::Truncated { expected: 2, .. })
        ));

        // u32
        let mut cursor = Bytes::copy_from_slice(&[0x34, 0x12, 0x00]);
        assert!(matches!(
            TestDWord::decode(&mut cursor),
            Err(BinaryError::Truncated { expected: 4, .. })
        ));

        // u64
        let mut cursor = Bytes::copy_from_slice(&[0x34, 0x12, 0x00, 0x00, 0x00, 0x00, 0x00]);
        assert!(matches!(
            TestLWord::decode(&mut cursor),
            Err(BinaryError::Truncated { expected: 8, .. })
        ));
    }

    #[test]
    fn serialization_error_buffer_too_small_fails() {
        // u32
        let val32 = TestDWord::new(0x12345678);
        let mut buf = [0u8; 3];
        let mut buffer = &mut buf[..];
        assert!(matches!(
            val32.encode(&mut buffer),
            Err(BinaryError::BufferTooSmall { expected: 4, .. })
        ));

        // u64
        let val64 = TestLWord::new(0x0123456789ABCDEF);
        let mut buf = [0u8; 7];
        let mut buffer = &mut buf[..];
        assert!(matches!(
            val64.encode(&mut buffer),
            Err(BinaryError::BufferTooSmall { expected: 8, .. })
        ));
    }
}
