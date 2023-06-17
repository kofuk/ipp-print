use num::FromPrimitive;
use reqwest::blocking::Client;
use std::collections::{HashMap, LinkedList};
use std::error::Error;
use std::fmt;
use std::io;
use std::io::prelude::*;
use std::ops::Range;

#[macro_use]
extern crate num_derive;

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy)]
#[allow(unused)]
enum PrinterOperation {
    PrintJob = 0x0002,
    PrintURI = 0x0003,
    ValidateJob = 0x0004,
    CreateJob = 0x0005,
    SendDocument = 0x0006,
    SendURI = 0x0007,
    CancelJob = 0x0008,
    GetJobAttributes = 0x0009,
    GetJobs = 0x000a,
    GetPrinterAttributes = 0x000b,
    HoldJob = 0x000c,
    ReleaseJob = 0x000d,
    RestartJob = 0x000e,
    PausePrinter = 0x0010,
    ResumePrinter = 0x0011,
    PurgeJobs = 0x0012,
}

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq, Clone, Copy)]
#[allow(unused)]
enum DelimiterOrValueTag {
    // delimiter-tag
    OperationAttributesTag = 0x01,
    JobAttributesTag = 0x02,
    EndOfAttributesTag = 0x03,
    PrinterAttributesTag = 0x04,
    UnsupportedAttributesTag = 0x05,

    // value-tag
    Unsupported = 0x10,
    Unknown = 0x12,
    NoValue = 0x13,

    Integer = 0x21,
    Boolean = 0x22,
    Enum = 0x23,

    OctetStringUnspecified = 0x30,
    DateTime = 0x31,
    Resolution = 0x32,
    RangeOfInteger = 0x33,
    BegCollection = 0x34,
    TextWithLanguage = 0x35,
    NameWithLanguage = 0x36,
    EndCollection = 0x37,

    TextWithoutLanguage = 0x41,
    NameWithoutLanguage = 0x42,
    Keyword = 0x44,
    Uri = 0x45,
    UriScheme = 0x46,
    Charset = 0x47,
    NaturalLanguage = 0x48,
    MimeMediaType = 0x49,
    MemberAttrName = 0x4a,
}

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq)]
#[allow(unused)]
enum StatusCode {
    SuccessfulOk = 0x0000,
    SuccessfulOkIgnoredOrSubstitutedAttributes = 0x0001,
    SuccessfulOkConflictingAttributes = 0x0002,
    ClientErrorBadRequest = 0x0400,
    ClientErrorForbidden = 0x0401,
    ClientErrorNotAuthenticated = 0x0402,
    ClientErrorNotAuthorized = 0x0403,
    ClientErrorNotPossible = 0x0404,
    ClientErrorTimeout = 0x045,
    ClientErrorNotFound = 0x0406,
    ClientErrorGone = 0x0407,
    ClientErrorRequestEntityTooLarge = 0x0408,
    ClientErrorRequestValueTooLong = 0x0409,
    ClientErrorDocumentFormatNotSupported = 0x040a,
    ClientErrorAttributesOrValuesNotSupported = 0x040b,
    ClientErrorUriSchemeNotSupported = 0x040c,
    ClientErrorCharsetNotSupported = 0x040d,
    ClientErrorConflictingAttributes = 0x040e,
    ClientErrorCompressionNotSupported = 0x040f,
    ClientErrorCompressionError = 0x0410,
    ClientErrorDocumentFormatError = 0x0411,
    ClientErrorDocumentAccessError = 0x04012,
    ServerErrorInternalError = 0x0500,
    ServerErrorOperationNotSupported = 0x0501,
    ServerErrorServiceUnavailable = 0x0502,
    ServerErrorVersionNotSupported = 0x0503,
    ServerErrorDeviceError = 0x0504,
    ServerErrorTemporaryError = 0x0505,
    ServerErrorNotAcceptingJobs = 0x0506,
    ServerErrorBusy = 0x0507,
    ServerErrorJobCanceled = 0x0508,
    ServerErrorMultipleDocumentJobsNotSupported = 0x0509,
}

