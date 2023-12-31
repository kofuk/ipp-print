use std::error::Error;
use std::fs::File;
use std::io::prelude::*;

#[derive(Debug)]
pub struct PageHeader {
    /// NUL-terminated string saying "PwgRaster".
    pwg_raster: [u8; 64],
    /// NUL-terminated string indicating media color name.
    /// Empty string for default value.
    media_color: [u8; 64],
    /// NUL-terminated string indicating media type name.
    /// Empty string for default value.
    media_type: [u8; 64],
    print_content_optimize: [u8; 64],
    reserved_0: [u8; 12],
    /// (Maybe) when to cut paper.
    /// 0: Never cut media
    /// 1: Cut roll after file
    /// 2: Cut roll after job
    /// 3: Cut roll after set
    /// 4: Cut roll after page
    /// For plain papers, it should be ok to leave this 0.
    cut_media: u32,
    /// 0: Single-sided
    /// 1: Double-sided
    duplex: u32,
    /// [0]: Horizontal dpi
    /// [1]: Vertical dpi
    hw_resolution: [u32; 2],
    reserved_1: [u8; 16],
    /// Insert a single blank sheet prior to the current page.
    /// 0: Do not insert separator sheets
    /// 1: Insert separator sheets
    insert_sheet: u32,
    /// Offset pages in the output bin.
    /// 0: Do not jog pages
    /// 1: Jog pages after file
    /// 2: Jog pages after job
    /// 3: Jog pages after set
    jog: u32,
    /// 0: Short edge first
    /// 1: Long edge first
    ///
    /// ???  What this parameter means? Direction of printing?
    leading_edge: u32,
    reserved_2: [u8; 12],
    /// Where the paper placed.
    /// 0: Auto
    /// 1: Main
    /// 2: Secondary or alternate source
    /// 3: Large capacity source
    /// 4: Manual
    /// 5: Envelope feed
    /// 6: CD/DVD/Bluray disc
    /// 7: Photo
    /// 8: Hagaki
    /// 9: Main roll
    /// 10: Alternate roll
    /// 11: Topmost source
    /// 12: Middle source
    /// 13: Bottommost source
    /// 14: Side source
    /// 15: Leftmost source
    /// 16: Rightmost source
    /// 17: Center source
    /// 18: Rear source
    /// 19: By-pass or multipurpose source
    /// 20..39: Tray {1..20}
    /// 40..49: Roll {1..10}
    media_position: u32,
    /// Media weight in g/m^2.
    /// 0 for default.
    media_weight_metric: u32,
    reserved_3: [u8; 8],
    /// The number of sheets to print
    /// 0 for printer default.
    num_copies: u32,
    /// 0: Portrait (Do not rotate)
    /// 1: Landscape (Rotate 90 degrees CCW)
    /// 2: Reverse portrait (Rotate 180 degrees; up side down)
    /// 3: Reverse landscape (Rotate 90 degrees CW)
    orientation: u32,
    reserved_4: [u8; 4],
    /// [0]: Width in point
    /// [1]: Height in point
    /// Standardized in PWG5101.1
    page_size: [u32; 2],
    reserved_5: [u8; 8],
    /// Rotate the back page when printing on both sides of the page
    /// 0: Do not rotate (two-sided long edge if duplex=true)
    /// 1: Rotate (two-sided short edge if duplex=true)
    tumble: u32,
    /// Page width in pixels
    width: u32,
    /// Page height in pixels
    height: u32,
    reserved_6: [u8; 4],
    /// Bits per color
    /// 1, 2, 4, 8, 16 are valid.
    bits_per_color: u32,
    /// Bytes per pixel
    bits_per_pixel: u32,
    /// Uncompressed bytes per line.
    /// If this is 8-bit sRGB image, this will be a value of width * 3.
    bytes_per_line: u32,
    /// 0: chunky pixels (CMYK CMYK CMYK)
    color_order: u32,
    /// 1: RGB
    /// 3: black
    /// 6: CMYK
    /// 18: sGray (gray using sRGB gamma/white point)
    /// 19: sRGB
    /// 20: AdobeRGB
    /// 48..62: ICC{1..15} (CIE Lab with hint for {1..15} color)
    color_space: u32,
    reserved_7: [u8; 16],
    /// The number of colors
    num_colors: u32,
    reserved_8: [u8; 28],
    /// The number of pages.
    /// 0 if the total number of pages is not known wan the file is produced.
    total_page_count: u32,
    /// 1: Normal orientation
    /// -1: reversed orientation
    cross_feed_transform: i32,
    /// 1: Normal orientation
    /// -1: reversed orientation
    feed_transform: i32,
    /// 0 if unknown
    image_box_left: u32,
    /// 0 if unknown
    image_box_top: u32,
    /// 0 if unknown
    image_box_right: u32,
    /// 0 if unknown
    image_box_bottom: u32,
    /// For bi-level or monochrome page, use this color to print output the page.
    alternate_primary: u32,
    /// 0: Default
    /// 1: Draft
    /// 2: Normal
    /// 3: High
    print_quality: u32,
    reserved_9: [u8; 20],
    vendor_identifier: u32,
    vendor_length: u32,
    /// Vendor-specific data
    vendor_data: [u8; 1088],
    reserved_10: [u8; 64],
    rendering_intent: [u8; 64],
    /// Standardized in PWG5101.1
    page_size_name: [u8; 64],
}

