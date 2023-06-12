# IPP

## TODO

- [ ] Parse collection attributes as a map (or a similar data structure)
- [ ] Implement interface to access parsed data structure
    - It might be good if it has a struct which has each attribute group as member
- [ ] Implement interface to construct data which send to a printer.
- [ ] Send requests like job creation to a printer

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
- [Internet Printing Protocol/1.1: Model and Semantics](https://www.rfc-editor.org/rfc/inline-errata/rfc8011.html)
- [Driverless Printing Standards And their PDLS](https://openprinting.github.io/driverless/01-standards-and-their-pdls/)