#[derive(Debug)]
enum IPPError {
    IOError(io::Error),
    ProtocolError,
    ValueFormatError,
}

impl fmt::Display for IPPError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IOError(err) => err.fmt(f),
            Self::ProtocolError => {
                write!(f, "protocol error")
            }
            Self::ValueFormatError => {
                write!(f, "value format error")
            }
        }
    }
}

impl Error for IPPError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::IOError(err) => Some(err),
            Self::ProtocolError => None,
            Self::ValueFormatError => None,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(unused)]
struct StringWithLanguage {
    lang: String,
    string: String,
}

impl StringWithLanguage {
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, IPPError> {
        let len = buf.len();
        if len < 2 {
            return Err(IPPError::ValueFormatError);
        }
        let lang_len = i16::from_be_bytes([buf[0], buf[1]]) as usize;
        if len < 2 + lang_len + 2 {
            return Err(IPPError::ValueFormatError);
        }
        let str_len = i16::from_be_bytes([buf[2 + lang_len], buf[2 + lang_len + 1]]) as usize;
        if len < 2 + lang_len + 2 + str_len {
            return Err(IPPError::ValueFormatError);
        }

        Ok(Self {
            lang: String::from_utf8(buf[2..(2 + lang_len)].to_vec()).unwrap(),
            string: String::from_utf8(
                buf[(2 + lang_len + 2)..(2 + lang_len + 2 + str_len)].to_vec(),
            )
            .unwrap(),
        })
    }

    fn byte_len(&self) -> u16 {
        (2 + self.lang.as_bytes().len() + 2 + self.string.as_bytes().len()) as u16
    }

    fn write_to_stream<W>(&self, writer: &mut W) -> Result<usize, IPPError>
    where
        W: Write,
    {
        let mut written = 0;

        let lang_bytes = self.lang.as_bytes();
        written += match writer.write(&(lang_bytes.len() as u16).to_be_bytes()) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };
        written += match writer.write(&lang_bytes) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };

        let str_bytes = self.string.as_bytes();
        written += match writer.write(&(str_bytes.len() as u16).to_be_bytes()) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };
        written += match writer.write(&str_bytes) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };

        Ok(written)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(unused)]
struct Resolution {
    resolution_cross_feed: i32,
    resolution_feed: i32,
    units: i8,
}

impl Resolution {
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, IPPError> {
        if buf.len() != 9 {
            return Err(IPPError::ValueFormatError);
        }

        Ok(Self {
            resolution_cross_feed: i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            resolution_feed: i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            units: buf[8] as i8,
        })
    }

    fn byte_len(&self) -> u16 {
        9
    }

