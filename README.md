# tif2jp2

TIFF to JPEG2000 (JP2) lossless converter built in Rust with a thin FFI layer over OpenJPEG.

✅ Lossless 5/3 wavelet (reversible)  
✅ Preserves/reserializes DPI into JP2 res box (and optional XMP fallback)  
✅ ICC profile attachment (from TIFF or a user-supplied profile)  
✅ Tiled + code-block encoding  
✅ Multi-threaded via OpenJPEG  
✅ AVX2 fast path upload (opt-in)  
✅ Single file or batch/recursive mode  
✅ Friendly CLI with --help, --version, --verbose  

Goal: a practical, fast, no-nonsense archival path from TIFF to JP2 while staying compatible with common JP2 readers.

---

## Quick Start

```bash
# Convert a single TIFF to JP2 (same folder/name)
tif2jp2 input.tif

# Convert into a specific output file
tif2jp2 input.tif -o output.jp2

# Convert all TIFFs in a folder (non-recursive)
tif2jp2 ./scans -o ./out

# Convert recursively with verbose progress
tif2jp2 ./archive --recursive -o ./out -v
```

---

## Features

- **Lossless JP2:** OpenJPEG reversible 5/3 transform.  
- **DPI preservation:** writes resolution to `jp2h/resc+resd` (and optional XMP).  
- **ICC attachment:** embeds ICC from TIFF if available, or from `--icc` file.  
- **Tiling & code-blocks:** `--tile WxH` and `--block WxH` for efficient encoding.  
- **Multi-threaded:** `--threads` sets OpenJPEG worker threads (0 = auto).  
- **AVX2 upload:** accelerated upload to OpenJPEG planes, enabled explicitly via `--avx2`.  

---

## Usage

Run `tif2jp2 --help` anytime:

```
Usage: tif2jp2 [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file or directory (use --recursive for subdirectories)

Options:
  -o, --output <OUTPUT>        Output file or directory (mirrors input structure if directory)
      --recursive              Recursively traverse the input directory
      --tile <WxH>             Tile size, e.g. 1024x1024 [default: 1024x1024]
      --block <WxH>            Code-block size, e.g. 64x64 [default: 64x64]
      --levels <NUM|auto>      Number of resolutions [default: auto]
      --force                  Overwrite existing output files
      --threads <N>            OpenJPEG threads (0 = auto = all cores) [default: 0]
      --icc <PATH>             Path to ICC profile (overrides ICC detected in TIFF)
      --dpi-box                Write DPI into JP2 'res' box [default: on]
      --no-dpi-box             Disable Write DPI into JP2 'res' box
      --xmp-dpi                Write DPI into XMP 'uuid' box [default: on]
      --no-xmp-dpi             Disable Write DPI into XMP 'uuid' box
      --avx2                   Enable AVX2 fast path if supported [default: off]
      --no-avx2                Force no AVX2
  -v, --verbose                Increase verbosity (-v, -vv)
      --version                Print version
      --help                   Print help
```

---

## Examples

```bash
# Explicit tile/code-block and 6 resolution levels
tif2jp2 scan.tif -o scan.jp2 --tile 2048x2048 --block 64x64 --levels 6

# Recursive batch with mirrored structure, verbose
tif2jp2 ./input --recursive -o ./out -v

# Override ICC profile
tif2jp2 image.tif -o image.jp2 --icc ./profiles/sRGB.icc

# All cores, disable XMP fallback
tif2jp2 img.tif -o img.jp2 --threads 0 --no-xmp-dpi

# Enable AVX2 fast path (if CPU supports it)
tif2jp2 large.tif -o large.jp2 --avx2
```

---

## Metadata Handling

- **DPI in JP2 (res box):** Written by default (`--dpi-box`), disable with `--no-dpi-box`.  
- **XMP DPI fallback:** Enabled by default (`--xmp-dpi`), disable with `--no-xmp-dpi`.  
- **ICC profiles:** Taken from TIFF tag 34675, or overridden with `--icc path.icc`.  

---

## Performance

- **Threads:** `--threads 0` = all cores.  
- **Tiles:** Larger tiles improve throughput but use more RAM.  
- **Code-blocks:** Default 64x64 is a good balance.  
- **Levels:** `auto` chooses based on image size (clamped 3–8).  
- **AVX2 path:** Requires explicit `--avx2` (default off).  

---

## Troubleshooting

- **“No input TIFFs found”** → check path or add `--recursive`.  
- **“File is not a TIFF”** → only `.tif/.tiff` supported.  
- **“openjp2.dll not found” (Windows)** → place DLL next to EXE or add to PATH.  
- **Viewer shows 72 DPI** → enable `--xmp-dpi` (default).  
- **TIFF with alpha/CMYK** → not supported, flatten/convert to RGB or Gray.  
