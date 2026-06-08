mod decoder;
mod encoder;
mod info;

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::Instant,
};

use anyhow::{Context, Result, anyhow};
use clap::{ArgAction, Parser};
use encoder::{Effective, EncodeOptions};
use walkdir::WalkDir;

/// Tiny logger with verbosity levels (0 = errors only, 1 = info)
struct Log {
    lvl: u8,
}

impl Log {
    fn new(lvl: u8) -> Self {
        Self { lvl }
    }

    #[inline]
    fn v1(&self, msg: impl AsRef<str>) {
        if self.lvl >= 1 {
            eprintln!("{}", msg.as_ref());
        }
    }
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

    /// Decode JPEG2000 input to TIFF instead of encoding TIFF to JP2
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "info")]
    pub decode: bool,

    /// Print JPEG2000 header information and exit
    #[arg(long, action = ArgAction::SetTrue, conflicts_with = "decode")]
    pub info: bool,

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

    /// Progression order (LRCP|RLCP|RPCL|PCRL|CPRL)
    #[arg(long, default_value = "RPCL", value_name = "ORDER")]
    pub order: String,

    /// Archival master NDK preset (alias: --archival). Forces RPCL, 4096x4096 tiles, 64x64 blocks,
    /// levels=6, SOP/EPH on, precincts on (256..128), tile-parts R, reversible MCT on, TLM on.
    #[arg(long = "archival-master-ndk", alias = "archival", action = ArgAction::SetTrue)]
    pub archival_master_ndk: bool,

    /// Write DPI into JP2 'res' box [default: on]
    #[arg(long = "dpi-box", action = ArgAction::SetTrue, overrides_with = "dpi_box_off")]
    pub dpi_box_on: bool,
    /// Disable Write DPI into JP2 'res' box
    #[arg(long = "no-dpi-box", action = ArgAction::SetTrue, overrides_with = "dpi_box_on")]
    pub dpi_box_off: bool,

    /// Write DPI into XMP 'uuid' box [default: off]
    #[arg(long = "xmp-dpi", action = ArgAction::SetTrue, overrides_with = "xmp_dpi_off")]
    pub xmp_dpi_on: bool,
    /// Disable Write DPI into XMP 'uuid' box
    #[arg(long = "no-xmp-dpi", action = ArgAction::SetTrue, overrides_with = "xmp_dpi_on")]
    pub xmp_dpi_off: bool,

    /// Enable AVX2 fast path if supported [default: off]
    #[arg(long = "avx2", action = ArgAction::SetTrue, overrides_with = "avx2_off")]
    pub avx2_on: bool,
    /// Force no AVX2
    #[arg(long = "no-avx2", action = ArgAction::SetTrue, overrides_with = "avx2_on")]
    pub avx2_off: bool,

    /// Enable tile-parts split by Resolution (R) [default: on]
    #[arg(long = "tp-r", action = ArgAction::SetTrue, overrides_with = "tp_r_off")]
    pub tp_r_on: bool,
    /// Disable tile-parts split by Resolution (R)
    #[arg(long = "no-tp-r", action = ArgAction::SetTrue, overrides_with = "tp_r_on")]
    pub tp_r_off: bool,

    /// Enable precinct partitioning (256x256 ... 128x128) [default: on]
    #[arg(long = "precincts", action = ArgAction::SetTrue, overrides_with = "precincts_off")]
    pub precincts_on: bool,
    /// Disable precinct partitioning
    #[arg(long = "no-precincts", action = ArgAction::SetTrue, overrides_with = "precincts_on")]
    pub precincts_off: bool,

    /// Enable SOP markers (Start of Packet) [default: on]
    #[arg(long = "sop", action = ArgAction::SetTrue, overrides_with = "sop_off")]
    pub sop_on: bool,
    /// Disable SOP markers
    #[arg(long = "no-sop", action = ArgAction::SetTrue, overrides_with = "sop_on")]
    pub sop_off: bool,

    /// Enable EPH markers (End of Packet Header) [default: on]
    #[arg(long = "eph", action = ArgAction::SetTrue, overrides_with = "eph_off")]
    pub eph_on: bool,
    /// Disable EPH markers
    #[arg(long = "no-eph", action = ArgAction::SetTrue, overrides_with = "eph_on")]
    pub eph_off: bool,

    /// Enable reversible MCT for RGB [default: on]
    #[arg(long = "mct", action = ArgAction::SetTrue, overrides_with = "mct_off")]
    pub mct_on: bool,
    /// Disable reversible MCT for RGB
    #[arg(long = "no-mct", action = ArgAction::SetTrue, overrides_with = "mct_on")]
    pub mct_off: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short = 'v', action = ArgAction::Count)]
    /// keep long form working but hidden (prevents '--verbose...' in help)
    #[arg(long = "verbose", hide = true)]
    pub verbose: u8,

    /// Enable TLM markers (Tile-part Length) [NDK preset: on]
    #[arg(long = "tlm", action = ArgAction::SetTrue, overrides_with = "tlm_off")]
    pub tlm_on: bool,
    /// Disable TLM markers
    #[arg(long = "no-tlm", action = ArgAction::SetTrue, overrides_with = "tlm_on")]
    pub tlm_off: bool,

    /// Enable PLT markers (Packet Length in TPH) [default: off]
    #[arg(long = "plt", action = ArgAction::SetTrue, overrides_with = "plt_off")]
    pub plt_on: bool,
    /// Disable PLT markers
    #[arg(long = "no-plt", action = ArgAction::SetTrue, overrides_with = "plt_on")]
    pub plt_off: bool,

    /// Enable Selective arithmetic coding bypass (code-block LAZY) [NDK preset: on]
    #[arg(long = "bypass", action = ArgAction::SetTrue, overrides_with = "bypass_off")]
    pub bypass_on: bool,
    /// Disable Selective arithmetic coding bypass
    #[arg(long = "no-bypass", action = ArgAction::SetTrue, overrides_with = "bypass_on")]
    pub bypass_off: bool,
}