    fn write_to_stream<W>(&self, writer: &mut W) -> Result<usize, IPPError>
    where
        W: Write,
    {
        if let Err(err) = writer.write(&self.resolution_cross_feed.to_be_bytes()) {
            return Err(IPPError::IOError(err));
        }
        if let Err(err) = writer.write(&self.resolution_feed.to_be_bytes()) {
            return Err(IPPError::IOError(err));
        }
        if let Err(err) = writer.write(&[self.units as u8]) {
            return Err(IPPError::IOError(err));
        }

        Ok(9)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[allow(unused)]
struct DateTime {
    year: u16,
    month: u8,
    day: u8,
    hour: u8,
    minutes: u8,
    seconds: u8,
    deci_seconds: u8,
    direction_from_utc: char,
    hours_from_utc: u8,
    minutes_from_utc: u8,
}

impl DateTime {
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, IPPError> {
        if buf.len() != 11 {
            return Err(IPPError::ValueFormatError);
        }
        Ok(Self {
            year: u16::from_be_bytes([buf[0], buf[1]]),
            month: buf[2],
            day: buf[3],
            hour: buf[4],
            minutes: buf[5],
            seconds: buf[6],
            deci_seconds: buf[7],
            direction_from_utc: buf[8] as char,
            hours_from_utc: buf[9],
            minutes_from_utc: buf[10],
        })
    }

    fn byte_len(&self) -> u16 {
        11
    }

    fn write_to_stream<W>(&self, writer: &mut W) -> Result<usize, IPPError>
    where
        W: Write,
    {
        if let Err(err) = writer.write(&self.year.to_be_bytes()) {
            return Err(IPPError::IOError(err));
        }
        if let Err(err) = writer.write(&[
            self.month,
            self.day,
            self.hour,
            self.minutes,
            self.seconds,
            self.deci_seconds,
            self.direction_from_utc as u8,
            self.hours_from_utc,
            self.minutes_from_utc,
        ]) {
            return Err(IPPError::IOError(err));
        }

        Ok(11)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum AttributeValue {
    Unsupported(Vec<u8>),
    Unknown(Vec<u8>),
    NoValue,
    Integer(i32),
    Boolean(bool),
    Enum(i32),
    OctetStringUnspecified(String),
    DateTime(DateTime),
    Resolution(Resolution),
    RangeOfInteger(Range<i32>),
    BegCollection,
    TextWithLanguage(StringWithLanguage),
    NameWithLanguage(StringWithLanguage),
    EndCollection,
    TextWithoutLanguage(String),
    NameWithoutLanguage(String),
    Keyword(String),
    Uri(String),
    UriScheme(String),
    Charset(String),
    NaturalLanguage(String),
    MimeMediaType(String),
    MemberAttrName(String),
    CollectionAttribute(HashMap<String, AttributeValue>),
    VectorAttribute(Vec<AttributeValue>),
}

#[derive(Debug, PartialEq, Eq)]
struct IPPRequest {
    version_major: i8,
    version_minor: i8,
    operation_id: PrinterOperation,
    request_id: i32,
    attrs: Vec<(DelimiterOrValueTag, Vec<(String, AttributeValue)>)>,
    data: Vec<u8>,
}

impl IPPRequest {
    fn write_tag<W>(writer: &mut W, tag: DelimiterOrValueTag) -> Result<usize, IPPError>
    where
        W: Write,
    {
        match writer.write(&[tag as u8]) {
            Ok(written) => Ok(written),
            Err(err) => Err(IPPError::IOError(err)),
        }
    }

    fn write_u16<W>(writer: &mut W, val: u16) -> Result<usize, IPPError>
    where
        W: Write,
    {
        match writer.write(&val.to_be_bytes()) {
            Ok(written) => Ok(written),
            Err(err) => Err(IPPError::IOError(err)),
        }
    }

    fn write_str_and_len<W>(writer: &mut W, val: &str) -> Result<usize, IPPError>
    where
        W: Write,
    {
        let mut written = 0;
        written += IPPRequest::write_u16(writer, val.as_bytes().len() as u16)?;
        written += match writer.write(val.as_bytes()) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };
        Ok(written)
    }

    fn write_attribute_group<W>(&self, writer: &mut W) -> Result<usize, IPPError>
    where
        W: Write,
    {
        let mut written = 0;

        for group in &self.attrs {
            written += match writer.write(&[group.0 as u8]) {
                Ok(written) => written,
                Err(err) => return Err(IPPError::IOError(err)),
            };

            for attr in &group.1 {
                match &attr.1 {
                    AttributeValue::Unsupported(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Unsupported)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.len() as u16)?;
                        written += match writer.write(val.as_slice()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::Unknown(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Unknown)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.len() as u16)?;
                        written += match writer.write(val.as_slice()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::NoValue => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::NoValue)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 0u16)?;
                    }
                    AttributeValue::Integer(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Integer)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 4u16)?;
                        written += match writer.write(&val.to_be_bytes()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::Boolean(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Boolean)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 1u16)?;
                        written += match writer.write(&[if *val { 1u8 } else { 0u8 }]) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::Enum(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Enum)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 4u16)?;
                        written += match writer.write(&val.to_be_bytes()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::OctetStringUnspecified(val) => {
                        written += IPPRequest::write_tag(
                            writer,
                            DelimiterOrValueTag::OctetStringUnspecified,
                        )?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::DateTime(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::DateTime)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.byte_len())?;
                        written += val.write_to_stream(writer)?;
                    }
                    AttributeValue::Resolution(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Resolution)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.byte_len())?;
                        written += val.write_to_stream(writer)?;
                    }
                    AttributeValue::RangeOfInteger(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::RangeOfInteger)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 8u16)?;
                        written += match writer.write(&val.start.to_be_bytes()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                        written += match writer.write(&val.end.to_be_bytes()) {
                            Ok(written) => written,
                            Err(err) => return Err(IPPError::IOError(err)),
                        };
                    }
                    AttributeValue::BegCollection => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::BegCollection)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 0u16)?;
                    }
                    AttributeValue::TextWithLanguage(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::TextWithLanguage)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.byte_len())?;
                        written += val.write_to_stream(writer)?;
                    }
                    AttributeValue::NameWithLanguage(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::NameWithLanguage)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, val.byte_len())?;
                        written += val.write_to_stream(writer)?;
                    }
                    AttributeValue::EndCollection => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::EndCollection)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_u16(writer, 0u16)?;
                    }
                    AttributeValue::TextWithoutLanguage(val) => {
                        written += IPPRequest::write_tag(
                            writer,
                            DelimiterOrValueTag::TextWithoutLanguage,
                        )?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::NameWithoutLanguage(val) => {
                        written += IPPRequest::write_tag(
                            writer,
                            DelimiterOrValueTag::NameWithoutLanguage,
                        )?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::Keyword(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Keyword)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::Uri(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Uri)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::UriScheme(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::UriScheme)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::Charset(val) => {
                        written += IPPRequest::write_tag(writer, DelimiterOrValueTag::Charset)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::NaturalLanguage(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::NaturalLanguage)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::MimeMediaType(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::MimeMediaType)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::MemberAttrName(val) => {
                        written +=
                            IPPRequest::write_tag(writer, DelimiterOrValueTag::MemberAttrName)?;
                        written += IPPRequest::write_str_and_len(writer, attr.0.as_str())?;
                        written += IPPRequest::write_str_and_len(writer, val.as_str())?;
                    }
                    AttributeValue::CollectionAttribute(val) => (),
                    AttributeValue::VectorAttribute(vals) => (),
                }
            }
        }

        Ok(written)
    }

    fn write_to_stream<W>(&self, writer: &mut W) -> Result<usize, IPPError>
    where
        W: Write,
    {
        let mut written = 0;
        written += match writer.write(&[self.version_major as u8, self.version_minor as u8]) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };
        written += match writer.write(&i16::to_be_bytes(self.operation_id as i16)) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };
        written += match writer.write(&i32::to_be_bytes(self.request_id as i32)) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };

        written += self.write_attribute_group(writer)?;

        written += match writer.write(&self.data) {
            Ok(written) => written,
            Err(err) => return Err(IPPError::IOError(err)),
        };

        Ok(written)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct IPPResponse {
    version_major: i8,
    version_minor: i8,
    status_code: StatusCode,
    request_id: i32,
    attrs: Vec<(DelimiterOrValueTag, Vec<(String, AttributeValue)>)>,
    data: Vec<u8>,
}

