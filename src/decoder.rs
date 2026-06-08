use std::ffi::CString;
use std::fs::File;
use std::path::Path;

use anyhow::{Context, Result, anyhow, bail};
use openjpeg_sys as opj;
use tiff::encoder::{TiffEncoder, colortype};
use tiff::tags::Tag;

use crate::info::{Jp2ComponentInfo, Jp2Info};

struct DecodedImage {
    width: u32,
    height: u32,
    color: DecodedColor,
    pixels: DecodedPixels,
    icc_profile: Option<Vec<u8>>,
}

#[derive(Clone, Copy)]
enum DecodedColor {
    Gray,
    Rgb,
}

enum DecodedPixels {
    U8(Vec<u8>),
    U16(Vec<u16>),
}

pub fn read_info(path: &Path, threads: i32) -> Result<Jp2Info> {
    let decoder = Decoder::open(path, threads, 0)?;
    unsafe { info_from_image(decoder.image) }
}

pub fn decode_to_tiff(input: &Path, output: &Path, threads: i32) -> Result<()> {
    let image = decode_full(input, threads)?;
    write_tiff(output, &image)
}

fn decode_full(path: &Path, threads: i32) -> Result<DecodedImage> {
    let decoder = Decoder::open(path, threads, 0)?;

    let decoded = unsafe { opj::opj_decode(decoder.codec, decoder.stream, decoder.image) } != 0;
    let ended = unsafe { opj::opj_end_decompress(decoder.codec, decoder.stream) } != 0;
    if !decoded || !ended {
        bail!("OpenJPEG FFI decompression failed");
    }

    unsafe { image_to_pixels(decoder.image) }
}

struct Decoder {
    codec: *mut opj::opj_codec_t,
    stream: *mut opj::opj_stream_t,
    image: *mut opj::opj_image_t,
}

impl Decoder {
    fn open(path: &Path, threads: i32, reduce: u32) -> Result<Self> {
        let mut params = unsafe {
            let mut p = std::mem::MaybeUninit::<opj::opj_dparameters_t>::zeroed();
            opj::opj_set_default_decoder_parameters(p.as_mut_ptr());
            p.assume_init()
        };
        params.cp_reduce = reduce;
        params.decod_format = if is_raw_codestream(path) { 0 } else { 1 };

        let codec = unsafe {
            opj::opj_create_decompress(if is_raw_codestream(path) {
                opj::CODEC_FORMAT::OPJ_CODEC_J2K
            } else {
                opj::CODEC_FORMAT::OPJ_CODEC_JP2
            })
        };
        if codec.is_null() {
            bail!("opj_create_decompress failed");
        }

        let setup_ok = unsafe { opj::opj_setup_decoder(codec, &mut params) } != 0;
        if !setup_ok {
            unsafe { opj::opj_destroy_codec(codec) };
            bail!("opj_setup_decoder failed");
        }

        if threads > 1 {
            let ok = unsafe { opj::opj_codec_set_threads(codec, threads) } != 0;
            if !ok {
                unsafe { opj::opj_destroy_codec(codec) };
                bail!("opj_codec_set_threads failed");
            }
        }

        let c_path = path_to_cstring(path)?;
        let stream = unsafe {
            opj::opj_stream_create_default_file_stream(c_path.as_ptr(), opj::OPJ_TRUE as i32)
        };
        if stream.is_null() {
            unsafe { opj::opj_destroy_codec(codec) };
            bail!("opj_stream_create_default_file_stream failed");
        }

        let mut image: *mut opj::opj_image_t = std::ptr::null_mut();
        let ok = unsafe { opj::opj_read_header(stream, codec, &mut image) } != 0;
        if !ok || image.is_null() {
            unsafe {
                opj::opj_stream_destroy(stream);
                opj::opj_destroy_codec(codec);
            }
            bail!("opj_read_header failed");
        }

        Ok(Self {
            codec,
            stream,
            image,
        })
    }
}

impl Drop for Decoder {
    fn drop(&mut self) {
        unsafe {
            if !self.image.is_null() {
                opj::opj_image_destroy(self.image);
            }
            if !self.stream.is_null() {
                opj::opj_stream_destroy(self.stream);
            }
            if !self.codec.is_null() {
                opj::opj_destroy_codec(self.codec);
            }
        }
    }
}

