# IPP

## Current Status

- Send IPP request
- Receive IPP response and parse it to Rust struct
- Print sample URF raster data

## TODO

- [ ] Generate raster data

## About IPP and Driverless Printing

There are some specifications based upon IPP.

- AirPrint: Transfers page data with Apple Raster (image/urf), JPEG, and PDF.
- IPP Everywhere: Transfers page data with PWG Raster, JPEG, and PDF.
- Mopria: Transfers page data with PCLm (raster-only PDF subset), PWG Raster, and PDF.
- Wi-Fi Direct Print Services: Transfers page data with PCLm, PWG Raster, and PDF.

We should be able to get supported format by calling `Get-Printer-Attributes`,
so I expect that driverless printing could be implemented by treating printers
that support these standards as regular IPP devices.

## Reference

- [RFC 8010: Internet Printing Protocol/1.1: Encoding and Transport](https://www.rfc-editor.org/rfc/rfc8010.html)
- [RFC 8011: Internet Printing Protocol/1.1: Model and Semantics](https://www.rfc-editor.org/rfc/inline-errata/rfc8011.html)
- [PWG5102.4: PWG Raster Format \[PDF\]](https://ftp.pwg.org/pub/pwg/candidates/cs-ippraster10-20120420-5102.4.pdf)
- [CUPS Raster Format](https://www.cups.org/doc/spec-raster.html) (PWG Raster is subset of CUPS Raster v2))
- [PWG Raster sample files](https://ftp.pwg.org/pub/pwg/ipp/examples/)
- [PWG5101.1: PWG Media Standardized Names 2.0 \[PDF\]](https://ftp.pwg.org/pub/pwg/candidates/cs-pwgmsn20-20130328-5101.1.pdf)
- [Driverless Printing Standards And their PDLS](https://openprinting.github.io/driverless/01-standards-and-their-pdls/)
