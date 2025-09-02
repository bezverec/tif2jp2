use anyhow::{anyhow, Context, Result};
use clap::{Parser, ArgAction};
use std::{
    ffi::CString,
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Instant,
};
use tiff::decoder::{Decoder, DecodingResult};
use tiff::decoder::ifd::Value;
use tiff::tags::Tag;
use tiff::ColorType;
use walkdir::WalkDir;

use libc::malloc;

use openjpeg_sys::{
    opj_cparameters_t, opj_codec_t, opj_image_cmptparm_t, opj_image_t, opj_stream_t,
    opj_create_compress, opj_destroy_codec, opj_encode, opj_end_compress, opj_image_create,
    opj_image_destroy, opj_set_default_encoder_parameters, opj_setup_encoder,
    opj_start_compress, opj_stream_create_default_file_stream, opj_stream_destroy,
    opj_codec_set_threads,
    CODEC_FORMAT, COLOR_SPACE, PROG_ORDER,
};

// Rayon for parallel row processing (de-interleave)
use rayon::prelude::*;

/// Tiny logger with verbosity levels (0 = errors only, 1 = info)
struct Log { lvl: u8 }
impl Log {
    fn new(lvl: u8) -> Self { Self { lvl } }
    #[inline] fn v1(&self, msg: impl AsRef<str>) { if self.lvl >= 1 { eprintln!("{}", msg.as_ref()); } }
}

/// CLI
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "TIFF to JPEG2000 (JP2) lossless via OpenJPEG FFI",
    long_about = None
)]
pub struct Args {
    /// Input file or directory (use --recursive for subdirectories)
    pub input: PathBuf,

    /// Output file or directory (mirrors input structure if directory)
    #[arg(short, long, value_name = "OUTPUT")]
    pub output: Option<PathBuf>,

    /// Recursively traverse the input directory
    #[arg(long)]
    pub recursive: bool,

    /// Tile size, e.g. 1024x1024
    #[arg(long, default_value = "4096x4096", value_name = "WxH")]
    pub tile: String,

    /// Code-block size, e.g. 64x64
    #[arg(long, default_value = "64x64", value_name = "WxH")]
    pub block: String,

    /// Number of resolutions
    #[arg(long, default_value = "6", value_name = "NUM|auto")]
    pub levels: String,

    /// Overwrite existing output files
    #[arg(long)]
    pub force: bool,

    /// OpenJPEG threads (0 = auto = all cores)
    #[arg(long, default_value_t = 0usize, value_name = "N")]
    pub threads: usize,

    /// Path to ICC profile (overrides ICC detected in TIFF)
    #[arg(long, value_name = "PATH")]
    pub icc: Option<PathBuf>,

    // ---------- toggles: define positive + negative as SEPARATE flags ----------

    /// Write DPI into JP2 'res' box [default: on]
    #[arg(long = "dpi-box", action = ArgAction::SetTrue, overrides_with = "no-dpi-box")]
    pub dpi_box_on: bool,

    /// Disable Write DPI into JP2 'res' box
    #[arg(long = "no-dpi-box", action = ArgAction::SetTrue, overrides_with = "dpi-box")]
    pub dpi_box_off: bool,

    /// Write DPI into XMP 'uuid' box [default: on]
    #[arg(long = "xmp-dpi", action = ArgAction::SetTrue, overrides_with = "no-xmp-dpi")]
    pub xmp_dpi_on: bool,

    /// Disable Write DPI into XMP 'uuid' box
    #[arg(long = "no-xmp-dpi", action = ArgAction::SetTrue, overrides_with = "xmp-dpi")]
    pub xmp_dpi_off: bool,

    /// Enable AVX2 fast path if supported [default: off]
    #[arg(long = "avx2", action = ArgAction::SetTrue, overrides_with = "no-avx2")]
    pub avx2_on: bool,

    /// Force no AVX2
    #[arg(long = "no-avx2", action = ArgAction::SetTrue, overrides_with = "avx2")]
    pub avx2_off: bool,

    /// Enable tile-parts split by Resolution [default: on]
    #[arg(long = "tp-r", action = ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "no-tp-r", action = ArgAction::SetFalse)]
    pub tp_r: bool,

    /// Enable precincts 256x256 … 128x128 [default: on]
    #[arg(long = "precincts", action = ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "no-precincts", action = ArgAction::SetFalse)]
    pub precincts: bool,

    /// Enable SOP markers (Start of Packet) [default: on]
    #[arg(long = "sop", action = ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "no-sop", action = ArgAction::SetFalse)]
    pub sop: bool,

    /// Enable EPH markers (End of Packet Header) [default: on]
    #[arg(long = "eph", action = ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "no-eph", action = ArgAction::SetFalse)]
    pub eph: bool,

    /// Enable reversible MCT for RGB [default: on]
    #[arg(long = "mct", action = ArgAction::SetTrue, default_value_t = true)]
    #[arg(long = "no-mct", action = ArgAction::SetFalse)]
    pub mct: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', action = ArgAction::Count)]
    /// keep long form working but hidden (prevents '--verbose...' in help)
    #[arg(long = "verbose", hide = true)]
    pub verbose: u8,
}

// Normalized config used in the program
pub struct Effective {
    pub avx2: bool,     // default: false
    pub dpi_box: bool,  // default: true
    pub xmp_dpi: bool,  // default: true
}

impl Args {
    pub fn effective(&self) -> Effective {
        let dpi_box = if self.dpi_box_on {
            true
        } else if self.dpi_box_off {
            false
        } else {
            true
        };

        let xmp_dpi = if self.xmp_dpi_on {
            true
        } else if self.xmp_dpi_off {
            false
        } else {
            true
        };

        let avx2 = if self.avx2_on {
            true
        } else if self.avx2_off {
            false
        } else {
            false
        };

        Effective { avx2, dpi_box, xmp_dpi }
    }
}