unsafe fn info_from_image(image: *mut opj::opj_image_t) -> Result<Jp2Info> {
    if image.is_null() {
        bail!("OpenJPEG returned a null image");
    }
    let image_ref = unsafe { &*image };
    if image_ref.numcomps > 0 && image_ref.comps.is_null() {
        bail!("OpenJPEG returned image components without data");
    }
    let comps = unsafe { std::slice::from_raw_parts(image_ref.comps, image_ref.numcomps as usize) };
    let components = comps
        .iter()
        .map(|component| Jp2ComponentInfo {
            width: component.w,
            height: component.h,
            dx: component.dx,
            dy: component.dy,
            precision: component.prec,
            signed: component.sgnd != 0,
        })
        .collect();

    Ok(Jp2Info {
        width: image_ref.x1.saturating_sub(image_ref.x0),
        height: image_ref.y1.saturating_sub(image_ref.y0),
        components,
        icc_profile_len: image_ref.icc_profile_len,
    })
}

unsafe fn image_to_pixels(image: *mut opj::opj_image_t) -> Result<DecodedImage> {
    if image.is_null() {
        bail!("OpenJPEG returned a null image");
    }
    let image_ref = unsafe { &*image };
    if image_ref.numcomps == 0 || image_ref.comps.is_null() {
        bail!("OpenJPEG returned an image without components");
    }

    let comps = unsafe { std::slice::from_raw_parts(image_ref.comps, image_ref.numcomps as usize) };
    let width = comps[0].w;
    let height = comps[0].h;
    if width == 0 || height == 0 {
        bail!("OpenJPEG returned empty image components");
    }

    let component_count = comps.len();
    let color = if component_count >= 3 {
        DecodedColor::Rgb
    } else {
        DecodedColor::Gray
    };
    let precision = comps
        .iter()
        .take(if matches!(color, DecodedColor::Rgb) {
            3
        } else {
            1
        })
        .map(|component| component.prec)
        .max()
        .unwrap_or(8);

    let icc_profile = if !image_ref.icc_profile_buf.is_null() && image_ref.icc_profile_len > 0 {
        let bytes = unsafe {
            std::slice::from_raw_parts(
                image_ref.icc_profile_buf,
                image_ref.icc_profile_len as usize,
            )
        };
        Some(bytes.to_vec())
    } else {
        None
    };

    let pixels = if precision <= 8 {
        DecodedPixels::U8(image_to_interleaved_u8(comps, width, height, color)?)
    } else {
        DecodedPixels::U16(image_to_interleaved_u16(comps, width, height, color)?)
    };

    Ok(DecodedImage {
        width,
        height,
        color,
        pixels,
        icc_profile,
    })
}

fn image_to_interleaved_u8(
    comps: &[opj::opj_image_comp_t],
    width: u32,
    height: u32,
    color: DecodedColor,
) -> Result<Vec<u8>> {
    let channels = if matches!(color, DecodedColor::Rgb) {
        3
    } else {
        1
    };
    let mut out = vec![0u8; width as usize * height as usize * channels];
    for y in 0..height as usize {
        for x in 0..width as usize {
            let dst = (y * width as usize + x) * channels;
            if matches!(color, DecodedColor::Rgb) {
                out[dst] = component_sample_to_u8(&comps[0], x, y, width, height)?;
                out[dst + 1] = component_sample_to_u8(&comps[1], x, y, width, height)?;
                out[dst + 2] = component_sample_to_u8(&comps[2], x, y, width, height)?;
            } else {
                out[dst] = component_sample_to_u8(&comps[0], x, y, width, height)?;
            }
        }
    }
    Ok(out)
}

fn image_to_interleaved_u16(
    comps: &[opj::opj_image_comp_t],
    width: u32,
    height: u32,
    color: DecodedColor,
) -> Result<Vec<u16>> {
    let channels = if matches!(color, DecodedColor::Rgb) {
        3
    } else {
        1
    };
    let mut out = vec![0u16; width as usize * height as usize * channels];
    for y in 0..height as usize {
        for x in 0..width as usize {
            let dst = (y * width as usize + x) * channels;
            if matches!(color, DecodedColor::Rgb) {
                out[dst] = component_sample_to_u16(&comps[0], x, y, width, height)?;
                out[dst + 1] = component_sample_to_u16(&comps[1], x, y, width, height)?;
                out[dst + 2] = component_sample_to_u16(&comps[2], x, y, width, height)?;
            } else {
                out[dst] = component_sample_to_u16(&comps[0], x, y, width, height)?;
            }
        }
    }
    Ok(out)
}

