# tif2jp2

TIFF to JPEG2000 (JP2) lossless converter built in Rust with a thin FFI layer over OpenJPEG.

✅ Lossless 5/3 wavelet (reversible)  
✅ Preserves/reserializes DPI into JP2 res box (and optional XMP fallback)  
✅ ICC profile attachment (from TIFF or a user-supplied profile)  
✅ Tiled + code-block encoding  
✅ Multi-threaded via OpenJPEG  
✅ AVX2 fast path upload  
✅ Single file or batch/recursive mode  
✅ Friendly CLI with `--help`, `--version`, `--verbose`  

Goal: a practical, fast, no-nonsense archival path from TIFF to JP2 while staying compatible with common JP2 readers.

---

## Quick Start

**Convert a single TIFF to JP2 (same folder/name)**

```bash
tif2jp2 input.tif
```

**Convert into a specific output file**

```bash
tif2jp2 input.tif -o output.jp2
```

**Convert all TIFFs in a folder (non-recursive)**

```bash
tif2jp2 ./scans -o ./out
```

**Convert recursively with verbose progress**

```bash
tif2jp2 ./archive --recursive -o ./out -v
```

**Force overwrite existing files**

```bash
tif2jp2 ./scans -o ./out --force
```

---

## Features

- **Lossless JP2**: OpenJPEG reversible 5/3 transform (irreversible = 0)  
- **DPI preservation**: Writes resolution to JP2 `res` box (optionally XMP)  
- **ICC attachment**: Embeds ICC from TIFF or `--icc` file  
- **Tiling & code-blocks**: `--tile WxH`, `--block WxH` for efficient encoding  
- **Multi-threaded**: `--threads` sets worker threads (0 = auto)  
- **AVX2 upload**: Accelerated buffer upload (`--avx2`)  
- **Smart file handling**: Skips existing files unless `--force`  
- **Robust error handling**: Continues even if some files fail  

---

## Installation

### Prerequisites
- Rust (stable) and Cargo  
- OpenJPEG development/runtime libraries  

**Linux:** `libopenjp2` (`libopenjp2-7`, `libopenjp2-7-dev`)  
**Windows:** `openjp2.dll` must be available at runtime  

---

### Build from Source

**Linux (Debian/Ubuntu):**

```bash
sudo apt-get update
sudo apt-get install -y libopenjp2-7 libopenjp2-7-dev
git clone https://github.com/bezverec/tif2jp2.git
cd tif2jp2
RUSTFLAGS="-C target-cpu=native" cargo build --release
# binary at target/release/tif2jp2
```

**Windows (PowerShell):**

```powershell
git clone https://github.com/bezverec/tif2jp2.git
cd tif2jp2
$env:RUSTFLAGS="-C target-cpu=native"; cargo build --release
# binary at target\release\tif2jp2.exe
```

⚠️ **Windows note:** If you see  
`The code execution cannot proceed because openjp2.dll was not found`,  
place `openjp2.dll` next to `tif2jp2.exe` or add its folder to your PATH.

---

## Usage

Run:

```bash
tif2jp2 --help
```

```
TIFF → JPEG 2000 (JP2) lossless via OpenJPEG FFI

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
      --no-dpi-box        Disable Write DPI into JP2 'res' box
      --xmp-dpi           Write DPI into XMP 'uuid' box [default: on]
      --no-xmp-dpi        Disable Write DPI into XMP 'uuid' box
      --avx2              Enable AVX2 fast path if supported [default: off]
      --no-avx2           Force no AVX2
  -v, --verbose...        Increase verbosity (-v, -vv). 0 = errors only.
      --help              Print help
      --version           Print version
```

---

## Examples

```bash
# Single file conversion
tif2jp2 scan.tif -o scan.jp2

# Batch conversion with custom tile size and code-blocks
tif2jp2 scan.tif -o scan.jp2 --tile 2048x2048 --block 64x64 --levels 6

# Convert all TIFFs in directory (non-recursive)
tif2jp2 ./scans -o ./output

# Recursive conversion with verbose output
tif2jp2 ./archive --recursive -o ./converted -v

# Force overwrite existing files
tif2jp2 ./scans -o ./output --force

# Use specific ICC profile
tif2jp2 image.tif -o image.jp2 --icc ./profiles/sRGB.icc

# Custom thread count and disable XMP fallback
tif2jp2 img.tif -o img.jp2 --threads 4 --no-xmp-dpi

# Enable AVX2 acceleration
tif2jp2 large.tif -o large.jp2 --avx2

# Minimal output (errors only)
tif2jp2 scan.tif -o scan.jp2

# Verbose output (info level)
tif2jp2 scan.tif -o scan.jp2 -v

# Debug output (maximum verbosity)
tif2jp2 scan.tif -o scan.jp2 -vv
```

---

## Metadata Handling

### DPI / Resolution
- JP2 Resolution Box (`--dpi-box`) → embeds DPI in standard res box  
- XMP Fallback (`--xmp-dpi`) → adds XMP metadata  

Both are **enabled by default**.  
Converter automatically converts TIFF resolution units (inch/cm) → pixels-per-meter.

### ICC Profiles
- **Automatic**: extracted from TIFF if present  
- **Manual override**: `--icc profile.icc`  

⚠️ Some TIFF ICCs may be incomplete → for archival use, supply a known good profile.

---

## Performance Tips
- **Threading**: `--threads 0` (default, auto-detect cores)  
- **Tiling**: larger tiles (`--tile 2048x2048`) help large images  
- **AVX2**: enable `--avx2` for faster buffer processing  
- **Batch**: skips already processed files unless `--force`  

---

## Troubleshooting

- `"No input TIFFs found"` → check path or use `--recursive`  
- `"File is not a TIFF"` → only `.tif`/`.tiff` supported  
- **Missing OpenJPEG** → install `openjp2.dll` (Windows) or `libopenjp2` (Linux)  
- **Unsupported** → CMYK & alpha channels not supported (convert first)  

**Windows DLL errors:**  
Put `openjp2.dll` next to exe or add its folder to PATH.

---

## Limitations
❌ CMYK color space not supported  
❌ Alpha channels not supported  
❌ Limited to 8/16-bit grayscale or RGB images  
❌ Progressive decoding not implemented  