impl Args {
    fn operation(&self) -> Operation {
        if self.info {
            Operation::Info
        } else if self.decode {
            Operation::Decode
        } else {
            Operation::Encode
        }
    }

    fn encode_options(&self) -> EncodeOptions {
        EncodeOptions {
            tile: self.tile.clone(),
            block: self.block.clone(),
            levels: self.levels.clone(),
            threads: self.threads,
            icc: self.icc.clone(),
            order: self.order.clone(),
            toggles: self.effective(),
        }
    }

    fn effective(&self) -> Effective {
        #[inline]
        fn resolve(on: bool, off: bool, default_: bool) -> bool {
            if on {
                true
            } else if off {
                false
            } else {
                default_
            }
        }

        Effective {
            avx2: resolve(self.avx2_on, self.avx2_off, false),
            dpi_box: resolve(self.dpi_box_on, self.dpi_box_off, true),
            xmp_dpi: resolve(self.xmp_dpi_on, self.xmp_dpi_off, false),
            tp_r: resolve(self.tp_r_on, self.tp_r_off, true),
            precincts: resolve(self.precincts_on, self.precincts_off, true),
            sop: resolve(self.sop_on, self.sop_off, true),
            eph: resolve(self.eph_on, self.eph_off, true),
            mct: resolve(self.mct_on, self.mct_off, true),
            tlm: resolve(self.tlm_on, self.tlm_off, true),
            plt: resolve(self.plt_on, self.plt_off, false),
            bypass: resolve(self.bypass_on, self.bypass_off, true),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Operation {
    Encode,
    Decode,
    Info,
}

fn apply_archival_master_ndk_defaults(args: &mut Args) {
    args.tile = "4096x4096".into();
    args.block = "64x64".into();
    args.levels = "6".into();
    args.order = "RPCL".into();

    args.dpi_box_on = true;
    args.dpi_box_off = false;
    args.xmp_dpi_on = false;
    args.xmp_dpi_off = false;
    args.avx2_off = false;

    args.tp_r_on = true;
    args.tp_r_off = false;
    args.precincts_on = true;
    args.precincts_off = false;
    args.sop_on = true;
    args.sop_off = false;
    args.eph_on = true;
    args.eph_off = false;
    args.mct_on = true;
    args.mct_off = false;
    args.tlm_on = true;
    args.tlm_off = false;
    args.plt_on = false;
    args.plt_off = true;
    args.bypass_on = true;
    args.bypass_off = false;
}

fn main() -> Result<()> {
    let mut args = Args::parse();
    let operation = args.operation();

    if operation == Operation::Encode && args.archival_master_ndk {
        apply_archival_master_ndk_defaults(&mut args);
        eprintln!(
            "[preset] Using Archival Master NDK defaults (RPCL, 4096x4096, 64x64, levels=6, SOP/EPH/precincts/tp-r/MCT/bypass on)"
        );
    }

    let log = Log::new(args.verbose);
    log.v1(format!("Input:  {}", args.input.display()));
    if let Some(out) = &args.output {
        log.v1(format!("Output: {}", out.display()));
    }

    if let Some(out_dir) = &args.output {
        if out_dir.is_dir() || (!out_dir.exists() && args.input.is_dir()) {
            fs::create_dir_all(out_dir).context("Creating output directory")?;
        }
    }

    let inputs = collect_inputs(&args.input, args.recursive, operation)?;
    log.v1(format!("Found {} input file(s).", inputs.len()));
    if inputs.is_empty() {
        return Err(anyhow!("No input files found"));
    }

    let encode_options = args.encode_options();
    for (idx, input) in inputs.iter().enumerate() {
        if operation == Operation::Info {
            let info = decoder::read_info(input, info::openjpeg_threads(args.threads)?)?;
            info::print_jp2_info(input, &info);
            continue;
        }

        let out = derive_output_path(&args, input, operation)?;
        if let Some(parent) = out.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).context("Creating output subdirectory")?;
            }
        }

        if out.exists() && !args.force {
            eprintln!("Skipping (exists): {}", out.display());
            continue;
        }

        eprintln!(
            "({}/{}) -> Processing: {} -> {}",
            idx + 1,
            inputs.len(),
            input.display(),
            out.display()
        );
        let t0 = Instant::now();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match operation {
            Operation::Encode => encoder::encode_tiff_to_jp2(input, &out, &encode_options),
            Operation::Decode => {
                decoder::decode_to_tiff(input, &out, info::openjpeg_threads(args.threads)?)
            }
            Operation::Info => unreachable!("info mode is handled before output derivation"),
        }));

        match result {
            Ok(Ok(())) => {
                let dt = t0.elapsed();
                eprintln!("OK {} -> {} ({:.2?})", input.display(), out.display(), dt);
            }
            Ok(Err(e)) => {
                eprintln!("ERR {} - Error: {}", input.display(), e);
            }
            Err(panic) => {
                if let Some(s) = panic.downcast_ref::<&str>() {
                    eprintln!("ERR {} - Panic during conversion: {}", input.display(), s);
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    eprintln!("ERR {} - Panic during conversion: {}", input.display(), s);
                } else {
                    eprintln!(
                        "ERR {} - Panic during conversion: <unknown reason>",
                        input.display()
                    );
                }
            }
        }

        let _ = std::io::stderr().flush();
        let _ = std::io::stdout().flush();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    eprintln!("All files processed successfully!");
    Ok(())
}

