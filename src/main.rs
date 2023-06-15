use num::FromPrimitive;
use reqwest::blocking::Client;
use std::collections::HashMap;
use std::error::Error;
use std::io::prelude::*;
use std::ops::Range;

#[macro_use]
extern crate num_derive;

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

#[derive(Debug, FromPrimitive, ToPrimitive, PartialEq, Eq)]
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

#[derive(Debug, FromPrimitive, ToPrimitive)]
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
#[allow(unused)]
struct StringWithLanguage {
    lang: String,
    string: String,
}

impl StringWithLanguage {
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        let len = buf.len();
        if len < 2 {
            panic!();
        }
        let lang_len = i16::from_be_bytes([buf[0], buf[1]]) as usize;
        if len < 2 + lang_len + 2 {
            panic!();
        }
        let str_len = i16::from_be_bytes([buf[2 + lang_len], buf[2 + lang_len + 1]]) as usize;
        if len < 2 + lang_len + 2 + str_len {
            panic!();
        }

        Ok(Self {
            lang: String::from_utf8(buf[2..(2 + lang_len)].to_vec()).unwrap(),
            string: String::from_utf8(
                buf[(2 + lang_len + 2)..(2 + lang_len + 2 + str_len)].to_vec(),
            )
            .unwrap(),
        })
    }
}

#[derive(Debug)]
#[allow(unused)]
struct Resolution {
    resolution_cross_feed: i32,
    resolution_feed: i32,
    units: i8,
}

impl Resolution {
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if buf.len() != 9 {
            panic!();
        }

        Ok(Self {
            resolution_cross_feed: i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]),
            resolution_feed: i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]),
            units: buf[8] as i8,
        })
    }
}

