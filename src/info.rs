use std::path::Path;

use anyhow::{Context, Result};

#[derive(Debug, Clone)]
pub struct Jp2Info {
    pub width: u32,
    pub height: u32,
    pub components: Vec<Jp2ComponentInfo>,
    pub icc_profile_len: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct Jp2ComponentInfo {
    pub width: u32,
    pub height: u32,
    pub dx: u32,
    pub dy: u32,
    pub precision: u32,
    pub signed: bool,
}

pub fn print_jp2_info(path: &Path, info: &Jp2Info) {
    println!("{}", path.display());
    println!("  size: {}x{}", info.width, info.height);
    println!("  components: {}", info.components.len());
    println!("  icc_profile_len: {}", info.icc_profile_len);
    for (idx, component) in info.components.iter().enumerate() {
        println!(
            "  component {}: {}x{}, dx={}, dy={}, precision={}, signed={}",
            idx,
            component.width,
            component.height,
            component.dx,
            component.dy,
            component.precision,
            component.signed
        );
    }
}

pub fn openjpeg_threads(threads: usize) -> Result<i32> {
    let threads = if threads == 0 {
        std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1)
    } else {
        threads
    };
    i32::try_from(threads).context("OpenJPEG thread count is too large")
}

pub fn is_tiff(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase()),
        Some(ref extension) if extension == "tif" || extension == "tiff"
    )
}

pub fn is_jpeg2000(path: &Path) -> bool {
    matches!(
        path.extension()
            .and_then(|extension| extension.to_str())
            .map(|extension| extension.to_ascii_lowercase()),
        Some(ref extension) if matches!(extension.as_str(), "jp2" | "j2k" | "j2c" | "jpc")
    )
}
