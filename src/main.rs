use num::FromPrimitive;
use reqwest::blocking::Client;
use std::error::Error;
use std::io::prelude::*;

#[macro_use]
extern crate num_derive;

// IPP/1.1
// https://www.rfc-editor.org/rfc/rfc8010.html
// https://www.rfc-editor.org/rfc/inline-errata/rfc8011.html

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

#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(unused)]
enum DelimiterTag {
    OperationAttributesTag = 0x01,
    JobAttributesTag = 0x02,
    EndOfAttributesTag = 0x03,
    PrinterAttributesTag = 0x04,
    UnsupportedAttributesTag = 0x05,
}

#[derive(Debug, FromPrimitive, ToPrimitive)]
#[allow(unused)]
enum AttributeSyntax {
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

macro_rules! write_int_be {
    ($writer:ident,$var:path as $ty:ident) => {{
        let data = $ty::to_be_bytes($var as $ty);
        $writer.write(&data)
    }};
}

macro_rules! write_attr {
    ($writer:ident,$type:ident,$name:expr,$value:expr) => {
        if let Err(err) = $writer.write(&[AttributeSyntax::$type as u8]) {
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

macro_rules! read_and_decode {
    ($reader:ident,String) => {{
        let len = read_and_decode!($reader, i16)?;
        let mut buf = vec![0u8; len as usize];
        $reader.read_exact(buf.as_mut_slice())?;
        String::from_utf8(buf)
    }};

    ($reader:ident,$type:ident) => {{
        let mut buf = [0u8; ($type::BITS / 8) as usize];
        match $reader.read_exact(&mut buf) {
            Ok(_) => Ok($type::from_be_bytes(buf)),
            Err(err) => Err(err),
        }
    }};
}

fn parse_response(data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    let mut data = &data[..];

    let version_major = read_and_decode!(data, i8)?;
    let version_minor = read_and_decode!(data, i8)?;
    println!("version: {}.{}", version_major, version_minor);

    let status_code: StatusCode = FromPrimitive::from_i16(read_and_decode!(data, i16)?).unwrap();
    println!("status-code: {:?}", status_code);

    let request_id = read_and_decode!(data, i32)?;
    println!("request-id: {}", request_id);

    let begin_attribute_group_tag: DelimiterTag =
        FromPrimitive::from_i8(read_and_decode!(data, i8)?).unwrap();
    println!("begin-attribute-group-tag: {:?}", begin_attribute_group_tag);

    let value_tag: AttributeSyntax = FromPrimitive::from_i8(read_and_decode!(data, i8)?).unwrap();
    println!("value-tag: {:?}", value_tag);

    let name = read_and_decode!(data, String)?;
    println!("name: {}", name);

    let value = read_and_decode!(data, String)?;
    println!("value: {}", value);

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let printer_addr = std::env::var("PRINTER_ADDR").expect("PRINTER_ADDR is not set");

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
    write_int_be!(buf, DelimiterTag::OperationAttributesTag as i8)?;

    write_attr!(buf, Charset, "attributes-charset", "utf-8")?;
    write_attr!(buf, NaturalLanguage, "attributes-natural-language", "ja-jp")?;
    write_attr!(buf, Uri, "printer-uri", format!("ipp://{}", printer_addr))?;

    // end-of-attributes
    write_int_be!(buf, DelimiterTag::EndOfAttributesTag as i8)?;

    let resp = client
        .post(format!("http://{}", printer_addr))
        .header("Content-Type", "application/ipp")
        .body(buf)
        .send()?;

    let resp = resp.bytes().unwrap().to_vec();
    parse_response(resp)
}