impl Default for PageHeader {
    fn default() -> Self {
        Self {
            pwg_raster: [0; 64],
            media_color: [0; 64],
            media_type: [0; 64],
            print_content_optimize: [0; 64],
            reserved_0: [0; 12],
            cut_media: 0,
            duplex: 0,
            hw_resolution: [300, 300],
            reserved_1: [0; 16],
            insert_sheet: 0,
            jog: 0,
            leading_edge: 0,
            reserved_2: [0; 12],
            media_position: 0,
            media_weight_metric: 0,
            reserved_3: [0; 8],
            num_copies: 0,
            orientation: 0,
            reserved_4: [0; 4],
            page_size: [595, 841],
            reserved_5: [0;8],
            tumble: 0,
            width: 2480,
            height: 3507,
            reserved_6: [0; 4],
            bits_per_color: 8,
            bits_per_pixel: 24,
            bytes_per_line: 7440,
            color_order: 0,
            color_space: 19,
            reserved_7: [0; 16],
            num_colors: 3,
            reserved_8: [0; 28],
            total_page_count: 1,
            cross_feed_transform: 1,
            feed_transform: 1,
            image_box_left: 0,
            image_box_top: 0,
            image_box_right: 0,
            image_box_bottom: 0,
            alternate_primary: 0xFFFFFF,
            print_quality: 0,
            reserved_9: [0; 20],
            vendor_identifier: 0,
            vendor_length: 0,
            vendor_data: [0; 1088],
            reserved_10: [0; 64],
            rendering_intent: [0; 64],
            page_size_name: *b"iso_a4_210x297mm\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0\0",
        }
    }
}

