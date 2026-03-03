mod port_segment;

pub struct EPath {}

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

pub enum Segment {
    Port(port_segment::PortSegment),
}
