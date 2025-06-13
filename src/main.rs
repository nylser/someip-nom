use nom::{
    IResult, Input, Parser,
    bits::{bits, bytes, streaming::take},
    error::{Error as NomError, ErrorKind, ParseError},
    number::streaming::{be_u8, be_u16, be_u32},
};

/// client id / session id
type RequestId = u32;
type InterfaceVersion = u8;
type ProtocolVersion = u8;
type ReturnCode = u8;
type ClientId = u16;
type SessionId = u16;
type MessageId = u16;

#[derive(Debug, PartialEq)]
pub struct SomeIPHeader {
    service_id: u16,
    method_id: u16,
    length: u32,
    client_id: u16,
    session_id: u16,
    protocol_version: ProtocolVersion,
    interface_version: InterfaceVersion,
    message_type: SomeIPMessageType,
    return_code: ReturnCode,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error<'a> {
    pub input: &'a [u8],
    pub error: InnerError,
}

#[derive(Clone, Debug, PartialEq)]
pub enum InnerError {
    Nom(ErrorKind),
    NoError,
    ProtocolError,
    InternalError,
    FlowControlError,
    SettingsTimeout,
    StreamClosed,
    FrameSizeError,
    RefusedStream,
    Cancel,
    CompressionError,
    ConnectError,
    EnhanceYourCalm,
    InadequateSecurity,
    HTTP11Required,
}

impl<'a> Error<'a> {
    pub fn new(input: &'a [u8], error: InnerError) -> Error<'a> {
        Error { input, error }
    }
}

impl<'a> ParseError<&'a [u8]> for Error<'a> {
    fn from_error_kind(input: &'a [u8], kind: ErrorKind) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }

    fn append(input: &'a [u8], kind: ErrorKind, other: Self) -> Self {
        Error {
            input,
            error: InnerError::Nom(kind),
        }
    }
}

pub fn some_ip_header(input: &[u8]) -> IResult<&[u8], SomeIPHeader, Error> {
    let (i1, service_id) = be_u16(input)?;
    let (i2, method_id) = be_u16(i1)?;
    let (i3, length) = be_u32(i2)?;
    let (i4, client_id) = be_u16(i3)?;
    let (i5, session_id) = be_u16(i4)?;
    let (i6, protocol_version) = be_u8(i5)?;
    let (i7, interface_version) = be_u8(i6)?;
    let (i8, message_type) = be_u8(i7)?;
    let (i9, return_code) = be_u8(i8)?;
    Ok((
        i9,
        SomeIPHeader {
            service_id,
            method_id,
            length,
            client_id,
            session_id,
            protocol_version,
            interface_version,
            message_type: message_type.into(),
            return_code,
        },
    ))
}

#[derive(Debug, PartialEq)]
pub enum SomeIPMessageType {
    Request(),
    RequestNoReturn(),
    Notification(),
    Response(),
    Error(),
    TPRequest(),
    TPRequestNoReturn(),
    TPNotification(),
    Unknown(),
}

impl From<u8> for SomeIPMessageType {
    fn from(value: u8) -> Self {
        todo!("add values from documentation here");
        match value {
            0x00 => Self::Request(),
            _ => Self::Unknown(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct SomeIPMessage {
    header: SomeIPHeader,
}

fn main() {
    let bytes: Vec<u8> = vec![
        0xff, 0xff, 0x81, 0x0, 0x0, 0x0, 0x0, 0x30, 0x0, 0x0, 0x0, 0x3, 0x1, 0x1, 0x2, 0x0, 0xc0,
        0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x10, 0x1, 0x0, 0x0, 0x10, 0x0, 0xeb, 0x0, 0x0, 0x1, 0x0,
        0x0, 0x1e, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0xc, 0x0, 0x9, 0x4, 0x0, 0xc0, 0xa8, 0x58,
        0x49, 0x0, 0x11, 0xc3, 0x50,
    ];
    let slice = bytes.as_slice();
    let (payload, header) = some_ip_header(slice).unwrap();
    println!("{:?}", header);
}