#[derive(Debug)]
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
    fn parse_buffer(buf: Vec<u8>) -> Result<Self, Box<dyn Error>> {
        if buf.len() != 11 {
            panic!();
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
}

#[derive(Debug)]
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

fn decode_attribute_value(
    value_type: DelimiterOrValueTag,
    buf: Vec<u8>,
) -> Result<AttributeValue, Box<dyn Error>> {
    match value_type {
        DelimiterOrValueTag::Unsupported => Ok(AttributeValue::Unsupported(buf)),
        DelimiterOrValueTag::Unknown => Ok(AttributeValue::Unknown(buf)),
        DelimiterOrValueTag::NoValue => {
            if buf.is_empty() {
                Ok(AttributeValue::NoValue)
            } else {
                panic!();
            }
        }
        DelimiterOrValueTag::Integer => {
            if buf.len() == 4 {
                Ok(AttributeValue::Integer(i32::from_be_bytes([
                    buf[0], buf[1], buf[2], buf[3],
                ])))
            } else {
                panic!();
            }
        }
        DelimiterOrValueTag::Boolean => {
            if buf.len() == 1 {
                Ok(AttributeValue::Boolean(buf[0] == 1u8))
            } else {
                panic!();
            }
        }
        DelimiterOrValueTag::Enum => {
            if buf.len() == 4 {
                Ok(AttributeValue::Enum(i32::from_be_bytes([
                    buf[0], buf[1], buf[2], buf[3],
                ])))
            } else {
                panic!();
            }
        }
        DelimiterOrValueTag::OctetStringUnspecified => Ok(AttributeValue::OctetStringUnspecified(
            String::from_utf8(buf)?,
        )),
        DelimiterOrValueTag::DateTime => Ok(AttributeValue::DateTime(DateTime::parse_buffer(buf)?)),
        DelimiterOrValueTag::Resolution => {
            Ok(AttributeValue::Resolution(Resolution::parse_buffer(buf)?))
        }
        DelimiterOrValueTag::RangeOfInteger => {
            if buf.len() != 8 {
                panic!();
            }
            let start = i32::from_be_bytes([buf[0], buf[1], buf[2], buf[3]]);
            let end = i32::from_be_bytes([buf[4], buf[5], buf[6], buf[7]]);

            Ok(AttributeValue::RangeOfInteger(start..end))
        }
        DelimiterOrValueTag::BegCollection => {
            if !buf.is_empty() {
                panic!();
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
                panic!();
            }
            Ok(AttributeValue::EndCollection)
        }
        DelimiterOrValueTag::TextWithoutLanguage => {
            Ok(AttributeValue::TextWithoutLanguage(String::from_utf8(buf)?))
        }
        DelimiterOrValueTag::NameWithoutLanguage => {
            Ok(AttributeValue::NameWithoutLanguage(String::from_utf8(buf)?))
        }
        DelimiterOrValueTag::Keyword => Ok(AttributeValue::Keyword(String::from_utf8(buf)?)),
        DelimiterOrValueTag::Uri => Ok(AttributeValue::Uri(String::from_utf8(buf)?)),
        DelimiterOrValueTag::UriScheme => Ok(AttributeValue::UriScheme(String::from_utf8(buf)?)),
        DelimiterOrValueTag::Charset => Ok(AttributeValue::Charset(String::from_utf8(buf)?)),
        DelimiterOrValueTag::NaturalLanguage => {
            Ok(AttributeValue::NaturalLanguage(String::from_utf8(buf)?))
        }
        DelimiterOrValueTag::MimeMediaType => {
            Ok(AttributeValue::MimeMediaType(String::from_utf8(buf)?))
        }
        DelimiterOrValueTag::MemberAttrName => {
            Ok(AttributeValue::MemberAttrName(String::from_utf8(buf)?))
        }

        _ => panic!(),
    }
}

fn parse_attribute<R>(
    reader: &mut R,
    value_type: DelimiterOrValueTag,
) -> Result<(String, AttributeValue), Box<dyn Error>>
where
    R: Read,
{
    let mut buf = [0u8; 2];
    reader.read_exact(&mut buf)?;
    let name_len = i16::from_be_bytes(buf);
    let mut name_buf = vec![0u8; name_len as usize];
    reader.read_exact(name_buf.as_mut_slice())?;
    let name = String::from_utf8(name_buf)?;

    reader.read_exact(&mut buf)?;
    let value_len = i16::from_be_bytes(buf);
    let mut value_buf = vec![0u8; value_len as usize];
    reader.read_exact(value_buf.as_mut_slice())?;

    let value = decode_attribute_value(value_type, value_buf)?;

    Ok((name, value))
}

fn parse_tag<R>(reader: &mut R) -> Result<DelimiterOrValueTag, Box<dyn Error>>
where
    R: Read,
{
    let mut buf = [0u8; 1];
    reader.read_exact(&mut buf)?;

    match FromPrimitive::from_u8(buf[0]) {
        Some(value) => Ok(value),
        None => panic!("invalid tag: {}", buf[0]),
    }
}

fn parse_attribute_group<R>(
    reader: &mut R,
    attribute_group_type: DelimiterOrValueTag,
) -> Result<Vec<(String, AttributeValue)>, Box<dyn Error>>
where
    R: Read,
{
    let beg_attr_tag = parse_tag(reader)?;
    if beg_attr_tag != attribute_group_type {
        panic!();
    }

    let mut result = Vec::new();

    loop {
        match parse_tag(reader)? {
            DelimiterOrValueTag::EndOfAttributesTag => break,
            DelimiterOrValueTag::OperationAttributesTag
            | DelimiterOrValueTag::JobAttributesTag
            | DelimiterOrValueTag::PrinterAttributesTag
            | DelimiterOrValueTag::UnsupportedAttributesTag => panic!(),
            tag => {
                let attr = parse_attribute(reader, tag)?;
                result.push(attr);
            }
        }
    }

    // TODO: cope with collections and addtional values

    Ok(result)
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

    let resp = client
        .post(format!("http://{}", printer_addr))
        .header("Content-Type", "application/ipp")
        .body(buf)
        .send()?;

    let resp = resp.bytes().unwrap().to_vec();
    parse_attribute_group(&mut &resp[..], DelimiterOrValueTag::OperationAttributesTag)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_collection() {
        let mut buf = Vec::<u8>::new();

        // begin-attribute-group-tag
        write_int_be!(buf, DelimiterOrValueTag::OperationAttributesTag as i8).unwrap();

        write_attr!(buf, Charset, "attributes-charset", "utf-8").unwrap();
        write_attr!(buf, NaturalLanguage, "attributes-natural-language", "ja-jp").unwrap();
        write_attr!(buf, Uri, "printer-uri", "ipp://192.0.2.1:631").unwrap();

        write_attr!(buf, BegCollection, "collection", "").unwrap();
        write_attr!(buf, MemberAttrName, "", "key1").unwrap();
        write_attr!(buf, Keyword, "", "value1").unwrap();
        write_attr!(buf, EndCollection, "", "").unwrap();

        // end-of-attributes
        write_int_be!(buf, DelimiterOrValueTag::EndOfAttributesTag as i8).unwrap();

        let mut reader = &buf[..];
        let attr_group =
            parse_attribute_group(&mut reader, DelimiterOrValueTag::OperationAttributesTag)
                .unwrap();

        assert_eq!(attr_group.len(), 4);
    }
}