impl PageHeader {
    pub fn write_to_stream<W>(&self, writer: &mut W) -> Result<usize, Box<dyn Error>>
    where
        W: Write,
    {
        let mut written = 0;

        written += writer.write(&self.pwg_raster)?;
        written += writer.write(&self.media_color)?;
        written += writer.write(&self.media_type)?;
        written += writer.write(&self.print_content_optimize)?;
        written += writer.write(&self.reserved_0)?;
        written += writer.write(&self.cut_media.to_be_bytes())?;
        written += writer.write(&self.duplex.to_be_bytes())?;
        written += writer.write(&self.hw_resolution[0].to_be_bytes())?;
        written += writer.write(&self.hw_resolution[1].to_be_bytes())?;
        written += writer.write(&self.reserved_1)?;
        written += writer.write(&self.insert_sheet.to_be_bytes())?;
        written += writer.write(&self.jog.to_be_bytes())?;
        written += writer.write(&self.leading_edge.to_be_bytes())?;
        written += writer.write(&self.reserved_2)?;
        written += writer.write(&self.media_position.to_be_bytes())?;
        written += writer.write(&self.media_weight_metric.to_be_bytes())?;
        written += writer.write(&self.reserved_3)?;
        written += writer.write(&self.num_copies.to_be_bytes())?;
        written += writer.write(&self.orientation.to_be_bytes())?;
        written += writer.write(&self.reserved_4)?;
        written += writer.write(&self.page_size[0].to_be_bytes())?;
        written += writer.write(&self.page_size[1].to_be_bytes())?;
        written += writer.write(&self.reserved_5)?;
        written += writer.write(&self.tumble.to_be_bytes())?;
        written += writer.write(&self.width.to_be_bytes())?;
        written += writer.write(&self.height.to_be_bytes())?;
        written += writer.write(&self.reserved_6)?;
        written += writer.write(&self.bits_per_color.to_be_bytes())?;
        written += writer.write(&self.bits_per_pixel.to_be_bytes())?;
        written += writer.write(&self.bytes_per_line.to_be_bytes())?;
        written += writer.write(&self.color_order.to_be_bytes())?;
        written += writer.write(&self.color_space.to_be_bytes())?;
        written += writer.write(&self.reserved_7)?;
        written += writer.write(&self.num_colors.to_be_bytes())?;
        written += writer.write(&self.reserved_8)?;
        written += writer.write(&self.total_page_count.to_be_bytes())?;
        written += writer.write(&self.cross_feed_transform.to_be_bytes())?;
        written += writer.write(&self.feed_transform.to_be_bytes())?;
        written += writer.write(&self.image_box_left.to_be_bytes())?;
        written += writer.write(&self.image_box_top.to_be_bytes())?;
        written += writer.write(&self.image_box_right.to_be_bytes())?;
        written += writer.write(&self.image_box_bottom.to_be_bytes())?;
        written += writer.write(&self.alternate_primary.to_be_bytes())?;
        written += writer.write(&self.print_quality.to_be_bytes())?;
        written += writer.write(&self.reserved_9)?;
        written += writer.write(&self.vendor_identifier.to_be_bytes())?;
        written += writer.write(&self.vendor_length.to_be_bytes())?;
        written += writer.write(&self.vendor_data)?;
        written += writer.write(&self.reserved_10)?;
        written += writer.write(&self.rendering_intent)?;
        written += writer.write(&self.page_size_name)?;

        Ok(written)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SrgbColor {
    r: u8,
    g: u8,
    b: u8,
}

impl SrgbColor {
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

impl From<u32> for SrgbColor {
    fn from(value: u32) -> Self {
        SrgbColor {
            r: (value >> 16) as u8,
            g: (value >> 8) as u8,
            b: (value >> 0) as u8,
        }
    }
}

#[derive(Debug)]
pub struct ImageEncoder {
    width: u32,
    height: u32,
    prev_row: Option<Vec<SrgbColor>>,
    written_rows: u32,
    comm_rows: u8,
}

impl ImageEncoder {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            prev_row: None,
            written_rows: 0,
            comm_rows: 0,
        }
    }

    fn do_encode_row<W>(writer: &mut W, row: Vec<SrgbColor>) -> Result<usize, Box<dyn Error>>
    where
        W: Write,
    {
        if row.is_empty() {
            panic!("row mustn't be empty");
        }

        let mut comm = vec![0i16; row.len()];
        for x in (0..row.len() - 1).rev() {
            comm[x] = if row[x + 1] == row[x] {
                if comm[x + 1] < 0 {
                    1
                } else if comm[x + 1] == 127 {
                    0
                } else {
                    comm[x + 1] + 1
                }
            } else {
                if comm[x + 1] > 0 {
                    0
                } else if comm[x + 1] <= -128 {
                    0
                } else {
                    comm[x + 1] - 1
                }
            }
        }

        let mut written = 0;

        let mut x = 0;
        while x < row.len() {
            written += writer.write(&[comm[x] as u8])?;
            if comm[x] < 0 {
                for i in 0..(-comm[x] + 1) {
                    written += writer.write(&[
                        row[x + i as usize].r,
                        row[x + i as usize].g,
                        row[x + i as usize].b,
                    ])?;
                }
                x += -comm[x] as usize + 1;
            } else {
                written += writer.write(&[row[x].r, row[x].g, row[x].b])?;
                x += comm[x] as usize + 1;
            }
        }

        Ok(written)
    }

    pub fn write_row<W>(
        &mut self,
        writer: &mut W,
        row: Vec<SrgbColor>,
    ) -> Result<usize, Box<dyn Error>>
    where
        W: Write,
    {
        if row.len() != self.width as usize {
            panic!();
        }
        if self.written_rows >= self.height {
            panic!();
        }

        println!("{} {}", self.written_rows, self.height);

        if self.prev_row.is_none() {
            self.written_rows += 1;
            self.prev_row = Some(row);
            return Ok(0);
        }

        let mut written = 0;

        if self.written_rows + 1 != self.height && self.prev_row.as_ref().unwrap() == &row {
            self.comm_rows += 1;
            self.written_rows += 1;
            return Ok(0);
        }

        // TODO: handle comm rows longer than 128.

        let prev_row = self.prev_row.take().unwrap();
        self.prev_row = Some(row);

        written += writer.write(&[self.comm_rows])?;
        written += ImageEncoder::do_encode_row(writer, prev_row)?;

        self.comm_rows = 0;
        self.written_rows += 1;

        Ok(written)
    }
}

pub fn read_raster<R>(reader: &mut R) -> Result<(), Box<dyn Error>>
where
    R: Read,
{
    let mut n_read = 0;

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    println!(
        "Synchronization Word={}",
        String::from_utf8(buf.to_vec()).unwrap()
    );

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("PwgRaster={}", String::from_utf8(buf.to_vec()).unwrap());

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    // if this is empty, default one is used
    println!("MediaColor={}", String::from_utf8(buf.to_vec()).unwrap());

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("MediaType={}", String::from_utf8(buf.to_vec()).unwrap());

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!(
        "PrintContentOptimize={}",
        String::from_utf8(buf.to_vec()).unwrap()
    );

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)AdvanceDistance={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)AdvanceMedia={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)Collate={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("CutMedia={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Duplex={}", u32::from_be_bytes(buf));

    // dpi
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("HWResolution[v]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("HWResolution[h]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBoundingBox[l]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBoundingBox[b]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBoundingBox[r]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBoundingBox[t]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("InsertSheet={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Jog={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("LeadingEdge={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)Margins[l]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)Margins[r]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ManualFeed={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("MediaPosition={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("MediaWeightMetric={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)MirrorPrint={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)NegativePrint={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("NumCopies={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Orientation={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)OutputFaceUp={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("PageSize[width]={}", u32::from_be_bytes(buf));

    // size in points
    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("PageSize[length]={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)Separations={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)TraySwitch={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Tumble={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Width={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("Height={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)MediaType={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("BitsPerColor={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("BitsPerPixel={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("BytesPerLine={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ColorOrder={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ColorSpace={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)Compression={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)RowCount={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)RowFeed={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)RowStep={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("NumColors={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)BorderlessScalingFactor={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)pageSize[width]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)pageSize[length]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBBox[left]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBBox[bottom]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBBox[right]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("(cups)ImagingBBox[top]={}", f32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("TotalPageCount={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("CrossFeedTransform={}", i32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("FeedTransform={}", i32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ImageBoxLeft={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ImageBoxTop={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ImageBoxRight={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("ImageBoxBottom={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("AlternatePrimary={:x}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("PrintQuality={}", u32::from_be_bytes(buf));

    // reserved
    let mut buf = [0u8; 20];
    n_read += buf.len();
    reader.read_exact(&mut buf)?;

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("VendorIdentifier={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("VendorLength={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 1088];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("VendorData={:?}", buf);

    // reserved
    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("RenderingIntent={:?}", buf);

    let mut buf = [0u8; 64];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!(
        "PageSizeName={:?}",
        String::from_utf8(buf.to_vec()).unwrap()
    );

    println!("{}", n_read);

    let mut out = File::create("/tmp/out.ppm")?;
    writeln!(out, "P3")?;
    writeln!(out, "2480 3507")?;
    writeln!(out, "255")?;

    let mut written_rows: u32 = 0;
    loop {
        let mut buf = [0u8; 1];
        if let Err(err) = reader.read_exact(&mut buf) {
            panic!("{}", err);
        }

        let mut row = Vec::<u8>::new();

        let mut x_written: u32 = 0;

        loop {
            let mut buf = [0u8; 1];
            if let Err(err) = reader.read_exact(&mut buf) {
                panic!("{}", err)
            }
            let run_len = buf[0] as i8;
            if run_len >= 0 {
                let mut color = [0u8; 3];
                if let Err(err) = reader.read_exact(&mut color) {
                    panic!("{}", err);
                }
                for _ in 0..=run_len {
                    if x_written >= 2480 {
                        println!(
                            "warning: current line exceeded its size on line {}",
                            written_rows
                        );
                        break;
                    }
                    write!(row, "{} {} {} ", color[0], color[1], color[2])?;
                    x_written += 1;
                }
            } else {
                let run_len = -run_len;
                for _ in 0..=run_len {
                    if x_written >= 2480 {
                        println!(
                            "warning: current line exceeded its size on line {}",
                            written_rows
                        );
                        break;
                    }
                    let mut color = [0u8; 3];
                    if let Err(err) = reader.read_exact(&mut color) {
                        panic!("{}", err);
                    }
                    write!(row, "{} {} {} ", color[0], color[1], color[2])?;
                    x_written += 1;
                }
            }

            if x_written >= 2480 {
                break;
            }
        }
        writeln!(row)?;

        for _ in 0..=buf[0] {
            if written_rows >= 3507 {
                println!("warning: image too long!");
                break;
            }
            out.write(row.as_slice())?;
            written_rows += 1;
        }

        if written_rows >= 3507 {
            break;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_row() {
        let data = [
            0xFFFF00, 0x0000FF, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x00FF00, 0xFFFFFF,
        ]
        .into_iter()
        .map(|e| e.into())
        .collect::<_>();
        let mut out = Vec::new();
        ImageEncoder::do_encode_row(&mut out, data).unwrap();
        let expected_bytes = vec![
            0xFE, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x02, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0xFF,
        ];
        assert_eq!(expected_bytes, out);
    }

    #[test]
    fn encode_row_long_comm_pixels() {
        let data = [0; 200].into_iter().map(|e| e.into()).collect::<_>();
        let mut out = Vec::new();
        ImageEncoder::do_encode_row(&mut out, data).unwrap();
        let expected_bytes = vec![0x47, 0x00, 0x00, 0x00, 0x7F, 0x00, 0x00, 0x00];
        assert_eq!(expected_bytes, out);
    }

    #[test]
    fn encode_row_long_diff_pixels() {
        let data = (0..200).map(|e| e.into()).collect::<_>();
        let mut out = Vec::new();
        ImageEncoder::do_encode_row(&mut out, data).unwrap();
        let expected_bytes = vec![
            186, 0, 0, 0, 0, 0, 1, 0, 0, 2, 0, 0, 3, 0, 0, 4, 0, 0, 5, 0, 0, 6, 0, 0, 7, 0, 0, 8,
            0, 0, 9, 0, 0, 10, 0, 0, 11, 0, 0, 12, 0, 0, 13, 0, 0, 14, 0, 0, 15, 0, 0, 16, 0, 0,
            17, 0, 0, 18, 0, 0, 19, 0, 0, 20, 0, 0, 21, 0, 0, 22, 0, 0, 23, 0, 0, 24, 0, 0, 25, 0,
            0, 26, 0, 0, 27, 0, 0, 28, 0, 0, 29, 0, 0, 30, 0, 0, 31, 0, 0, 32, 0, 0, 33, 0, 0, 34,
            0, 0, 35, 0, 0, 36, 0, 0, 37, 0, 0, 38, 0, 0, 39, 0, 0, 40, 0, 0, 41, 0, 0, 42, 0, 0,
            43, 0, 0, 44, 0, 0, 45, 0, 0, 46, 0, 0, 47, 0, 0, 48, 0, 0, 49, 0, 0, 50, 0, 0, 51, 0,
            0, 52, 0, 0, 53, 0, 0, 54, 0, 0, 55, 0, 0, 56, 0, 0, 57, 0, 0, 58, 0, 0, 59, 0, 0, 60,
            0, 0, 61, 0, 0, 62, 0, 0, 63, 0, 0, 64, 0, 0, 65, 0, 0, 66, 0, 0, 67, 0, 0, 68, 0, 0,
            69, 0, 0, 70, 128, 0, 0, 71, 0, 0, 72, 0, 0, 73, 0, 0, 74, 0, 0, 75, 0, 0, 76, 0, 0,
            77, 0, 0, 78, 0, 0, 79, 0, 0, 80, 0, 0, 81, 0, 0, 82, 0, 0, 83, 0, 0, 84, 0, 0, 85, 0,
            0, 86, 0, 0, 87, 0, 0, 88, 0, 0, 89, 0, 0, 90, 0, 0, 91, 0, 0, 92, 0, 0, 93, 0, 0, 94,
            0, 0, 95, 0, 0, 96, 0, 0, 97, 0, 0, 98, 0, 0, 99, 0, 0, 100, 0, 0, 101, 0, 0, 102, 0,
            0, 103, 0, 0, 104, 0, 0, 105, 0, 0, 106, 0, 0, 107, 0, 0, 108, 0, 0, 109, 0, 0, 110, 0,
            0, 111, 0, 0, 112, 0, 0, 113, 0, 0, 114, 0, 0, 115, 0, 0, 116, 0, 0, 117, 0, 0, 118, 0,
            0, 119, 0, 0, 120, 0, 0, 121, 0, 0, 122, 0, 0, 123, 0, 0, 124, 0, 0, 125, 0, 0, 126, 0,
            0, 127, 0, 0, 128, 0, 0, 129, 0, 0, 130, 0, 0, 131, 0, 0, 132, 0, 0, 133, 0, 0, 134, 0,
            0, 135, 0, 0, 136, 0, 0, 137, 0, 0, 138, 0, 0, 139, 0, 0, 140, 0, 0, 141, 0, 0, 142, 0,
            0, 143, 0, 0, 144, 0, 0, 145, 0, 0, 146, 0, 0, 147, 0, 0, 148, 0, 0, 149, 0, 0, 150, 0,
            0, 151, 0, 0, 152, 0, 0, 153, 0, 0, 154, 0, 0, 155, 0, 0, 156, 0, 0, 157, 0, 0, 158, 0,
            0, 159, 0, 0, 160, 0, 0, 161, 0, 0, 162, 0, 0, 163, 0, 0, 164, 0, 0, 165, 0, 0, 166, 0,
            0, 167, 0, 0, 168, 0, 0, 169, 0, 0, 170, 0, 0, 171, 0, 0, 172, 0, 0, 173, 0, 0, 174, 0,
            0, 175, 0, 0, 176, 0, 0, 177, 0, 0, 178, 0, 0, 179, 0, 0, 180, 0, 0, 181, 0, 0, 182, 0,
            0, 183, 0, 0, 184, 0, 0, 185, 0, 0, 186, 0, 0, 187, 0, 0, 188, 0, 0, 189, 0, 0, 190, 0,
            0, 191, 0, 0, 192, 0, 0, 193, 0, 0, 194, 0, 0, 195, 0, 0, 196, 0, 0, 197, 0, 0, 198, 0,
            0, 199,
        ];
        assert_eq!(expected_bytes, out);
    }

    #[test]
    fn encode_image() {
        // test with sample sRGB bitmap described in the spec.

        let mut encoder = ImageEncoder::new(8, 8);

        #[rustfmt::skip]
        let image_data = [
            [0xFFFFFF, 0xFFFF00, 0xFFFF00, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF],
            [0xFFFF00, 0x0000FF, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x00FF00, 0xFFFFFF],
            [0xFFFF00, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x00FF00, 0x00FF00, 0x00FF00],
            [0xFFFF00, 0xFFFF00, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x00FF00, 0xFFFFFF],
            [0xFFFFFF, 0xFFFF00, 0xFFFF00, 0xFFFF00, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF],
            [0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF],
            [0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000],
            [0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000, 0xFF0000],
        ];

        let mut out = Vec::new();
        for row in image_data {
            encoder
                .write_row(&mut out, row.into_iter().map(|e| e.into()).collect::<_>())
                .unwrap();
        }
        let expected_bytes = vec![
            0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x02, 0xFF, 0xFF, 0x00, 0x03, 0xFF, 0xFF, 0xFF, 0x00,
            0xFE, 0xFF, 0xFF, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x02, 0xFF, 0xFF, 0xFF,
            0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x01, 0xFF, 0xFF, 0x00, 0x02, 0xFF,
            0xFF, 0xFF, 0x02, 0x00, 0xFF, 0x00, 0x00, 0x02, 0xFF, 0xFF, 0x00, 0x02, 0xFF, 0xFF,
            0xFF, 0xFF, 0x00, 0xFF, 0x00, 0xFF, 0xFF, 0xFF, 0x00, 0x00, 0xFF, 0xFF, 0xFF, 0x02,
            0xFF, 0xFF, 0x00, 0x03, 0xFF, 0xFF, 0xFF, 0x00, 0x07, 0xFF, 0xFF, 0xFF, 0x01, 0x07,
            0xFF, 0x00, 0x00,
        ];
        assert_eq!(expected_bytes, out);
    }
}
