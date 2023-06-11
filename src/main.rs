use reqwest::blocking::Client;
use std::io::prelude::*;
use std::error::Error;

// IPP/1.1
// https://www.rfc-editor.org/rfc/rfc8010.html
// https://www.rfc-editor.org/rfc/inline-errata/rfc8011.html

#[allow(unused)]
enum PrinterOperations {
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

#[allow(unused)]
enum DelimiterTags {
    OperationAttributesTag = 0x01,
    JobAttributesTag = 0x02,
    EndOfAttributesTag = 0x03,
    PrinterAttributesTag = 0x04,
    UnsupportedAttributesTag = 0x05,
}

#[allow(unused)]
enum AttributeSyntaxes {
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

macro_rules! write_int_be {
    ($writer:ident,$var:path as u8) => {
        $writer.write(&[$var as u8])
    };

    ($writer:ident,$var:path as $ty:ident) => {{
        let data = $ty::to_be_bytes($var as $ty);
        $writer.write(&data)
    }};
}

macro_rules! write_attr {
    ($writer:ident,$type:ident,$name:expr,$value:expr) => {
        if let Err(err) = $writer.write(&[AttributeSyntaxes::$type as u8]) {
            Err(err)
        } else {
            if let Err(err) = $writer.write(&u16::to_be_bytes($name.len() as u16)) {
                Err(err)
            } else {
                if let Err(err) = $writer.write($name.as_bytes()) {
                    Err(err)
                } else {
                    if let Err(err) = $writer.write(&u16::to_be_bytes($value.len() as u16)) {
                        Err(err)
                    } else {
                        $writer.write($value.as_bytes())
                    }
                }
            }
        }
    };
}

fn parse_response(data: Vec<u8>) -> Result<(), Box<dyn Error>> {
    // humans parse response maunally!!!!
    std::io::stdout().write(data.as_slice())?;
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
    write_int_be!(buf, PrinterOperations::GetPrinterAttributes as u16)?;
    // request-id
    write_int_be!(buf, current_req_id as u32)?;

    // begin-attribute-group-tag
    write_int_be!(buf, DelimiterTags::OperationAttributesTag as u8)?;

    write_attr!(buf, Charset, "attributes-charset", "utf-8")?;
    write_attr!(buf, NaturalLanguage, "attributes-natural-language", "ja-jp")?;
    write_attr!(buf, Uri, "printer-uri", format!("ipp://{}", printer_addr))?;

    // end-of-attributes
    write_int_be!(buf, DelimiterTags::EndOfAttributesTag as u8)?;

    let resp = client
        .post(format!("http://{}", printer_addr))
        .header("Content-Type", "application/ipp")
        .body(buf)
        .send()?;

    let resp = resp.bytes().unwrap().to_vec();
    parse_response(resp)
}
