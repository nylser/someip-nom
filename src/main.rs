use nom::{
    IResult, Input, Parser,
    error::{Error as NomError, ErrorKind, ParseError},
    number::{
        streaming::{be_i8, be_i16, be_i32, be_i64},
        streaming::{be_u8, be_u16, be_u32, be_u64},
    },
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

pub fn some_ip_value<'a>(
    input: &'a [u8],
    def: &'a SomeIPType,
) -> IResult<&'a [u8], Value, Error<'a>> {
    let (i1, value) = match def {
        SomeIPType::UInt8 => {
            let (i1, val) = be_u8(input)?;
            (i1, Value::UInt(val.into()))
        }
        SomeIPType::UInt16 => {
            let (i1, val) = be_u16(input)?;
            (i1, Value::UInt(val.into()))
        }
        SomeIPType::UInt32 => {
            let (i1, val) = be_u32(input)?;
            (i1, Value::UInt(val.into()))
        }
        SomeIPType::UInt64 => {
            let (i1, val) = be_u64(input)?;
            (i1, Value::UInt(val))
        }
        SomeIPType::SInt8 => {
            let (i1, val) = be_i8(input)?;
            (i1, Value::Int(val.into()))
        }
        SomeIPType::SInt16 => {
            let (i1, val) = be_i16(input)?;
            (i1, Value::Int(val.into()))
        }
        SomeIPType::SInt32 => {
            let (i1, val) = be_i32(input)?;
            (i1, Value::Int(val.into()))
        }
        SomeIPType::SInt64 => {
            let (i1, val) = be_i64(input)?;
            (i1, Value::Int(val))
        }
        SomeIPType::Struct { fields } => someip_struct(input, fields),
        SomeIPType::DynamicArray {
            length_width,
            element,
        } => {
            let (i1, length) = someip_dynamic_length(input, length_width)?;

            someip_array(i1, element, length)?
        }
        SomeIPType::StaticArray { length, element } => {
            someip_array(input, element, *length as u64)?
        }
        SomeIPType::Enum { variants } => {
            let (i1, variant) = be_u8(input)?;

            (
                i1,
                Value::Enum(
                    variants
                        .iter()
                        .find(|(i, _)| *i == variant.into())
                        .unwrap()
                        .1
                        .clone(),
                ),
            )
        }
        SomeIPType::StaticString { length, coding: _ } => {
            let (i1, str_bytes) = nom::bytes::streaming::take(*length).parse(input)?;
            let str = String::from_utf8(str_bytes.to_vec()).unwrap();
            (i1, Value::String(str))
        }
        SomeIPType::DynamicString {
            length_width,
            coding: _,
        } => {
            let (i1, length) = someip_dynamic_length(input, length_width)?;
            let (i2, str_bytes) = nom::bytes::streaming::take(length).parse(i1)?;
            let str = String::from_utf8(str_bytes.to_vec()).unwrap();
            (i2, Value::String(str))
        }

        _ => {
            panic!("not implemented")
        }
    };
    Ok((i1, value))
    //Ok((input, Value::Int(8)))
}

fn someip_dynamic_length<'a>(
    input: &'a [u8],
    length_width: &'a u8,
) -> Result<(&'a [u8], u64), nom::Err<Error<'a>>> {
    let (i1, length) = match length_width {
        8 => {
            let (input, length) = be_u8(input)?;
            (input, length as u64)
        }
        16 => {
            let (input, length) = be_u16(input)?;
            (input, length as u64)
        }
        32 => {
            let (input, length) = be_u32(input)?;
            (input, length as u64)
        }
        64 => {
            let (input, length) = be_u64(input)?;
            (input, length)
        }
        _ => {
            panic!("invalid length width")
        }
    };
    Ok((i1, length))
}

fn someip_array<'a>(
    mut input: &'a [u8],
    element: &'a SomeIPType,
    length: u64,
) -> Result<(&'a [u8], Value), nom::Err<Error<'a>>> {
    let mut elements = Vec::new();
    for _ in 0..length {
        let (new_input, value) = some_ip_value(input, element)?;
        input = new_input;
        elements.push(value);
    }
    Ok((input, Value::Array(elements)))
}

fn someip_struct<'a>(input: &'a [u8], fields: &'a [(String, SomeIPType)]) -> (&'a [u8], Value) {
    let mut i1 = input;
    let fields = fields
        .iter()
        .map(|(name, def)| {
            let (new_input, value) = some_ip_value(i1, def).unwrap();
            i1 = new_input;
            (name.clone(), value)
        })
        .collect();
    (i1, Value::Struct { fields })
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

pub struct SomeIPMessageBody {}

pub enum StringCoding {
    Utf8,
    Utf16,
}

pub enum SomeIPType {
    Float32,
    Float64,
    SInt8,
    SInt16,
    SInt32,
    SInt64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Struct {
        fields: Vec<(String, SomeIPType)>,
    },
    StaticArray {
        length: u32,
        element: Box<SomeIPType>,
    },
    DynamicArray {
        length_width: u8,
        element: Box<SomeIPType>,
    },
    Enum {
        variants: Vec<(u64, String)>,
    },
    StaticString {
        length: u32,
        coding: Option<StringCoding>,
    },
    DynamicString {
        length_width: u8,
        coding: Option<StringCoding>,
    },
}

pub enum Value {
    Float(f64),
    UInt(u64),
    Int(i64),
    Struct { fields: Vec<(String, Value)> },
    Array(Vec<Value>),
    Enum(String),
    String(String),
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