fn main() -> Result<()> {
    let args = Args::parse();
    let log = Log::new(args.verbose);

    log.v1(format!("Input:  {}", args.input.display()));
    if let Some(out) = &args.output {
        log.v1(format!("Output: {}", out.display()));
    }

    // Create outpu dir if necessary
    if let Some(out_dir) = &args.output {
        if out_dir.is_dir() || (!out_dir.exists() && args.input.is_dir()) {
            fs::create_dir_all(out_dir).context("Creating output directory")?;
        }
    }

    let inputs = collect_inputs(&args.input, args.recursive, args.output.as_deref())?;
    log.v1(format!("Found {} input TIFF(s).", inputs.len()));
    
    if inputs.is_empty() {
        return Err(anyhow!("No input TIFFs found"));
    }

    for (idx, input) in inputs.iter().enumerate() {
        let out = derive_output_path(&args, input)?;
        
        // check output dir, create if not present
        if let Some(parent) = out.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Creating output subdirectory")?;
            }
        }

        if out.exists() && !args.force {
            eprintln!("Skipping (exists): {}", out.display());
            continue;
        }

        eprintln!("({}/{}) → Processing: {} -> {}", idx + 1, inputs.len(), input.display(), out.display());
        let t0 = Instant::now();

        // Isolate 1 conversion
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            convert_one(input, &out, &args)
        }));

        match result {
            Ok(Ok(())) => {
                let dt = t0.elapsed();
                eprintln!("✔ {} → {} ({:.2?})", input.display(), out.display(), dt);
            }
            Ok(Err(e)) => {
                eprintln!("✖ {} — Error: {}", input.display(), e);
            }
            Err(panic) => {
                // Better panic info catching
                if let Some(s) = panic.downcast_ref::<&str>() {
                    eprintln!("✖ {} — Panic during conversion: {}", input.display(), s);
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    eprintln!("✖ {} — Panic during conversion: {}", input.display(), s);
                } else {
                    eprintln!("✖ {} — Panic during conversion: <unknown reason>", input.display());
                }
            }
        }

        // Flush
        let _ = std::io::stderr().flush();
        let _ = std::io::stdout().flush();
        
        // Short pause for system stabilization
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    eprintln!("All files processed successfully!");
    Ok(())
}

/// Collects input TIFF files based on a root path and recursion flag.
/// Excludes the output directory (if it is inside the input root).
fn collect_inputs(root: &Path, recursive: bool, _out_dir: Option<&Path>) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if root.is_file() {
        if is_tiff(root) {
            files.push(root.to_path_buf());
            eprintln!("Found file: {}", root.display());
        }
    } else if root.is_dir() {
        eprintln!("Scanning directory: {}", root.display());
        
        let walker = if recursive {
            WalkDir::new(root).min_depth(1)
        } else {
            WalkDir::new(root).max_depth(1).min_depth(1)
        };

        for entry in walker.into_iter().filter_map(Result::ok) {
            let path = entry.path();
            if path.is_file() && is_tiff(path) {
                eprintln!("Found TIFF: {}", path.display());
                files.push(path.to_path_buf());
            }
        }
    }

    eprintln!("Total files found: {}", files.len());
    files.sort();
    Ok(files)
}

/// Basic extension check (.tif / .tiff), case-insensitive.
fn is_tiff(p: &Path) -> bool {
    matches!(
        p.extension().and_then(|s| s.to_str()).map(|s| s.to_ascii_lowercase()),
        Some(ref ext) if ext == "tif" || ext == "tiff"
    )
}

/// Derives output path from args and input, handling file/dir modes.
fn derive_output_path(args: &Args, input: &Path) -> Result<PathBuf> {
    let result = match &args.output {
        Some(out) => {
            if out.is_dir() || (!out.exists() && args.input.is_dir()) {
                let file_name = input.file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let mut output_path = out.clone();
                output_path.push(format!("{}.jp2", file_name));
                output_path
            } else {
                out.clone()
            }
        }
        None => {
            input.with_extension("jp2")
        }
    };
    
    // Path normalization for Windows
    let normalized_path = result.to_string_lossy().replace('\\', "/");
    let normalized_path = PathBuf::from(normalized_path);
    
    eprintln!("Input: {} -> Output: {}", input.display(), normalized_path.display());
    Ok(normalized_path)
}

/// Parses "WxH" string into (width, height).
fn parse_wh(s: &str) -> Result<(u32, u32)> {
    let (w, h) = s.split_once('x').ok_or_else(|| anyhow!("Use format WxH, e.g. 1024x1024"))?;
    Ok((w.parse()?, h.parse()?))
}

/// Heuristic for number of wavelet resolution levels (clamped to 3..=8).
fn auto_levels(w: u32, h: u32) -> u32 {
    let m = w.min(h) as f32;
    let mut k = (m.log2().floor() as i32) - 1;
    if k < 1 { k = 1; }
    (k as u32).clamp(3, 8)
}

// --- TIFF metadata (DPI + ICC) ------------------------------------------------

#[derive(Clone, Copy)]
enum ResUnit { Inch, Centimeter, None }

struct TiffMeta {
    xdpi: Option<f64>,
    ydpi: Option<f64>,
    unit: ResUnit,
    icc: Option<Vec<u8>>,
}