impl IPPResponse {
    fn decode_attribute_value(
        value_type: DelimiterOrValueTag,
        buf: Vec<u8>,
    ) -> Result<AttributeValue, IPPError> {
        match value_type {
            DelimiterOrValueTag::Unsupported => Ok(AttributeValue::Unsupported(buf)),
            DelimiterOrValueTag::Unknown => Ok(AttributeValue::Unknown(buf)),
            DelimiterOrValueTag::NoValue => {
                if buf.is_empty() {
                    Ok(AttributeValue::NoValue)
                } else {
                    Err(IPPError::ProtocolError)
                }
            }
            DelimiterOrValueTag::Integer => {
                if buf.len() == 4 {
                    Ok(AttributeValue::Integer(i32::from_be_bytes([
                        buf[0], buf[1], buf[2], buf[3],
                    ])))
                } else {
                    Err(IPPError::ProtocolError)
                }
            }
            DelimiterOrValueTag::Boolean => {
                if buf.len() == 1 {
                    Ok(AttributeValue::Boolean(buf[0] == 1u8))
                } else {
                    Err(IPPError::ProtocolError)
                }
            }
            DelimiterOrValueTag::Enum => {
                if buf.len() == 4 {
                    Ok(AttributeValue::Enum(i32::from_be_bytes([
                        buf[0], buf[1], buf[2], buf[3],
                    ])))
                } else {
                    Err(IPPError::ProtocolError)
                }
            }
            DelimiterOrValueTag::OctetStringUnspecified => Ok(
                AttributeValue::OctetStringUnspecified(match String::from_utf8(buf) {
                    Ok(str) => str,
                    Err(_) => return Err(IPPError::ValueFormatError),
                }),
            ),
            DelimiterOrValueTag::DateTime => {
                Ok(AttributeValue::DateTime(DateTime::parse_buffer(buf)?))
            }
            DelimiterOrValueTag::Resolution => {
                Ok(AttributeValue::Resolution(Resolution::parse_buffer(buf)?))
            }
            DelimiterOrValueTag::RangeOfInteger => {
                if buf.len() != 8 {
                    return Err(IPPError::ValueFormatError);
                }
                let start = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
                let end = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

                Ok(AttributeValue::RangeOfInteger(start..end))
            }
            DelimiterOrValueTag::BegCollection => {
                if !buf.is_empty() {
                    return Err(IPPError::ProtocolError);
                }
                Ok(AttributeValue::BegCollection)
            }
            DelimiterOrValueTag::TextWithLanguage => Ok(AttributeValue::TextWithLanguage(
                StringWithLanguage::parse_buffer(buf)?,
            )),
            DelimiterOrValueTag::NameWithLanguage => Ok(AttributeValue::NameWithLanguage(
                StringWithLanguage::parse_buffer(buf)?,
            )),
            DelimiterOrValueTag::EndCollection => {
                if !buf.is_empty() {
                    return Err(IPPError::ProtocolError);
                }
                Ok(AttributeValue::EndCollection)
            }
            DelimiterOrValueTag::TextWithoutLanguage => Ok(AttributeValue::TextWithoutLanguage(
                match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                },
            )),
            DelimiterOrValueTag::NameWithoutLanguage => Ok(AttributeValue::NameWithoutLanguage(
                match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                },
            )),
            DelimiterOrValueTag::Keyword => {
                Ok(AttributeValue::Keyword(match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                }))
            }
            DelimiterOrValueTag::Uri => Ok(AttributeValue::Uri(match String::from_utf8(buf) {
                Ok(result) => result,
                Err(_) => return Err(IPPError::ValueFormatError),
            })),
            DelimiterOrValueTag::UriScheme => {
                Ok(AttributeValue::UriScheme(match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                }))
            }
            DelimiterOrValueTag::Charset => {
                Ok(AttributeValue::Charset(match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                }))
            }
            DelimiterOrValueTag::NaturalLanguage => Ok(AttributeValue::NaturalLanguage(
                match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                },
            )),
            DelimiterOrValueTag::MimeMediaType => Ok(AttributeValue::MimeMediaType(
                match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                },
            )),
            DelimiterOrValueTag::MemberAttrName => Ok(AttributeValue::MemberAttrName(
                match String::from_utf8(buf) {
                    Ok(result) => result,
                    Err(_) => return Err(IPPError::ValueFormatError),
                },
            )),

            _ => return Err(IPPError::ProtocolError),
        }
    }

    fn parse_attribute<R>(
        reader: &mut R,
        value_type: DelimiterOrValueTag,
    ) -> Result<(String, AttributeValue), IPPError>
    where
        R: Read,
    {
        let mut buf = [0u8; 2];
        if let Err(err) = reader.read_exact(&mut buf) {
            return Err(IPPError::IOError(err));
        };
        let name_len = i16::from_be_bytes(buf);
        let mut name_buf = vec![0u8; name_len as usize];
        if let Err(err) = reader.read_exact(name_buf.as_mut_slice()) {
            return Err(IPPError::IOError(err));
        };
        let name = match String::from_utf8(name_buf) {
            Ok(name) => name,
            Err(_) => return Err(IPPError::ValueFormatError),
        };

        if let Err(err) = reader.read_exact(&mut buf) {
            return Err(IPPError::IOError(err));
        };
        let value_len = i16::from_be_bytes(buf);
        let mut value_buf = vec![0u8; value_len as usize];
        if let Err(err) = reader.read_exact(value_buf.as_mut_slice()) {
            return Err(IPPError::IOError(err));
        };

        let value = IPPResponse::decode_attribute_value(value_type, value_buf)?;

        Ok((name, value))
    }

    fn parse_tag<R>(reader: &mut R) -> Result<DelimiterOrValueTag, IPPError>
    where
        R: Read,
    {
        let mut buf = [0u8; 1];
        if let Err(err) = reader.read_exact(&mut buf) {
            return Err(IPPError::IOError(err));
        };

        match FromPrimitive::from_u8(buf[0]) {
            Some(value) => Ok(value),
            None => Err(IPPError::ProtocolError),
        }
    }

    fn parse_collection<R>(reader: &mut R) -> Result<AttributeValue, IPPError>
    where
        R: Read,
    {
        let mut map = HashMap::<String, AttributeValue>::new();
        loop {
            let attr_name = match IPPResponse::parse_tag(reader)? {
                DelimiterOrValueTag::EndCollection => {
                    IPPResponse::parse_attribute(reader, DelimiterOrValueTag::EndCollection)?;
                    break;
                }
                DelimiterOrValueTag::MemberAttrName => {
                    if let (_, AttributeValue::MemberAttrName(name)) =
                        IPPResponse::parse_attribute(reader, DelimiterOrValueTag::MemberAttrName)?
                    {
                        name
                    } else {
                        return Err(IPPError::ProtocolError);
                    }
                }
                _ => return Err(IPPError::ProtocolError),
            };

            let value = match IPPResponse::parse_tag(reader)? {
                DelimiterOrValueTag::OperationAttributesTag
                | DelimiterOrValueTag::JobAttributesTag
                | DelimiterOrValueTag::PrinterAttributesTag
                | DelimiterOrValueTag::UnsupportedAttributesTag => {
                    return Err(IPPError::ProtocolError)
                }
                DelimiterOrValueTag::BegCollection => {
                    IPPResponse::parse_attribute(reader, DelimiterOrValueTag::BegCollection)?;
                    IPPResponse::parse_collection(reader)?
                }
                tag => {
                    let (_, attr) = IPPResponse::parse_attribute(reader, tag)?;
                    attr
                }
            };

            map.insert(attr_name, value);
        }

        Ok(AttributeValue::CollectionAttribute(map))
    }

    fn parse_attribute_group<R>(
        reader: &mut R,
    ) -> Result<Vec<(DelimiterOrValueTag, Vec<(String, AttributeValue)>)>, IPPError>
    where
        R: Read,
    {
        let mut cur_attr_tag = IPPResponse::parse_tag(reader)?;
        let mut attr_groups = Vec::new();

        loop {
            let mut attrs = LinkedList::<(String, AttributeValue)>::new();
            let next_attr_tag;
            let mut end = false;

            loop {
                let tag = IPPResponse::parse_tag(reader)?;

                match tag {
                    DelimiterOrValueTag::EndOfAttributesTag => {
                        next_attr_tag = tag;
                        end = true;
                        break;
                    }
                    DelimiterOrValueTag::OperationAttributesTag
                    | DelimiterOrValueTag::JobAttributesTag
                    | DelimiterOrValueTag::PrinterAttributesTag
                    | DelimiterOrValueTag::UnsupportedAttributesTag => {
                        next_attr_tag = tag;
                        break;
                    }
                    DelimiterOrValueTag::BegCollection => {
                        let (name, _) = IPPResponse::parse_attribute(reader, tag)?;
                        attrs.push_back((name, IPPResponse::parse_collection(reader)?));
                    }
                    _ => attrs.push_back(IPPResponse::parse_attribute(reader, tag)?),
                }
            }

            let mut group = Vec::new();

            let mut attr_vec = vec![];

            // Convert additional-values to vector.
            while !attrs.is_empty() {
                let attr = attrs.pop_front().unwrap();

                loop {
                    let next = attrs.iter_mut().nth(0);
                    if next.is_some() {
                        let next = next.unwrap();
                        if next.0.len() == 0 {
                            if attr_vec.is_empty() {
                                attr_vec.push(attr.1.clone());
                            }
                            attr_vec.push(attrs.pop_front().unwrap().1);
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                if attr_vec.is_empty() {
                    group.push(attr);
                } else {
                    group.push((attr.0, AttributeValue::VectorAttribute(attr_vec)));
                    attr_vec = vec![];
                }
            }

            attr_groups.push((cur_attr_tag, group));
            if end {
                break;
            }

            cur_attr_tag = next_attr_tag;
        }

        Ok(attr_groups)
    }

    pub fn read_from_stream<R>(reader: &mut R) -> Result<IPPResponse, IPPError>
    where
        R: Read,
    {
        let mut buf = [0u8; 8];
        if let Err(err) = reader.read_exact(&mut buf) {
            return Err(IPPError::IOError(err));
        };
        let version_major = buf[0] as i8;
        let version_minor = buf[1] as i8;
        let status_code: StatusCode =
            match FromPrimitive::from_i16(i16::from_be_bytes([buf[2], buf[3]])) {
                Some(status_code) => status_code,
                None => return Err(IPPError::ProtocolError),
            };
        let request_id = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

        let attrs = IPPResponse::parse_attribute_group(reader)?;

        let mut data = Vec::<u8>::new();
        if let Err(err) = reader.read_to_end(&mut data) {
            return Err(IPPError::IOError(err));
        };

        Ok(IPPResponse {
            version_major,
            version_minor,
            status_code,
            request_id,
            attrs,
            data,
        })
    }
}

macro_rules! write_int_be {
    ($writer:ident,$var:path as $ty:ident) => {{
        let data = $ty::to_be_bytes($var as $ty);
        $writer.write(&data)
    }};
}

macro_rules! write_attr {
    ($writer:ident,$type:ident,$name:expr,$value:expr) => {
        if let Err(err) = $writer.write(&[DelimiterOrValueTag::$type as u8]) {
            Err(err)
        } else {
            if let Err(err) = $writer.write(&i16::to_be_bytes($name.len() as i16)) {
                Err(err)
            } else {
                if let Err(err) = $writer.write($name.as_bytes()) {
                    Err(err)
                } else {
                    if let Err(err) = $writer.write(&i16::to_be_bytes($value.len() as i16)) {
                        Err(err)
                    } else {
                        $writer.write($value.as_bytes())
                    }
                }
            }
        }
    };
}

fn main() -> Result<(), Box<dyn Error>> {
    let printer_addr = std::env::var("PRINTER_ADDR")
        .expect("PRINTER_ADDR is not set (should be a value like \"192.0.2.1:631\")");

    let client = Client::new();

    let mut buf = Vec::<u8>::new();

    let current_req_id = 1;

    // version-number
    buf.write(&[1u8, 1u8])?;
    // operation-id
    write_int_be!(buf, PrinterOperation::GetPrinterAttributes as i16)?;
    // request-id
    write_int_be!(buf, current_req_id as i32)?;

    // begin-attribute-group-tag
    write_int_be!(buf, DelimiterOrValueTag::OperationAttributesTag as i8)?;

    write_attr!(buf, Charset, "attributes-charset", "utf-8")?;
    write_attr!(buf, NaturalLanguage, "attributes-natural-language", "ja-jp")?;
    write_attr!(buf, Uri, "printer-uri", format!("ipp://{}", printer_addr))?;

    // end-of-attributes
    write_int_be!(buf, DelimiterOrValueTag::EndOfAttributesTag as i8)?;

    let mut resp = client
        .post(format!("http://{}", printer_addr))
        .header("Content-Type", "application/ipp")
        .body(buf)
        .send()?;

    println!("{:?}", IPPResponse::read_from_stream(&mut resp)?);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_collection() {
        let mut buf = Vec::<u8>::new();

        // version-number
        buf.write(&[1u8, 1u8]).unwrap();
        // status-code
        write_int_be!(buf, StatusCode::SuccessfulOk as i16).unwrap();
        // request-id
        let req_id = 1;
        write_int_be!(buf, req_id as i32).unwrap();

        // begin-attribute-group-tag
        write_int_be!(buf, DelimiterOrValueTag::OperationAttributesTag as i8).unwrap();

        write_attr!(buf, Charset, "attributes-charset", "utf-8").unwrap();
        write_attr!(buf, NaturalLanguage, "attributes-natural-language", "ja-jp").unwrap();
        write_attr!(buf, Uri, "array", "ipp://192.0.2.1:631").unwrap();
        write_attr!(buf, Uri, "", "ipp://192.0.2.2:631").unwrap();
        write_attr!(buf, Uri, "", "ipp://192.0.2.3:631").unwrap();

        write_attr!(buf, BegCollection, "collection", "").unwrap();

        write_attr!(buf, MemberAttrName, "", "key1").unwrap();
        write_attr!(buf, Keyword, "", "value1").unwrap();

        write_attr!(buf, MemberAttrName, "", "key2").unwrap();
        write_attr!(buf, BegCollection, "", "").unwrap();
        write_attr!(buf, MemberAttrName, "", "key2-1").unwrap();
        write_attr!(buf, Keyword, "", "value2-1").unwrap();
        write_attr!(buf, EndCollection, "", "").unwrap();

        write_attr!(buf, EndCollection, "", "").unwrap();

        // end-of-attributes
        write_int_be!(buf, DelimiterOrValueTag::EndOfAttributesTag as i8).unwrap();

        let mut reader = &buf[..];
        let response = IPPResponse::read_from_stream(&mut reader).unwrap();

        let expected_resp = IPPResponse {
            version_major: 1,
            version_minor: 1,
            status_code: StatusCode::SuccessfulOk,
            request_id: req_id,
            attrs: vec![(
                DelimiterOrValueTag::OperationAttributesTag,
                vec![
                    (
                        "attributes-charset".to_string(),
                        AttributeValue::Charset("utf-8".to_string()),
                    ),
                    (
                        "attributes-natural-language".to_string(),
                        AttributeValue::NaturalLanguage("ja-jp".to_string()),
                    ),
                    (
                        "array".to_string(),
                        AttributeValue::VectorAttribute(vec![
                            AttributeValue::Uri("ipp://192.0.2.1:631".to_string()),
                            AttributeValue::Uri("ipp://192.0.2.2:631".to_string()),
                            AttributeValue::Uri("ipp://192.0.2.3:631".to_string()),
                        ]),
                    ),
                    (
                        "collection".to_string(),
                        AttributeValue::CollectionAttribute(
                            [
                                (
                                    "key1".to_string(),
                                    AttributeValue::Keyword("value1".to_string()),
                                ),
                                (
                                    "key2".to_string(),
                                    AttributeValue::CollectionAttribute(
                                        [(
                                            "key2-1".to_string(),
                                            AttributeValue::Keyword("value2-1".to_string()),
                                        )]
                                        .into_iter()
                                        .collect::<HashMap<_, _>>(),
                                    ),
                                ),
                            ]
                            .into_iter()
                            .collect::<HashMap<_, _>>(),
                        ),
                    ),
                ],
            )],
            data: vec![],
        };

        assert_eq!(expected_resp, response);
    }
}
