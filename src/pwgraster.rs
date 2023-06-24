use std::error::Error;
use std::io::prelude::*;

// Heavily commented based on CUPS Raster Format documentation.
// It may be inaccurate.

struct PWGRaster {
    /// Ras2 if big-endian, 2SaR if little endian
    sync_word: [u8; 4],
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
    /// 0: Do not insert separator sheets
    /// 1: Insert separator sheets
    ///
    /// ???  What "separator sheets" means? Simply insert paper which is not printed, or hardware-specific "separator" exists?
    insert_sheet: u32,
    /// 0: Do not jog pages
    /// 1: Jog pages after file
    /// 2: Jog pages after job
    /// 3: Jog pages after set
    ///
    /// ???  What is "Jog"? To wait for ink to dry?
    jog: u32,
    /// 0: Top edge is first
    /// 1: Right edge is first
    /// 2: Bottom edge is first
    /// 3: Left edge is first
    ///
    /// ???  What this parameter means? Direction of printing?
    leading_edge: u32,
    reserved_2: [u8; 12],
    /// Input slot position from 0 to N
    ///
    /// ???  What this parameter means? Place the paper is set?
    media_position: u32,
    /// Media weight in g/m^2.
    /// 0 for default.
    media_weight_metric: u32,
    reserved_3: [u8; 8],
    /// The number of sheets to print
    /// 0 for printer default.
    num_copies: u32,
}

pub fn read_raster<R>(reader: &mut R) -> Result<(), Box<dyn Error>>
where
    R: Read,
{
    let mut n_read = 0;

    let mut buf = [0u8; 4];
    n_read += buf.len();
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
    println!("TotalPageCount={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("CrossFeedTransform={}", u32::from_be_bytes(buf));

    let mut buf = [0u8; 4];
    reader.read_exact(&mut buf)?;
    n_read += buf.len();
    println!("FeedTransform={}", u32::from_be_bytes(buf));

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
    println!("AlternatePrimary={}", u32::from_be_bytes(buf));

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

    Ok(())
}