/// Reads basic metadata: X/Y resolution + unit, and best-effort ICC.
/// Note: with tiff 0.9.x the ICC tag (34675) may come as a single Byte/Ascii,
/// which is not a full profile. For archival use, prefer --icc <path.icc>.
fn read_tiff_meta(p: &Path) -> Result<TiffMeta> {
    let f = File::open(p)?;
    let mut dec = Decoder::new(std::io::BufReader::new(f))?;

    // Resolution
    let mut xdpi = None;
    let mut ydpi = None;
    let mut unit = ResUnit::None;

    if let Ok(v) = dec.get_tag(Tag::XResolution) {
        if let Value::Rational(a, b) = v { if b != 0 { xdpi = Some(a as f64 / b as f64); } }
    }
    if let Ok(v) = dec.get_tag(Tag::YResolution) {
        if let Value::Rational(a, b) = v { if b != 0 { ydpi = Some(a as f64 / b as f64); } }
    }
    if let Ok(v) = dec.get_tag(Tag::ResolutionUnit) {
        if let Value::Short(u) = v {
            unit = match u { 2 => ResUnit::Inch, 3 => ResUnit::Centimeter, _ => ResUnit::None };
        }
    }

    // ICC (tag 34675) – often a single Byte/Ascii in tiff 0.9.x; not a full profile.
    let mut icc: Option<Vec<u8>> = None;
    if let Ok(v) = dec.get_tag(Tag::Unknown(34675)) {
        match v {
            Value::Byte(b) => icc = Some(vec![b]),
            Value::Ascii(s) => icc = Some(s.into_bytes()),
            _ => {}
        }
    }

    Ok(TiffMeta { xdpi, ydpi, unit, icc })
}

// --- OpenJPEG helpers ----------------------------------------------------------

/// Attach ICC buffer to OpenJPEG image (takes ownership via raw pointer).
unsafe fn attach_icc(img: *mut opj_image_t, icc: &[u8]) {
    if img.is_null() || icc.is_empty() { return; }
    let ptr = unsafe { malloc(icc.len()) };
    if ptr.is_null() { return; }
    unsafe {
        std::ptr::copy_nonoverlapping(icc.as_ptr(), ptr as *mut u8, icc.len());
        (*img).icc_profile_buf = ptr as *mut u8;
        (*img).icc_profile_len = icc.len() as u32;
    }
}

// --- JP2 Resolution box (embed DPI into jp2h/resc+resd) -----------------------

#[derive(Clone, Copy)]
enum PpmUnit { Inch, Centimeter }

/// Convert DPI to pixels-per-metre as used by JP2 resolution boxes.
fn dpi_to_ppm(dpi: f64, unit: PpmUnit) -> f64 {
    match unit { PpmUnit::Inch => dpi / 0.0254, PpmUnit::Centimeter => dpi / 0.01 }
}

/// Find (num, den, exp) for JP2 rational-with-decimal-exponent (16/16/8-bit),
/// approximating a floating ppm value within the allowed ranges.
fn ppm_to_jp2_triplet(ppm: f64) -> (u16, u16, u8) {
    let mut best = (0u16, 1u16, 0u8);
    let mut err_best = f64::INFINITY;
    for exp in 0..6u8 {
        let target = ppm / 10f64.powi(exp as i32);
        for &den in &[1u32, 10, 100, 1000, 10000, 65535] {
            let numf = target * (den as f64);
            if numf <= 0.0 { continue; }
            let num = numf.round() as i64;
            if num <= 0 || num > 65535 { continue; }
            let approx = (num as f64) / (den as f64) * 10f64.powi(exp as i32);
            let err = (approx - ppm).abs();
            if err < err_best {
                err_best = err; best = (num as u16, den as u16, exp);
                if err < 1e-6 { return best; }
            }
        }
    }
    best
}

/// Serialize payload for 'resc'/'resd' (vertical/horizontal ppm and exponent).
fn build_resc_resd_payload(v_ppm: f64, h_ppm: f64) -> Vec<u8> {
    let (vrn, vrd, vre) = ppm_to_jp2_triplet(v_ppm);
    let (hrn, hrd, hre) = ppm_to_jp2_triplet(h_ppm);
    let mut v = Vec::with_capacity(2+2+1 + 2+2+1 + 1);
    v.extend_from_slice(&vrn.to_be_bytes());
    v.extend_from_slice(&vrd.to_be_bytes());
    v.push(vre);
    v.extend_from_slice(&hrn.to_be_bytes());
    v.extend_from_slice(&hrd.to_be_bytes());
    v.push(hre);
    v.push(0); // reserved
    v
}

fn be_u32(b: &[u8]) -> u32 { u32::from_be_bytes([b[0], b[1], b[2], b[3]]) }
fn put_be_u32(x: u32, out: &mut Vec<u8>) { out.extend_from_slice(&x.to_be_bytes()); }

/// Inserts a 'res ' superbox with 'resc' + 'resd' into 'jp2h'.
/// If the structure is atypical (e.g., XLBox), it silently returns without changes.
fn insert_resolution_box_jp2(path: &Path, xdpi: f64, ydpi: f64, unit: ResUnit) -> Result<()> {
    let data = std::fs::read(path)?;

    // Locate 'jp2h' among top-level boxes.
    let mut off = 0usize;
    let mut jp2h_pos: Option<(usize, usize)> = None;
    while off + 8 <= data.len() {
        let lbox = be_u32(&data[off..off + 4]) as u64;
        let tbox = &data[off + 4..off + 8];
        let blen = if lbox == 0 { (data.len() - off) as u64 } else { lbox };
        if tbox == b"jp2h" { jp2h_pos = Some((off, blen as usize)); break; }
        off += blen as usize;
    }
    let (jp2h_off, jp2h_len) = match jp2h_pos { Some(v) => v, None => return Ok(()) };

    // 'jp2h' header is 8 bytes (LBox+TBox)
    let jp2h_payload_start = jp2h_off + 8;
    let jp2h_payload_end   = jp2h_off + jp2h_len;
    if jp2h_payload_end > data.len() || jp2h_payload_start > jp2h_payload_end { return Ok(()); }
    let old_payload = &data[jp2h_payload_start..jp2h_payload_end];

    // Convert DPI → PPM using TIFF unit
    let unit_ppm = match unit { ResUnit::Inch => PpmUnit::Inch, ResUnit::Centimeter => PpmUnit::Centimeter, ResUnit::None => PpmUnit::Inch };
    let v_ppm = dpi_to_ppm(ydpi, unit_ppm);
    let h_ppm = dpi_to_ppm(xdpi, unit_ppm);

    // Build 'res ' superbox with 'resc' and 'resd'
    let resc_p = build_resc_resd_payload(v_ppm, h_ppm);
    let resd_p = build_resc_resd_payload(v_ppm, h_ppm);
    let resc_len = 8 + resc_p.len() as u32;
    let resd_len = 8 + resd_p.len() as u32;

    let mut res_super = Vec::new();
    let total_res_payload = (8 + resc_len) + (8 + resd_len);
    put_be_u32(total_res_payload as u32, &mut res_super);
    res_super.extend_from_slice(b"res ");
    put_be_u32(resc_len, &mut res_super); res_super.extend_from_slice(b"resc"); res_super.extend_from_slice(&resc_p);
    put_be_u32(resd_len, &mut res_super); res_super.extend_from_slice(b"resd"); res_super.extend_from_slice(&resd_p);

    // Reconstruct file: [before jp2h] + [new jp2h with extended length] + [after jp2h]
    let new_jp2h_len = jp2h_len as u32 + res_super.len() as u32;

    let mut new = Vec::with_capacity(data.len() + res_super.len());
    new.extend_from_slice(&data[..jp2h_off]);
    put_be_u32(new_jp2h_len, &mut new); new.extend_from_slice(b"jp2h");
    new.extend_from_slice(old_payload);
    new.extend_from_slice(&res_super);
    new.extend_from_slice(&data[jp2h_payload_end..]);

    std::fs::write(path, &new)?;
    Ok(())
}

