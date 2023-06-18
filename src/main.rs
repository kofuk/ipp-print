use reqwest::blocking::Client;
use std::error::Error;
use std::io::prelude::*;

#[macro_use]
extern crate num_derive;

mod ipp;
use crate::ipp::*;

fn print_sample_page() -> Result<(), Box<dyn Error>> {
    let printer_addr = std::env::var("PRINTER_ADDR")
        .expect("PRINTER_ADDR is not set (should be a value like \"192.0.2.1:631\")");

    let client = Client::new();
    let mut buf = Vec::new();

    // Get-Printer-Attributes
    IPPRequest {
        version_major: 1,
        version_minor: 1,
        operation_id: PrinterOperation::GetPrinterAttributes,
        request_id: 1,
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
                    "printer-uri".to_string(),
                    AttributeValue::Uri(format!("ipp://{}", printer_addr)),
                ),
            ],
        )],
        data: vec![],
    }
    .write_to_stream(&mut buf)?;

    println!(
        "{:?}",
        IPPResponse::read_from_stream(
            &mut client
                .post(format!("http://{}", printer_addr))
                .header("Content-Type", "application/ipp")
                .body(buf)
                .send()?
        )?
    );

    buf = Vec::new();

    // Validate-Job (like 4.2.1.1. Print-Job Request)
    IPPRequest {
        version_major: 1,
        version_minor: 1,
        operation_id: PrinterOperation::ValidateJob,
        request_id: 2,
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
                    "printer-uri".to_string(),
                    AttributeValue::Uri(format!("ipp://{}", printer_addr)),
                ),
                (
                    "requesting-user-name".to_string(),
                    AttributeValue::NameWithoutLanguage(std::env::var("USER")?),
                ),
                (
                    "document-format".to_string(),
                    AttributeValue::MimeMediaType("image/jpeg".to_string()),
                ),
            ],
        )],
        data: vec![],
    }
    .write_to_stream(&mut buf)?;

    println!(
        "{:?}",
        IPPResponse::read_from_stream(
            &mut client
                .post(format!("http://{}", printer_addr))
                .header("Content-Type", "application/ipp")
                .body(buf)
                .send()?
        )?
    );

    buf = Vec::new();

    // Create-Job
    IPPRequest {
        version_major: 1,
        version_minor: 1,
        operation_id: PrinterOperation::CreateJob,
        request_id: 2,
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
                    "printer-uri".to_string(),
                    AttributeValue::Uri(format!("ipp://{}", printer_addr)),
                ),
                (
                    "requesting-user-name".to_string(),
                    AttributeValue::NameWithoutLanguage(std::env::var("USER")?),
                ),
            ],
        )],
        data: vec![],
    }
    .write_to_stream(&mut buf)?;

    let create_job_resp = IPPResponse::read_from_stream(
        &mut client
            .post(format!("http://{}", printer_addr))
            .header("Content-Type", "application/ipp")
            .body(buf)
            .send()?,
    )?;
    println!("{:?}", create_job_resp);

    if create_job_resp.attrs.len() < 2 {
        panic!("Create-Job response doesn't contain required attribute group");
    }
    let job_id = *match create_job_resp.attrs[1]
        .1
        .iter()
        .find(|(key, _)| key == "job-id")
    {
        Some((_, AttributeValue::Integer(val))) => val,
        _ => panic!("job-id was not found"),
    };

    buf = Vec::new();

    // Send-Document
    let doc_data = {
        let mut buf = Vec::new();
        let mut file = std::fs::File::open("data/sample.urf")?;
        file.read_to_end(&mut buf)?;
        buf
    };
    IPPRequest {
        version_major: 1,
        version_minor: 1,
        operation_id: PrinterOperation::SendDocument,
        request_id: 2,
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
                    "printer-uri".to_string(),
                    AttributeValue::Uri(format!("ipp://{}", printer_addr)),
                ),
                ("job-id".to_string(), AttributeValue::Integer(job_id)),
                (
                    "requesting-user-name".to_string(),
                    AttributeValue::NameWithoutLanguage(std::env::var("USER")?),
                ),
                (
                    "document-format".to_string(),
                    AttributeValue::MimeMediaType("image/urf".to_string()),
                ),
                ("last-document".to_string(), AttributeValue::Boolean(true)),
            ],
        )],
        data: doc_data,
    }
    .write_to_stream(&mut buf)?;

    println!(
        "{:?}",
        IPPResponse::read_from_stream(
            &mut client
                .post(format!("http://{}", printer_addr))
                .header("Content-Type", "application/ipp")
                .body(buf)
                .send()?,
        )?
    );

    buf = Vec::new();

    // Get-Jobs
    IPPRequest {
        version_major: 1,
        version_minor: 1,
        operation_id: PrinterOperation::GetJobs,
        request_id: 2,
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
                    "printer-uri".to_string(),
                    AttributeValue::Uri(format!("ipp://{}", printer_addr)),
                ),
                (
                    "requesting-user-name".to_string(),
                    AttributeValue::NameWithoutLanguage(std::env::var("USER")?),
                ),
            ],
        )],
        data: vec![],
    }
    .write_to_stream(&mut buf)?;

    println!(
        "{:?}",
        IPPResponse::read_from_stream(
            &mut client
                .post(format!("http://{}", printer_addr))
                .header("Content-Type", "application/ipp")
                .body(buf)
                .send()?,
        )?
    );

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    print_sample_page()
}
