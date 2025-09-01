# tif2jp2

TIFF to JPEG2000 (JP2) lossless converter built in Rust with a thin FFI layer over OpenJPEG.

✅ Lossless 5/3 wavelet (reversible)

✅ Preserves/reserializes DPI into JP2 res box (and optional XMP fallback)

✅ ICC profile attachment (from TIFF or a user-supplied profile)

✅ Tiled + code-block encoding

✅ Multi-threaded via OpenJPEG

✅ AVX2 fast path upload

✅ Single file or batch/recursive mode

✅ Friendly CLI with --help, --version, --verbose

Goal: a practical, fast, no-nonsense archival path from TIFF to JP2 while staying compatible with common JP2 readers.

## Quick Start

**Convert a single TIFF to JP2 (same folder/name)**

```
tif2jp2 input.tif
```

**Convert into a specific output file**

```
tif2jp2 input.tif -o output.jp2
```

**Convert all TIFFs in a folder (non-recursive)**

```
tif2jp2 ./scans -o ./out
```

**Convert recursively with verbose progress**

```
tif2jp2 ./archive --recursive -o ./out -v
```

## Features

- **Lossless JP2:** uses OpenJPEG’s reversible 5/3 transform (irreversible = 0).

- **DPI preservation:** writes resolution to jp2h/resc+resd (and optionally XMP).

- **ICC attachment:** embeds ICC from TIFF if available, or from --icc file.

- **Tiling & code-blocks:** --tile WxH and --block WxH allow large images to encode efficiently.

- **Multi-threaded:** --threads sets OpenJPEG worker threads (0 = auto).

- **AVX2 upload:** accelerated buffer upload to OpenJPEG image planes.

## Installation

### Prerequisites

- **Rust** (stable) and **Cargo**

- **OpenJPEG** development/runtime libraries

  - Linux: `libopenjp2/libopenjpeg` (headers + shared library)

  - Windows: `openjp2.dll` available at runtime

- The crate depends on:

`openjpeg-sys, tiff, walkdir, which, clap, anyhow, libc, rayon, thiserror`

The program is an FFI wrapper around OpenJPEG for encoding, but it handles TIFF parsing, metadata mapping (DPI/ICC/XMP), and planar upload itself.

### Build on Linux

Install OpenJPEG dev packages (example on Debian/Ubuntu):
```
sudo apt-get update
sudo apt-get install -y libopenjp2-7 libopenjp2-7-dev
```
Build:

```
git clone https://github.com/bezverec/tif2jp2.git
cd tif2jp2
cargo build --release
# binary at target/release/tif2jp2
```

### Build on Windows

1. Install a recent **Rust** toolchain (MSVC).

2. Install **OpenJPEG** and make sure `openjp2.dll` is reachable at runtime:

  - Put `openjp2.dll` next to tif2jp2.exe, or

  - Add the folder containing `openjp2.dll` to your user/system `PATH`.

Build:
```
git clone https://github.com/your-org/tif2jp2.git
cd tif2jp2
cargo build --release
# binary at target\release\tif2jp2.exe
```
If you see `“The code execution cannot proceed because openjp2.dll was not found”`, place the **DLL** next to the **EXE** or add it to `PATH`.

## Usage

Run `tif2jp2 --help` anytime:
```
TIFF to JPEG2000 (JP2) lossless via OpenJPEG FFI

Usage: tif2jp2 [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file or directory (use --recursive for subdirectories)

Options:
  -o, --output <OUTPUT>   Output file or directory (mirrors input structure if directory)
      --recursive         Recursively traverse the input directory
      --tile <WxH>        Tile size, e.g. 1024x1024 [default: 1024x1024]
      --block <WxH>       Code-block size, e.g. 64x64 [default: 64x64]
      --levels <NUM|auto> Number of resolutions [default: auto]
      --force             Overwrite existing output files
      --threads <N>       OpenJPEG threads (0 = auto = all cores) [default: 0]
      --icc <PATH>        Path to ICC profile (overrides ICC detected in TIFF)
      --dpi-box           Write DPI into JP2 'res' box [default: on]
      --xmp-dpi           Write DPI into XMP 'uuid' box [default: on]
      --avx2              Enable AVX2 fast path if supported [default: on]
  -v, --verbose...        Increase verbosity (-v, -vv)
      --version           Print version
      --help              Print help
```

### Basic examples

```
# Lossless convert with explicit tile/code-block and 6 resolution levels
tif2jp2 scan.tif -o scan.jp2 --tile 2048x2048 --block 64x64 --levels 6

# Recursively convert a directory, mirror structure to ./out, show timings
tif2jp2 ./input --recursive -o ./out -v

# Override ICC (e.g., embed sRGB or Gray profile explicitly)
tif2jp2 image.tif -o image.jp2 --icc ./profiles/sRGB.icc

# Use all cores automatically, disable XMP fallback
tif2jp2 img.tif -o img.jp2 --threads 0 --xmp-dpi=false
```

### Metadata handling
**DPI in JP2 (res box)**

If TIFF carries `XResolution`, `YResolution`, and `ResolutionUnit` (inch/centimeter), the tool converts DPI → pixels per metre and writes a `res` superbox under `jp2h` with both `resc` and `resd`. Many viewers read this and report correct DPI.

- Toggle via `--dpi-box` (default on).

**XMP DPI fallback**

Some consumers prefer XMP metadata. When enabled, the tool appends an **XMP UUID box** at the end of the JP2 carrying TIFF-style `tiff:XResolution`, `tiff:YResolution`, `tiff:ResolutionUnit`.

- Toggle via `--xmp-dpi` (default on).

**ICC profiles**

If TIFF has an ICC tag (34675), the tool **best-effort** attaches it to the JP2 image.

- You can override with `--icc <path.icc>`.

Note: with `tiff` crate v0.9.x, ICC may sometimes be read as a single Byte/Ascii value (not a full profile). For archival workflows, passing a known profile via `--icc` is recommended.

### Performance
- **Multi-threading:** OpenJPEG parallelism is controlled by `--threads` (0 = auto = all cores).

- **Tiling:** Larger tiles (e.g. `2048x2048`) usually improve throughput on big images.

- **Code-blocks:** Defaults `64x64` are good general-purpose values.

- **Resolutions:** `--levels auto` chooses a sane value based on image size (clamped 3..8).

- **AVX2 path:** On x86-64 with AVX2 upload is accelerated (`--avx2`, on by default).

### Troubleshooting
**“No input TIFFs found”**

Check your path or add `--recursive` for directories.

**“File is not a TIFF”**

Only .tif/.tiff extensions are supported.

**“The code execution cannot proceed because openjp2.dll was not found” (Windows)**

Place `openjp2.dll` in the same folder as tif2jp2.exe or add its folder to `PATH`.

**Produced JP2 shows 72 or no DPI**

Some viewers ignore JP2 res boxes. Enable `--xmp-dpi` (default on) to add XMP fallback, or verify DPI fields in your viewer.

**TIFF with alpha / CMYK**

Alpha and CMYK aren’t supported. Flatten alpha or convert to RGB/Gray before encoding.