// --- XMP DPI fallback ----------------------------------------------------------

/// Builds a minimal XMP packet carrying TIFF-style resolution metadata.
fn build_xmp_with_dpi(xdpi: f64, ydpi: f64, unit: ResUnit) -> String {
    let unit_val = match unit { ResUnit::Inch => 2, ResUnit::Centimeter => 3, ResUnit::None => 1 };
    let xr = format!("{}/{}", (xdpi * 1000.0).round() as u64, 1000u64);
    let yr = format!("{}/{}", (ydpi * 1000.0).round() as u64, 1000u64);
    format!(r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description xmlns:tiff="http://ns.adobe.com/tiff/1.0/"
    tiff:XResolution="{xr}"
    tiff:YResolution="{yr}"
    tiff:ResolutionUnit="{unit_val}"/>
 </rdf:RDF>
</x:xmpmeta>"#)
}

/// Appends an XMP UUID box at the end of the JP2 file.
fn append_jp2_xmp_box(path: &Path, xmp_xml: &[u8]) -> Result<()> {
    const XMP_UUID: [u8; 16] = [0xBE,0x7A,0xCF,0xCB,0x97,0xA9,0x42,0xE8,0x9C,0x71,0x99,0x94,0x91,0xE3,0xAF,0xAC];
    let mut f = fs::OpenOptions::new().append(true).open(path)?;
    let total_len = 8u32 + 16u32 + xmp_xml.len() as u32;
    f.write_all(&total_len.to_be_bytes())?;
    f.write_all(b"uuid")?;
    f.write_all(&XMP_UUID)?;
    f.write_all(xmp_xml)?;
    Ok(())
}

// --- AVX2 fast paths -----------------------------------------------------------

#[cfg(target_arch = "x86_64")]
#[inline]
fn widen_u16_to_i32_avx2(src: &[u16], dst: &mut [i32]) -> bool {
    if !std::is_x86_feature_detected!("avx2") { return false; }
    unsafe { widen_u16_to_i32_avx2_inner(src, dst) }
    true
}
#[cfg(not(target_arch = "x86_64"))]
#[inline]
fn widen_u16_to_i32_avx2(_src: &[u16], _dst: &mut [i32]) -> bool { false }

#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "avx2")]
unsafe fn widen_u16_to_i32_avx2_inner(src: &[u16], dst: &mut [i32]) {
    use core::arch::x86_64::*;
    let mut i = 0usize;
    let n = src.len();

    while i + 16 <= n {
        unsafe {
            // Load 16×u16 as two 128-bit lanes
            let base_u16 = src.as_ptr();
            let lo128 = _mm_loadu_si128(base_u16.cast::<__m128i>().add(i));
            let hi128 = _mm_loadu_si128(base_u16.cast::<__m128i>().add(i + 8));
            // Widen to i32 vectors
            let lo = _mm256_cvtepu16_epi32(lo128);
            let hi = _mm256_cvtepu16_epi32(hi128);
            // Store to destination
            let base_i32 = dst.as_mut_ptr();
            _mm256_storeu_si256(base_i32.cast::<__m256i>().add(i), lo);
            _mm256_storeu_si256(base_i32.cast::<__m256i>().add(i + 8), hi);
        }
        i += 16;
    }

    // Scalar tail
    for k in i..n { dst[k] = src[k] as i32; }
}

/// AVX2 de-interleave for **RGB8 one row**, 8 pixels per iteration using gathers.
#[cfg(target_arch = "x86_64")]
#[allow(unsafe_op_in_unsafe_fn)]
#[target_feature(enable = "avx2")]
unsafe fn deinterleave_rgb8_row_avx2(
    src_row: *const u8,
    n: usize,
    dst_r: *mut i32,
    dst_g: *mut i32,
    dst_b: *mut i32,
) {
    use core::arch::x86_64::*;
    let mut x = 0usize;
    let m_ff = _mm256_set1_epi32(0xFF);

    while x + 8 <= n {
        // Build byte offsets for 8 consecutive pixels
        let off_r = [
            (3*(x+0)+0) as i32, (3*(x+1)+0) as i32, (3*(x+2)+0) as i32, (3*(x+3)+0) as i32,
            (3*(x+4)+0) as i32, (3*(x+5)+0) as i32, (3*(x+6)+0) as i32, (3*(x+7)+0) as i32,
        ];
        let off_g = [
            off_r[0]+1, off_r[1]+1, off_r[2]+1, off_r[3]+1,
            off_r[4]+1, off_r[5]+1, off_r[6]+1, off_r[7]+1,
        ];
        let off_b = [
            off_r[0]+2, off_r[1]+2, off_r[2]+2, off_r[3]+2,
            off_r[4]+2, off_r[5]+2, off_r[6]+2, off_r[7]+2,
        ];

        let idx_r = _mm256_loadu_si256(off_r.as_ptr() as *const __m256i);
        let idx_g = _mm256_loadu_si256(off_g.as_ptr() as *const __m256i);
        let idx_b = _mm256_loadu_si256(off_b.as_ptr() as *const __m256i);

        // Gather 32-bit words at byte offsets; mask low byte
        let base_i32 = src_row as *const i32;
        let gr = _mm256_i32gather_epi32(base_i32, idx_r, 1);
        let gg = _mm256_i32gather_epi32(base_i32, idx_g, 1);
        let gb = _mm256_i32gather_epi32(base_i32, idx_b, 1);

        let rr = _mm256_and_si256(gr, m_ff);
        let rg = _mm256_and_si256(gg, m_ff);
        let rb = _mm256_and_si256(gb, m_ff);

        // store to dst pointers using add() + cast to __m256i
        _mm256_storeu_si256(dst_r.add(x) as *mut __m256i, rr);
        _mm256_storeu_si256(dst_g.add(x) as *mut __m256i, rg);
        _mm256_storeu_si256(dst_b.add(x) as *mut __m256i, rb);

        x += 8;
    }

    // Scalar tail
    for i in x..n {
        let p = src_row.add(3*i);
        *dst_r.add(i) = *p as i32;
        *dst_g.add(i) = *p.add(1) as i32;
        *dst_b.add(i) = *p.add(2) as i32;
    }
}

// --- Pixel buffer enum ---------------------------------------------------------

enum PixelBuf { U8(Vec<u8>), U16(Vec<u16>) }

// J2K code-style flags (mirror of OpenJPEG defines)
const J2K_CCP_CSTY_PRT: i32 = 0x01; // precinct partition
const J2K_CCP_CSTY_SOP: i32 = 0x02; // SOP markers
const J2K_CCP_CSTY_EPH: i32 = 0x04; // EPH markers

/// Round up to the next power of two, respecting a minimum.
#[inline]
fn next_pow2_at_least(x: i32, min: i32) -> i32 {
    let v = x.max(min).max(1);
    let mut p = 1;
    while p < v && p < 32768 {
        p <<= 1;
    }
    p
}

/// Enable precincts and fill per-resolution sizes (256×256 ... 128×128).
/// Ensures each precinct is a power of two and >= code-block size.
fn fill_precincts(enc: &mut openjpeg_sys::opj_cparameters_t, levels: u32, cblk_w: i32, cblk_h: i32) {
    // Enable precinct partitioning
    enc.csty |= J2K_CCP_CSTY_PRT;

    // How many resolution-specific entries we define
    enc.res_spec = levels.min(32) as i32;

    // For each resolution r: 256×256, and 128×128 at the lowest resolution
    let lvls = levels.min(32);
    for r in 0..lvls {
        let is_last = r == lvls - 1;
        let mut pw = if is_last { 128 } else { 256 };
        let mut ph = if is_last { 128 } else { 256 };

        // Must be >= code-block and power of two
        pw = next_pow2_at_least(pw, cblk_w);
        ph = next_pow2_at_least(ph, cblk_h);

        enc.prcw_init[r as usize] = pw;
        enc.prch_init[r as usize] = ph;
    }
}

// --- Main conversion -----------------------------------------------------------

fn convert_one(input: &Path, output: &Path, args: &Args) -> Result<()> {
    eprintln!("  [DEBUG] Starting conversion for: {}", input.display());

    // Normalize flags once for this conversion
    let eff = args.effective();
    
    // Metadata (DPI/ICC)
    eprintln!("  [DEBUG] Reading TIFF metadata");
    let meta = read_tiff_meta(input).unwrap_or(TiffMeta { xdpi: None, ydpi: None, unit: ResUnit::None, icc: None });

    // Decode TIFF
    eprintln!("  [DEBUG] Opening TIFF file");
    let f = File::open(input).with_context(|| format!("Open {}", input.display()))?;
    
    eprintln!("  [DEBUG] Creating decoder");
    let mut dec = Decoder::new(f)?;
    
    eprintln!("  [DEBUG] Reading dimensions");
    let (w, h) = dec.dimensions()?;
    eprintln!("  [DEBUG] Dimensions: {}x{}", w, h);
    
    eprintln!("  [DEBUG] Reading color type");
    let ct = dec.colortype()?;
    eprintln!("  [DEBUG] Color type: {:?}", ct);

    let (bit_depth, channels, rgb) = match ct {
        ColorType::Gray(n) => (n as u32, 1u32, false),
        ColorType::RGB(n)  => (n as u32, 3u32, true),
        ColorType::RGBA(n) => return Err(anyhow!("TIFF has alpha channel ({}-bit). Please flatten/remove alpha.", n)),
        ColorType::CMYK(n) => return Err(anyhow!("CMYK {}-bit is not supported (convert to RGB/Gray).", n)),
        other => return Err(anyhow!("Unsupported TIFF: {:?}", other)),
    };
    eprintln!("  [DEBUG] Bit depth: {}, Channels: {}, RGB: {}", bit_depth, channels, rgb);

    eprintln!("  [DEBUG] Reading image data");
    let pixels = match dec.read_image()? {
        DecodingResult::U8(buf)  => {
            eprintln!("  [DEBUG] Buffer type: U8, size: {}", buf.len());
            PixelBuf::U8(buf)
        },
        DecodingResult::U16(buf) => {
            eprintln!("  [DEBUG] Buffer type: U16, size: {}", buf.len());
            PixelBuf::U16(buf)
        },
        _ => return Err(anyhow!("Unsupported TIFF buffer")),
    };

    eprintln!("  [DEBUG] Creating OpenJPEG image components");
    let mut cmpts: Vec<opj_image_cmptparm_t> = (0..channels)
        .map(|_| opj_image_cmptparm_t {
            dx: 1, dy: 1, w, h, x0: 0, y0: 0, prec: bit_depth, bpp: bit_depth, sgnd: 0
        })
        .collect();

    let clrspc = if rgb { COLOR_SPACE::OPJ_CLRSPC_SRGB } else { COLOR_SPACE::OPJ_CLRSPC_GRAY };

    eprintln!("  [DEBUG] Creating OpenJPEG image");
    let img: *mut opj_image_t = unsafe {
        let p = opj_image_create(channels, cmpts.as_mut_ptr(), clrspc);
        if p.is_null() { return Err(anyhow!("opj_image_create failed")); }
        (*p).x0 = 0; (*p).y0 = 0; (*p).x1 = w; (*p).y1 = h; p
    };

    // ICC: override from --icc or use best-effort TIFF ICC
    if let Some(icc_path) = &args.icc {
        eprintln!("  [DEBUG] Loading ICC profile from: {}", icc_path.display());
        let mut buf = Vec::new();
        File::open(icc_path).with_context(|| format!("Read ICC {}", icc_path.display()))?.read_to_end(&mut buf)?;
        unsafe { attach_icc(img, &buf) };
    } else if let Some(icc) = &meta.icc {
        eprintln!("  [DEBUG] Using TIFF ICC profile (size: {})", icc.len());
        unsafe { attach_icc(img, icc) };
    }

    eprintln!("  [DEBUG] Filling component planes");
    match pixels {
        PixelBuf::U8(buf)  => {
            eprintln!("  [DEBUG] Filling U8 components");
            // <<< CHANGED: pass eff.avx2 >>>
            fill_components_u8(img, &buf, w, h, channels, eff.avx2)?
        },
        PixelBuf::U16(buf) => {
            eprintln!("  [DEBUG] Filling U16 components");
            // <<< CHANGED: pass eff.avx2 >>>
            fill_components_u16(img, &buf, w, h, channels, eff.avx2)?
        },
    }

    // Encoder parameters (lossless 5/3, tiles, code-blocks, levels)
    eprintln!("  [DEBUG] Setting encoder parameters");

    // Parse tile size
    let (tile_w, tile_h) = parse_wh(&args.tile)?;

    // Parse code-block size + validate (must be power of two in range 4..=1024)
    let (blk_w, blk_h) = parse_wh(&args.block)?;
    let is_pow2 = |v: u32| v != 0 && (v & (v - 1)) == 0;
    let valid_cb = |v: u32| is_pow2(v) && (4..=1024).contains(&v);
    if !valid_cb(blk_w) || !valid_cb(blk_h) {
        return Err(anyhow!(
            "Invalid code-block size {}x{} (must be power of two in 4..=1024)",
            blk_w, blk_h
        ));
    }

    // Number of wavelet decomposition levels
    let levels: u32 = if args.levels == "auto" {
        auto_levels(w, h)
    } else {
        args.levels.parse::<u32>().context("parse --levels")?
    };

    // Initialize OpenJPEG encoder parameters with defaults
    let mut enc_params: opj_cparameters_t = unsafe {
        let mut p = std::mem::MaybeUninit::<opj_cparameters_t>::zeroed();
        opj_set_default_encoder_parameters(p.as_mut_ptr());
        p.assume_init()
    };

    // Reversible 5/3 transform (lossless)
    enc_params.irreversible = 0;

    // Enable tiling
    enc_params.tile_size_on = 1;
    enc_params.cp_tx0 = 0;
    enc_params.cp_ty0 = 0;
    enc_params.cp_tdx = tile_w as i32;
    enc_params.cp_tdy = tile_h as i32;

    // Code-block size
    enc_params.cblockw_init = blk_w as i32;
    enc_params.cblockh_init = blk_h as i32;

    // Resolution levels
    enc_params.numresolution = levels as i32;

    // Single quality layer (lossless)
    enc_params.tcp_numlayers = 1;

    // Progression order (RPCL as required)
    enc_params.prog_order = PROG_ORDER::OPJ_RPCL;

    // Enable SOP markers (Start of Packet)
    enc_params.csty |= J2K_CCP_CSTY_SOP;

    // Enable EPH markers (End of Packet Header)
    enc_params.csty |= J2K_CCP_CSTY_EPH;

    // Resolution levels
    enc_params.numresolution = levels as i32;

    // Single quality layer (lossless)
    enc_params.tcp_numlayers = 1;

    // Progression order (RPCL as required)
    enc_params.prog_order = PROG_ORDER::OPJ_RPCL;

    // Enable SOP/EPH markers
    if args.sop {
        enc_params.csty |= J2K_CCP_CSTY_SOP;
    }
    if args.eph {
        enc_params.csty |= J2K_CCP_CSTY_EPH;
    }

    // Enable precincts 256×256 … 128×128 (clamped to >= code-block, power-of-two)
    // Copy code-block sizes first to avoid borrow issues
    let cblk_w_i32 = enc_params.cblockw_init;
    let cblk_h_i32 = enc_params.cblockh_init;
    fill_precincts(&mut enc_params, levels, cblk_w_i32, cblk_h_i32);

    // Enable tile-parts with R split order (by resolution)
    enc_params.tp_on = 1;
    enc_params.tp_flag = b'R' as i8;

    // Enable reversible MCT for RGB if allowed
    if rgb && args.mct {
        enc_params.tcp_mct = 1;
    }

    // Enable precincts 256×256 … 128×128 (power-of-two, >= code-block)
    if args.precincts {
        let cblk_w_i32 = enc_params.cblockw_init;
        let cblk_h_i32 = enc_params.cblockh_init;
        fill_precincts(&mut enc_params, levels, cblk_w_i32, cblk_h_i32);
    }

    // Enable tile-parts with R split order (by resolution)
    if args.tp_r {
        enc_params.tp_on = 1;
        enc_params.tp_flag = b'R' as i8;
    }

    // Single quality layer, explicitly lossless
    enc_params.tcp_numlayers = 1;     // already set, keep it
    enc_params.tcp_rates[0] = 0.0;    // 0.0 = lossless in OpenJPEG
    enc_params.cp_disto_alloc = 1;    // use rate/distortion allocation (required when using rates)
    enc_params.cp_fixed_quality = 0;  // make sure we're not using fixed PSNR mode

    eprintln!("  [DEBUG] Creating OpenJPEG codec");
    let codec: *mut opj_codec_t = unsafe { opj_create_compress(CODEC_FORMAT::OPJ_CODEC_JP2) };
    if codec.is_null() { 
        unsafe { opj_image_destroy(img); }
        return Err(anyhow!("opj_create_compress failed")); 
    }

    let n_threads = if args.threads == 0 { 
        std::thread::available_parallelism().map(|n| n.get()).unwrap_or(1) 
    } else { 
        args.threads 
    } as i32;
    
    unsafe { opj_codec_set_threads(codec, n_threads) };

    eprintln!("  [DEBUG] Setting up encoder");
    let ok = unsafe { opj_setup_encoder(codec, &mut enc_params as *mut _, img) } != 0;
    if !ok {
        unsafe { 
            opj_destroy_codec(codec); 
            opj_image_destroy(img); 
        }
        return Err(anyhow!("opj_setup_encoder failed"));
    }

    eprintln!("  [DEBUG] Creating output stream");
    
    // Normalize path for OpenJPEG
    let output_str = output.to_string_lossy().replace('\\', "/");
    let c_out = CString::new(output_str.as_bytes())?;
    
    let stream: *mut opj_stream_t = unsafe { opj_stream_create_default_file_stream(c_out.as_ptr(), 0) };
    if stream.is_null() {
        unsafe { 
            opj_destroy_codec(codec); 
            opj_image_destroy(img); 
        }
        return Err(anyhow!("opj_stream_create_default_file_stream failed"));
    }

    eprintln!("  [DEBUG] Starting compression");
    let started = unsafe { opj_start_compress(codec, img, stream) } != 0;
    if !started {
        unsafe { 
            opj_stream_destroy(stream); 
            opj_destroy_codec(codec); 
            opj_image_destroy(img); 
        }
        return Err(anyhow!("opj_start_compress failed"));
    }

    eprintln!("  [DEBUG] Encoding");
    let encoded = unsafe { opj_encode(codec, stream) } != 0;
    
    eprintln!("  [DEBUG] Ending compression");
    let ended = unsafe { opj_end_compress(codec, stream) } != 0;

    if !encoded || !ended { 
        return Err(anyhow!("Compression failed (opj_encode/opj_end_compress)")); 
    }

    eprintln!("  [DEBUG] Compression completed successfully");

    // JP2 Resolution box (visible DPI for most viewers)
    // <<< CHANGED: use eff.dpi_box >>>
    if eff.dpi_box {
        if let (Some(xdpi), Some(ydpi)) = (meta.xdpi, meta.ydpi) {
            eprintln!("  [DEBUG] Adding resolution box");
            let _ = insert_resolution_box_jp2(output, xdpi, ydpi, meta.unit);
        }
    }

    // XMP DPI (optional fallback)
    // <<< CHANGED: use eff.xmp_dpi >>>
    if eff.xmp_dpi {
        if let (Some(xdpi), Some(ydpi)) = (meta.xdpi, meta.ydpi) {
            eprintln!("  [DEBUG] Adding XMP metadata");
            let xmp = build_xmp_with_dpi(xdpi, ydpi, meta.unit);
            let _ = append_jp2_xmp_box(output, xmp.as_bytes());
        }
    }

    eprintln!("  [DEBUG] Conversion completed successfully");
    
    // Krátká pauza pro stabilizaci
    std::thread::sleep(std::time::Duration::from_millis(50));
    
    Ok(())
}

// ---- De-interleave helpers ----------------------------------------------------

/// Fill planar components from interleaved U8 buffer.
/// Fast paths:
///   • Gray8: simple copy (auto-vectorized)
///   • RGB8: parallel rows via rayon; AVX2 gather path if enabled
fn fill_components_u8(
    img: *mut opj_image_t,
    inter: &[u8],
    w: u32,
    h: u32,
    ch: u32,
    use_avx2: bool,
) -> Result<()> {
    let plane = (w as usize) * (h as usize);

    if ch == 1 {
        // ---------------- Gray8 ----------------
        let ptr_i32 = unsafe { malloc(std::mem::size_of::<i32>() * plane) as *mut i32 };
        if ptr_i32.is_null() {
            return Err(anyhow!("alloc comp u8 gray"));
        }
        let dst = unsafe { std::slice::from_raw_parts_mut(ptr_i32, plane) };
        // simple copy (compiler will auto-vectorize)
        for i in 0..plane {
            dst[i] = inter[i] as i32;
        }
        unsafe { (*(*img).comps.add(0)).data = ptr_i32; }
        return Ok(());
    } else if ch == 3 {
        // ---------------- RGB8 -----------------
        let ptr_r = unsafe { malloc(std::mem::size_of::<i32>() * plane) as *mut i32 };
        let ptr_g = unsafe { malloc(std::mem::size_of::<i32>() * plane) as *mut i32 };
        let ptr_b = unsafe { malloc(std::mem::size_of::<i32>() * plane) as *mut i32 };
        if ptr_r.is_null() || ptr_g.is_null() || ptr_b.is_null() {
            return Err(anyhow!("alloc comp u8 rgb"));
        }

        let w_us   = w as usize;
        let stride = w_us * 3;

        // Create mutable slices over the malloc'd planes (safe to split by rows later)
        let dst_r_slice = unsafe { std::slice::from_raw_parts_mut(ptr_r, plane) };
        let dst_g_slice = unsafe { std::slice::from_raw_parts_mut(ptr_g, plane) };
        let dst_b_slice = unsafe { std::slice::from_raw_parts_mut(ptr_b, plane) };

        // Parallelize by rows; each thread gets disjoint &mut chunks.
        dst_r_slice
            .par_chunks_mut(w_us)
            .zip(dst_g_slice.par_chunks_mut(w_us))
            .zip(dst_b_slice.par_chunks_mut(w_us))
            .enumerate()
            .for_each(|(y, ((rrow, grow), brow))| {
                let src_row = &inter[y * stride .. (y + 1) * stride];

                // AVX2 path with 3-byte padding to prevent OOB gathers at row end
                #[cfg(target_arch = "x86_64")]
                {
                    if use_avx2 && std::is_x86_feature_detected!("avx2") && w_us >= 8 {
                        // Pad by 3 bytes (gather may read up to +2 beyond last pixel byte)
                        let mut row_pad = Vec::with_capacity(stride + 3);
                        row_pad.extend_from_slice(src_row);
                        row_pad.extend_from_slice(&[0u8; 3]);

                        unsafe {
                            deinterleave_rgb8_row_avx2(
                                row_pad.as_ptr(),
                                w_us,
                                rrow.as_mut_ptr(),
                                grow.as_mut_ptr(),
                                brow.as_mut_ptr(),
                            );
                        }
                        return; // done for this row
                    }
                }

                // Scalar fallback
                for x in 0..w_us {
                    let p = 3 * x;
                    rrow[x] = src_row[p] as i32;
                    grow[x] = src_row[p + 1] as i32;
                    brow[x] = src_row[p + 2] as i32;
                }
            });

        unsafe {
            (*(*img).comps.add(0)).data = ptr_r;
            (*(*img).comps.add(1)).data = ptr_g;
            (*(*img).comps.add(2)).data = ptr_b;
        }
        return Ok(());
    }

    // ------------- Generic N-channel fallback (rare here) -------------
    for c in 0..(ch as usize) {
        let ptr_i32 = unsafe { malloc(std::mem::size_of::<i32>() * plane) as *mut i32 };
        if ptr_i32.is_null() {
            return Err(anyhow!("alloc comp u8 generic"));
        }
        let dst = unsafe { std::slice::from_raw_parts_mut(ptr_i32, plane) };
        for y in 0..(h as usize) {
            for x in 0..(w as usize) {
                let idx = y * (w as usize) * (ch as usize) + x * (ch as usize) + c;
                dst[y * (w as usize) + x] = inter[idx] as i32;
            }
        }
        unsafe { (*(*img).comps.add(c)).data = ptr_i32; }
    }

    Ok(())
}

/// Fill planar components from interleaved U16 buffer.
/// Fast paths:
///  - Gray16: AVX2 widen to i32 if available
///  - RGB16: parallel rows via rayon (scalar widen)
fn fill_components_u16(
    img: *mut opj_image_t,
    inter: &[u16],
    w: u32,
    h: u32,
    ch: u32,
    use_avx2: bool,
) -> Result<()> {
    let plane = (w as usize) * (h as usize);
    unsafe {
        if ch == 1 {
            // Gray16
            let ptr_i32 = malloc(std::mem::size_of::<i32>() * plane) as *mut i32;
            if ptr_i32.is_null() { return Err(anyhow!("alloc comp u16 gray")); }
            let dst = std::slice::from_raw_parts_mut(ptr_i32, plane);
            if use_avx2 && cfg!(target_arch = "x86_64") {
                #[cfg(target_arch = "x86_64")]
                {
                    if widen_u16_to_i32_avx2(inter, dst) {
                        (*(*img).comps.add(0)).data = ptr_i32;
                        return Ok(());
                    }
                }
            }
            for i in 0..plane { dst[i] = inter[i] as i32; }
            (*(*img).comps.add(0)).data = ptr_i32;
            return Ok(());
        } else if ch == 3 {
            // RGB16: allocate three planes and fill in parallel per row
            let ptr_r = malloc(std::mem::size_of::<i32>() * plane) as *mut i32;
            let ptr_g = malloc(std::mem::size_of::<i32>() * plane) as *mut i32;
            let ptr_b = malloc(std::mem::size_of::<i32>() * plane) as *mut i32;
            if ptr_r.is_null() || ptr_g.is_null() || ptr_b.is_null() {
                return Err(anyhow!("alloc comp u16 rgb"));
            }

            let w_us = w as usize;
            let stride = w_us * 3;

            // Create mutable slices over the malloc'd planes.
            let dst_r_slice = std::slice::from_raw_parts_mut(ptr_r, plane);
            let dst_g_slice = std::slice::from_raw_parts_mut(ptr_g, plane);
            let dst_b_slice = std::slice::from_raw_parts_mut(ptr_b, plane);

            // Parallelize by rows; scalar widen (16->32) per pixel.
            dst_r_slice
                .par_chunks_mut(w_us)
                .zip(dst_g_slice.par_chunks_mut(w_us))
                .zip(dst_b_slice.par_chunks_mut(w_us))
                .enumerate()
                .for_each(|(y, ((rrow, grow), brow))| {
                    let src_row = &inter[y * stride .. (y + 1) * stride];
                    for x in 0..w_us {
                        let p = 3 * x;
                        rrow[x] = src_row[p] as i32;
                        grow[x] = src_row[p + 1] as i32;
                        brow[x] = src_row[p + 2] as i32;
                    }
                });

            (*(*img).comps.add(0)).data = ptr_r;
            (*(*img).comps.add(1)).data = ptr_g;
            (*(*img).comps.add(2)).data = ptr_b;
            return Ok(());
        }

        // Generic N-channel fallback
        for c in 0..(ch as usize) {
            let ptr_i32 = malloc(std::mem::size_of::<i32>() * plane) as *mut i32;
            if ptr_i32.is_null() { return Err(anyhow!("alloc comp u16 generic")); }
            for y in 0..(h as usize) {
                for x in 0..(w as usize) {
                    let idx = y * (w as usize) * (ch as usize) + x * (ch as usize) + c;
                    *ptr_i32.add(y * (w as usize) + x) = inter[idx] as i32;
                }
            }
            (*(*img).comps.add(c)).data = ptr_i32;
        }
    }
    Ok(())
}