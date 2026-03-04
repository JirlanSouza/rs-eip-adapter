mod port_segment;

use bytes::Buf;
pub use port_segment::PortSegment;

use crate::common::binary::{BinaryError, FromBytes, ToBytes};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Segment {
    Port(PortSegment),
}

impl Segment {
    pub const MIN_LEN: usize = 1;
}

impl FromBytes for Segment {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Self::MIN_LEN {
            return Err(BinaryError::Truncated {
                expected: Self::MIN_LEN,
                actual: buffer.remaining(),
            });
        }

        let mut first_byte = [0u8; 1];
        buffer.copy_to_slice(&mut first_byte);
        let segment_type = SegmentType::from(first_byte[0]);

        match segment_type {
            SegmentType::PortSegment => {
                let port_segment = PortSegment::decode(buffer)?;
                Ok(Segment::Port(port_segment))
            }
            _ => Err(BinaryError::InvalidData {
                message: "Invalid EPATH segment data".to_string(),
                expected: "Valid segment type".to_string(),
                actual: segment_type.to_string(),
            }),
        }
    }
}

impl ToBytes for Segment {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        match self {
            Segment::Port(port_segment) => port_segment.encode(buffer)?,
        }

        Ok(())
    }

    fn encoded_len(&self) -> usize {
        match self {
            Segment::Port(port_segment) => port_segment.encoded_len(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaddedEPath {
    segments: Vec<Segment>,
}

impl PaddedEPath {
    pub fn new(segments: Vec<Segment>) -> Self {
        Self { segments }
    }
}

impl FromBytes for PaddedEPath {
    fn decode<T: Buf>(buffer: &mut T) -> Result<Self, BinaryError> {
        if buffer.remaining() < Segment::MIN_LEN {
            return Err(BinaryError::Truncated {
                expected: Segment::MIN_LEN,
                actual: buffer.remaining(),
            });
        }

        let mut segments = Vec::new();
        loop {
            if buffer.remaining() < Segment::MIN_LEN {
                break;
            }
            let segment = Segment::decode(buffer)?;
            segments.push(segment);
        }

        Ok(Self { segments })
    }
}

impl ToBytes for PaddedEPath {
    fn encode<T: bytes::BufMut>(&self, buffer: &mut T) -> Result<(), BinaryError> {
        if buffer.remaining_mut() < self.encoded_len() {
            return Err(BinaryError::BufferTooSmall {
                expected: self.encoded_len(),
                actual: buffer.remaining_mut(),
            });
        }

        for segment in &self.segments {
            segment.encode(buffer)?;
        }

        Ok(())
    }

    fn encoded_len(&self) -> usize {
        self.segments.iter().map(|seg| seg.encoded_len()).sum()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SegmentType {
    PortSegment = 0b000,
    LogicalSegment = 0b001,
    NetworkSegment = 0b010,
    SymbolicSegment = 0b011,
    DataSegment = 0b100,
    DataTypeConstructed = 0b101,
    DataTypeElementary = 0b110,
    Reserved = 0b111,
}

impl From<u8> for SegmentType {
    fn from(value: u8) -> Self {
        match value {
            0b000 => Self::PortSegment,
            0b001 => Self::LogicalSegment,
            0b010 => Self::NetworkSegment,
            0b011 => Self::SymbolicSegment,
            0b100 => Self::DataSegment,
            0b101 => Self::DataTypeConstructed,
            0b110 => Self::DataTypeElementary,
            0b111 => Self::Reserved,
            _ => unreachable!(),
        }
    }
}

impl From<SegmentType> for u8 {
    fn from(value: SegmentType) -> Self {
        value as u8
    }
}

impl std::fmt::Display for SegmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