fn collect_inputs(root: &Path, recursive: bool, operation: Operation) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    if root.is_file() {
        if accepts_input(root, operation) {
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
            if path.is_file() && accepts_input(path, operation) {
                eprintln!("Found input: {}", path.display());
                files.push(path.to_path_buf());
            }
        }
    }

    eprintln!("Total files found: {}", files.len());
    files.sort();
    Ok(files)
}

fn accepts_input(path: &Path, operation: Operation) -> bool {
    match operation {
        Operation::Encode => info::is_tiff(path),
        Operation::Decode | Operation::Info => info::is_jpeg2000(path),
    }
}

fn derive_output_path(args: &Args, input: &Path, operation: Operation) -> Result<PathBuf> {
    let extension = match operation {
        Operation::Encode => "jp2",
        Operation::Decode => "tif",
        Operation::Info => unreachable!("info mode has no output path"),
    };
    let result = match &args.output {
        Some(out) => {
            if out.is_dir() || (!out.exists() && args.input.is_dir()) {
                let file_name = input
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("output");
                let mut output_path = out.clone();
                output_path.push(format!("{}.{}", file_name, extension));
                output_path
            } else {
                out.clone()
            }
        }
        None => input.with_extension(extension),
    };

    let normalized_path = PathBuf::from(result.to_string_lossy().replace('\\', "/"));
    eprintln!(
        "Input: {} -> Output: {}",
        input.display(),
        normalized_path.display()
    );
    Ok(normalized_path)
}
