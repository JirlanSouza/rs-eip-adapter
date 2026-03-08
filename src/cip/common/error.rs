use thiserror::Error;

use crate::common::binary::BinaryError;

#[derive(Debug, Error)]
pub enum CipError {
    #[error("0x00 - success")]
    Success = 0x00,

    #[error("0x01 - connection failure")]
    ConnectionFailure = 0x01,

    #[error("0x02 - resource unavailable")]
    ResourceUnavailable = 0x02,

    #[error("0x03 - invalid parameter value")]
    InvalidParameterValue = 0x03,

    #[error("0x04 - path segment error")]
    PathSegmentError = 0x04,

    #[error("0x05 - path destination unknown")]
    PathDestinationUnknown = 0x05,

    #[error("0x06 - partial transfer")]
    PartialTransfer = 0x06,

    #[error("0x07 - connection lost")]
    ConnectionLost = 0x07,

    #[error("0x08 - service not supported")]
    ServiceNotSupported = 0x08,

    #[error("0x09 - invalid attribute value")]
    InvalidAttributeValue = 0x09,

    #[error("0x0A - attribute list error")]
    AttributeListError = 0x0A,

    #[error("0x0B - already in requested mode")]
    AlreadyInRequestedMode = 0x0B,

    #[error("0x0C - object state conflict")]
    ObjectStateConflict = 0x0C,

    #[error("0x0D - object already exists")]
    ObjectAlreadyExists = 0x0D,

    #[error("0x0E - attribute not setable")]
    AttributeNotSetable = 0x0E,

    #[error("0x0F - privilege violation")]
    PrivilegeViolation = 0x0F,

    #[error("0x10 - device state conflict")]
    DeviceStateConflict = 0x10,

    #[error("0x11 - reply data too large")]
    ReplyDataTooLarge = 0x11,

    #[error("0x12 - fragmentation of a primitive value")]
    FragmentationOfAPrimitiveValue = 0x12,

    #[error("0x13 - not enough data")]
    NotEnoughData = 0x13,

    #[error("0x14 - attribute not supported")]
    AttributeNotSupported = 0x14,

    #[error("0x15 - too much data")]
    TooMuchData = 0x15,

    #[error("0x16 - object does not exist")]
    ObjectDoesNotExist = 0x16,

    #[error("0x17 - service fragmentation sequence not in progress")]
    ServiceFragmentationSequenceNotInProgress = 0x17,

    #[error("0x18 - no stored attribute data")]
    NoStoredAttributeData = 0x18,

    #[error("0x19 - store operation failure")]
    StoreOperationFailure = 0x19,

    #[error("0x1A - routing failure: request packet too large")]
    RoutingFailureRequestPacketTooLarge = 0x1A,

    #[error("0x1B - routing failure: response packet too large")]
    RoutingFailureResponsePacketTooLarge = 0x1B,

    #[error("0x1C - missing attribute list entry")]
    MissingAttributeListEntry = 0x1C,

    #[error("0x1D - invalid attribute value list")]
    InvalidAttributeValueList = 0x1D,

    #[error("0x1E - embedded service error")]
    EmbeddedServiceError = 0x1E,

    #[error("0x1F - vendor specific error")]
    VendorSpecificError = 0x1F,

    #[error("0x20 - invalid parameter")]
    InvalidParameter = 0x20,

    #[error("0x21 - write-once value or medium already written")]
    WriteonceValueOrMediumAlreadyWritten = 0x21,

    #[error("0x22 - invalid reply received")]
    InvalidReplyReceived = 0x22,

    #[error("0x25 - key failure in path")]
    KeyFailureInPath = 0x25,

    #[error("0x26 - path size invalid")]
    PathSizeInvalid = 0x26,

    #[error("0x27 - unexpected attribute in list")]
    UnexpectedAttributeInList = 0x27,

    #[error("0x28 - invalid member id")]
    InvalidMemberId = 0x28,

    #[error("0x29 - member not setable")]
    MemberNotSetable = 0x29,

    #[error("0x2A - group 2 only server general failure")]
    Group2OnlyServerGeneralFailure = 0x2A,

    #[error("0x2C - attribute not gettable")]
    AttributeNotGettable = 0x2C,

    #[error("0xFF - general error")]
    GeneralError = 0xFF,
}

impl From<CipError> for u8 {
    fn from(value: CipError) -> Self {
        value as u8
    }
}

impl From<BinaryError> for CipError {
    fn from(value: BinaryError) -> Self {
        match value {
            BinaryError::BufferTooSmall { .. } => CipError::ReplyDataTooLarge,
            BinaryError::Truncated { .. } => CipError::NotEnoughData,
            BinaryError::InvalidData { .. } => CipError::InvalidParameterValue,
        }
    }
}