fn component_sample_to_u8(
    component: &opj::opj_image_comp_t,
    x: usize,
    y: usize,
    out_width: u32,
    out_height: u32,
) -> Result<u8> {
    let sample = component_sample(component, x, y, out_width, out_height)?;
    let precision = component.prec.clamp(1, 31);
    let value = normalize_sample(sample, precision, component.sgnd != 0);
    let max_value = ((1u64 << precision) - 1).max(1);
    Ok(((value * 255 + max_value / 2) / max_value) as u8)
}

fn component_sample_to_u16(
    component: &opj::opj_image_comp_t,
    x: usize,
    y: usize,
    out_width: u32,
    out_height: u32,
) -> Result<u16> {
    let sample = component_sample(component, x, y, out_width, out_height)?;
    let precision = component.prec.clamp(1, 31);
    let value = normalize_sample(sample, precision, component.sgnd != 0);
    if precision == 16 {
        Ok(value.min(u16::MAX as u64) as u16)
    } else {
        let max_value = ((1u64 << precision) - 1).max(1);
        Ok(((value * u16::MAX as u64 + max_value / 2) / max_value) as u16)
    }
}

fn component_sample(
    component: &opj::opj_image_comp_t,
    x: usize,
    y: usize,
    out_width: u32,
    out_height: u32,
) -> Result<i32> {
    if component.data.is_null() || component.w == 0 || component.h == 0 {
        bail!("OpenJPEG returned a component without sample data");
    }
    let cx = (x as u64 * component.w as u64 / out_width as u64)
        .min(component.w.saturating_sub(1) as u64) as usize;
    let cy = (y as u64 * component.h as u64 / out_height as u64)
        .min(component.h.saturating_sub(1) as u64) as usize;
    let index = cy
        .checked_mul(component.w as usize)
        .and_then(|base| base.checked_add(cx))
        .ok_or_else(|| anyhow!("OpenJPEG component index overflow"))?;
    Ok(unsafe { *component.data.add(index) })
}

fn normalize_sample(sample: i32, precision: u32, signed: bool) -> u64 {
    let value = if signed {
        sample.saturating_add(1i32.checked_shl(precision.saturating_sub(1)).unwrap_or(0))
    } else {
        sample
    };
    let max_value = ((1u64 << precision) - 1).max(1);
    value.clamp(0, max_value as i32) as u64
}

fn write_tiff(path: &Path, image: &DecodedImage) -> Result<()> {
    let file = File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let mut encoder = TiffEncoder::new(file)?;
    match (&image.color, &image.pixels) {
        (DecodedColor::Gray, DecodedPixels::U8(pixels)) => {
            let mut tiff = encoder.new_image::<colortype::Gray8>(image.width, image.height)?;
            write_icc_tag(&mut tiff, image.icc_profile.as_deref())?;
            tiff.write_data(pixels)?;
        }
        (DecodedColor::Gray, DecodedPixels::U16(pixels)) => {
            let mut tiff = encoder.new_image::<colortype::Gray16>(image.width, image.height)?;
            write_icc_tag(&mut tiff, image.icc_profile.as_deref())?;
            tiff.write_data(pixels)?;
        }
        (DecodedColor::Rgb, DecodedPixels::U8(pixels)) => {
            let mut tiff = encoder.new_image::<colortype::RGB8>(image.width, image.height)?;
            write_icc_tag(&mut tiff, image.icc_profile.as_deref())?;
            tiff.write_data(pixels)?;
        }
        (DecodedColor::Rgb, DecodedPixels::U16(pixels)) => {
            let mut tiff = encoder.new_image::<colortype::RGB16>(image.width, image.height)?;
            write_icc_tag(&mut tiff, image.icc_profile.as_deref())?;
            tiff.write_data(pixels)?;
        }
    }
    Ok(())
}

fn write_icc_tag<
    'a,
    W: std::io::Write + std::io::Seek,
    C: tiff::encoder::colortype::ColorType,
    K: tiff::encoder::TiffKind,
>(
    image: &mut tiff::encoder::ImageEncoder<'a, W, C, K>,
    icc_profile: Option<&[u8]>,
) -> Result<()> {
    if let Some(icc_profile) = icc_profile.filter(|icc| !icc.is_empty()) {
        image
            .encoder()
            .write_tag(Tag::Unknown(34675), icc_profile)
            .context("writing TIFF ICC profile tag")?;
    }
    Ok(())
}

fn path_to_cstring(path: &Path) -> Result<CString> {
    CString::new(path.to_string_lossy().as_bytes())
        .with_context(|| format!("path contains NUL byte: {}", path.display()))
}

fn is_raw_codestream(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| matches!(ext.to_ascii_lowercase().as_str(), "j2k" | "j2c" | "jpc"))
        .unwrap_or(false)
}
