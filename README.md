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

**Goals:** a practical, fast, no-nonsense archival path from TIFF to JP2 for x86_64 machines with AVX2 SIMD; while staying compatible with common JP2 readers, while not sacrificing acrhival level quality (FADGI, Metamorfoze, Czech national standard: [NDK](https://standardy.ndk.cz/ndk/standardy-digitalizace/standardy-pro-obrazova-data))

# Notice

Only Windows version has been tested so far. Only parts of the desired goals have been achieved.

---

## Quick Start

On Windows you can use absolute path (e.g. `.\tif2jp2.exe input.tif`), if tif2jp2 is not in PATH.

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
- **Precincts**: 256×256 for higher resolutions, 128×128 for lowest  
- **SOP/EPH markers**: Enabled by default (`--no-sop`, `--no-eph` to disable)  
- **Tile-parts R**: Enabled by default (`--no-tp-r` to disable)  
- **MCT**: Reversible MCT for RGB (`--no-mct` to disable)  
- **Multi-threaded**: `--threads` sets worker threads (0 = auto)  
- **AVX2 upload**: Accelerated buffer upload (`--avx2`)  
- **Smart file handling**: Skips existing files unless `--force`  
- **Robust error handling**: Continues even if some files fail  

---
## Compliance with the Czech Archival Standard (NDK)

This converter implements the parameters required by the Czech national standard for archival JPEG2000 masters:

| Parameter | Standard | Implemented |
|-----------|----------|-------------|
| Compression | Lossless | ✅ 5/3 reversible |
| Transform | 5-3 filter | ✅ |
| Layers | 1 | ✅ |
| Tiling | 4096×4096 | ✅ (`--tile 4096x4096`) |
| Progression order | RPCL | ✅ |
| Decomposition levels | 5 or 6 | ✅ (`--levels`) |
| Code-block size | 64×64 | ✅ (`--block 64x64`) |
| Precincts | 256×256, 128×128 | ✅ (via `fill_precincts`) |
| SOP markers | Yes | ✅ (default on) |
| EPH markers | Yes | ✅ (default on) |
| Tile-parts (R) | Yes | ✅ (default on) |
| ICC profiles | Yes | ✅ (from TIFF or `--icc`) |
| ROI | No | ✅ (disabled) |
| Embedded metadata | No | ✅ (only optional XMP if `--xmp-dpi`) |
| TLM markers | Yes | ❌ not available in current OpenJPEG Rust bindings |

⚠️ **Note:** TLM (Tile-part Length Markers) are part of the standard but cannot be enabled via the current `openjpeg_sys` bindings, as the `cp_tlm` field is missing. All other parameters are fully supported.

---

## Build from Source

### Prerequisites
1. [**Rust** (stable)](https://www.rust-lang.org/tools/install) and Cargo  
2. OpenJPEG development/runtime libraries  

**Linux:** `libopenjp2` (`libopenjp2-7`, `libopenjp2-7-dev`)  
**Windows:** `openjp2.dll` must be available at runtime. You can install [OpenJPEG](https://github.com/uclouvain/openjpeg/releases) simply by unzipping it next to `tif2jp2.exe`.  

---

**Linux (Debian / Ubuntu):**

```bash
sudo apt-get update
sudo apt-get install -y libopenjp2-7 libopenjp2-7-dev
git clone https://github.com/bezverec/tif2jp2.git
cd tif2jp2
RUSTFLAGS="-C target-cpu=native" cargo build --release
# binary at target/release/tif2jp2
```

**Windows (PowerShell / Terminal):**

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
      --order <ORDER>     Progression order (LRCP|RLCP|RPCL|PCRL|CPRL) [default: RPCL]
      --force             Overwrite existing output files
      --threads <N>       OpenJPEG threads (0 = auto = all cores) [default: 0]
      --icc <PATH>        Path to ICC profile (overrides ICC detected in TIFF)
      --dpi-box           Write DPI into JP2 'res' box [default: on]
      --no-dpi-box        Disable Write DPI into JP2 'res' box
      --xmp-dpi           Write DPI into XMP 'uuid' box [default: on]
      --no-xmp-dpi        Disable Write DPI into XMP 'uuid' box
      --avx2              Enable AVX2 fast path if supported [default: off]
      --no-avx2           Force no AVX2
      --sop               Enable SOP markers [default: on]
      --no-sop            Disable SOP markers
      --eph               Enable EPH markers [default: on]
      --no-eph            Disable EPH markers
      --precincts         Enable precinct partitioning (256x256, 128x128)
      --tp-r              Enable tile-parts split by resolution (R)
      --mct               Enable reversible MCT for RGB [default: on]
      --no-mct            Disable MCT
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

## AI generated code disclosure
The code is AI generated using ChatGPT model 5 and Deepseek v3.x.

---

## Benchmark

My benchmark attempt on Windows 11 (not objective):
- tif2jp2 v0.1.7
- Kakadu v8.4.1

```
Per-file results:

File                                                 Tool    Runs Time_s Size_MB Tile      Block Levels KduLevels Order
----                                                 ----    ---- ------ ------- ----      ----- ------ --------- -----
ilustrovany_zpravodaj_1938-03-17_cislo11_strana5.tif tif2jp2    5 0,414  10,88   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-03-17_cislo11_strana5.tif Kakadu     5 1,168  10,88   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-03-17_cislo11_strana6.tif tif2jp2    5 0,437  12,70   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-03-17_cislo11_strana6.tif Kakadu     5 1,375  12,70   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-05-12_cislo19_strana4.tif tif2jp2    5 0,421  11,03   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-05-12_cislo19_strana4.tif Kakadu     5 1,178  11,02   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-05-12_cislo19_strana5.tif tif2jp2    5 0,442  11,80   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-05-12_cislo19_strana5.tif Kakadu     5 1,252  11,80   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.tif tif2jp2    5 0,433  11,98   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.tif Kakadu     5 1,287  11,98   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana4.tif tif2jp2    5 0,427  10,97   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana4.tif Kakadu     5 1,178  10,97   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana5.tif tif2jp2    5 0,420  10,75   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana5.tif Kakadu     5 1,148  10,75   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana6.tif tif2jp2    5 0,431  12,23   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-23_cislo25_strana6.tif Kakadu     5 1,312  12,23   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-27_cislo29_strana1.tif tif2jp2    5 0,657  20,42   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-27_cislo29_strana1.tif Kakadu     5 2,164  20,42   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-27_cislo29_strana2.tif tif2jp2    5 0,632  19,20   4096x4096 64x64 6      6         RPCL
ilustrovany_zpravodaj_1938-06-27_cislo29_strana2.tif Kakadu     5 2,050  19,20   4096x4096 64x64 6      6         RPCL



Summary:

Tool    Files AvgTime_s AvgSize_MB
----    ----- --------- ----------
tif2jp2    10 0,471     13,20
Kakadu     10 1,411     13,19
```

Benchmark code:
```
# bench.ps1  (Windows PowerShell 5.1 compatible)
param(
  [string]$InputDir = ".",
  [switch]$Recursive,
  [int]$Runs = 3,

  # Encoder params (match tif2jp2 CLI)
  [string]$Tile   = "4096x4096",
  [string]$Block  = "64x64",
  [string]$Levels = "6",        # use "auto" to mirror program's auto
  [string]$Order  = "RPCL",     # LRCP|RLCP|RPCL|PCRL|CPRL

  # Feature toggles (match --sop/--eph/--precincts/--tp-r/--mct in main.rs)
  [switch]$Sop,                 # Start of Packet markers
  [switch]$Eph,                 # End of Packet Header markers
  [switch]$Precincts,           # 256x256 .. 128x128
  [switch]$TpR,                 # tile-parts split by Resolution
  [switch]$Mct,                 # reversible MCT for RGB

  # Perf toggles
  [switch]$Avx2,                # --avx2 (tif2jp2 only)

  # Convenience presets
  [switch]$Archival,            # NDK-like: 4096x4096, 64x64, RPCL, levels=6, SOP/EPH/precincts/tp-r/mct

  # CSV output (auto if empty)
  [string]$Csv = ""
)

$ErrorActionPreference = 'Stop'

# Prefer freshly built binary in target\release
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$tif2 = Join-Path $scriptDir "target\release\tif2jp2.exe"
$kdu  = Join-Path $scriptDir "kdu_compress.exe"
if (-not (Test-Path $tif2)) { $tif2 = Join-Path $scriptDir "tif2jp2.exe" }
if (-not (Test-Path $tif2)) { $tif2 = "tif2jp2.exe" }
if (-not (Test-Path $kdu))  { $kdu  = "kdu_compress.exe" }

Write-Host "Using tif2: $tif2" -ForegroundColor DarkGray
Write-Host "Using kdu : $kdu"  -ForegroundColor DarkGray

# Archival preset (Czech NDK-like)
if ($Archival) {
  $Tile   = "4096x4096"
  $Block  = "64x64"
  $Levels = "6"
  $Order  = "RPCL"
  $Sop = $true; $Eph = $true; $Precincts = $true; $TpR = $true; $Mct = $true
}

# Collect inputs
$files = Get-ChildItem -Path $InputDir -File -Recurse:$Recursive | Where-Object {
  $_.Extension -match '^\.(tif|tiff)$'
}
if (-not $files) {
  Write-Host "No .tif/.tiff files found in '$InputDir'." -ForegroundColor Yellow
  exit 1
}

# Helpers ----------------------------------------------------------------------

function Get-ImageDimensions([string]$Path) {
  Add-Type -AssemblyName System.Drawing -ErrorAction SilentlyContinue
  $img = [System.Drawing.Image]::FromFile($Path)
  try { return @{ W = [int]$img.Width; H = [int]$img.Height } }
  finally { $img.Dispose() }
}

function Get-AutoLevels([string]$Path) {
  $d = Get-ImageDimensions -Path $Path
  $m = [math]::Min($d.W, $d.H)
  if ($m -le 0) { return 3 }
  $k = [math]::Floor([math]::Log($m, 2)) - 1
  if ($k -lt 1) { $k = 1 }
  [int][math]::Max(3, [math]::Min(8, $k))
}

# Kakadu precinct spec: 256x256 for higher resolutions, 128x128 for the lowest
function Get-KduPrecinctsSpec([int]$levels) {
  if ($levels -le 1) { return "{128,128}" }
  $spec = @()
  for ($r = 0; $r -lt $levels; $r++) {
    if ($r -eq $levels - 1) { $spec += "{128,128}" } else { $spec += "{256,256}" }
  }
  ($spec -join ",")
}

# Warm-up
Write-Host "Warming up encoders..." -ForegroundColor DarkGray
$null = Measure-Command { & $tif2 --help | Out-Null } 
$null = Measure-Command { & $kdu -usage  | Out-Null } 

# Main loop --------------------------------------------------------------------

$results = @()
$blockParts = $Block.Split('x', [System.StringSplitOptions]::RemoveEmptyEntries)
$tileParts  = $Tile.Split('x',  [System.StringSplitOptions]::RemoveEmptyEntries)

foreach ($f in $files) {
  $input = $f.FullName
  $base  = [System.IO.Path]::GetFileNameWithoutExtension($f.Name)

  $out1 = Join-Path $f.DirectoryName "$base.tif2jp2.jp2"
  $out2 = Join-Path $f.DirectoryName "$base.kakadu.jp2"

  # Decide levels
  $tif2Levels = $Levels       # can be "auto" (tif2 supports) or numeric
  $kduLevels  = $Levels
  if ($Levels -eq "auto") {
    $kduLevels = (Get-AutoLevels -Path $input).ToString()
  }
  $kduLevelsInt = [int]$kduLevels

  # ---- tif2jp2 args (match main.rs exactly) ----------------------------------
  $tif2jp2Args = @(
    "`"$input`"", "-o", "`"$out1`"",
    "--threads", "0",
    "--tile", $Tile,
    "--block", $Block,
    "--levels", $tif2Levels,
    "--order", $Order,
    "--force"             # always overwrite outputs for clean timing
  )
  if ($Avx2)      { $tif2jp2Args += "--avx2" }
  if ($Sop)       { $tif2jp2Args += "--sop" }
  if ($Eph)       { $tif2jp2Args += "--eph" }
  if ($Precincts) { $tif2jp2Args += "--precincts" }
  if ($TpR)       { $tif2jp2Args += "--tp-r" }
  if ($Mct)       { $tif2jp2Args += "--mct" }

  # ---- Kakadu args (closest equivalents) -------------------------------------
  $kduArgs = @(
    "-i", "`"$input`"", "-o", "`"$out2`"",
    "-rate", "-", "Creversible=yes",
    "Clevels=$kduLevels",
    "Cblk={$($blockParts[0]),$($blockParts[1])}",
    "Stiles={$($tileParts[0]),$($tileParts[1])}",
    "Corder=$Order",
    "-num_threads", "0"
  )
  if ($Sop)       { $kduArgs += "Cuse_sop=yes" }
  if ($Eph)       { $kduArgs += "Cuse_eph=yes" }
  if ($Precincts) { $kduArgs += "Cprecincts=$(Get-KduPrecinctsSpec -levels $kduLevelsInt)" }
  if ($TpR)       { $kduArgs += "ORGtparts=R" }
  # MCT: Kakadu uses reversible RCT automatically in Creversible=yes mode

  Write-Host ("Processing {0}  (tif2jp2 levels: {1}, Kakadu Clevels={2})" -f $f.Name, $tif2Levels, $kduLevels) -ForegroundColor Cyan

  # Repeat to average (remove outputs between runs)
  $acc1 = 0.0; $acc2 = 0.0
  for ($i = 1; $i -le $Runs; $i++) {
    if (Test-Path $out1) { Remove-Item $out1 -Force }
    $t1 = Measure-Command { & $tif2 @tif2jp2Args | Out-Null }
    $acc1 += $t1.TotalSeconds

    if (Test-Path $out2) { Remove-Item $out2 -Force }
    $t2 = Measure-Command { & $kdu  @kduArgs      | Out-Null }
    $acc2 += $t2.TotalSeconds
  }

  $avg1 = $acc1 / [double]$Runs
  $avg2 = $acc2 / [double]$Runs

  # Sizes
  $size1 = 0; if (Test-Path $out1) { $size1 = (Get-Item $out1).Length }
  $size2 = 0; if (Test-Path $out2) { $size2 = (Get-Item $out2).Length }

  # Record rows
  $results += [PSCustomObject]@{
    File      = $f.Name
    Tool      = "tif2jp2"
    Runs      = $Runs
    Time_s    = [double]$avg1
    Size_MB   = [double]($size1 / 1MB)
    Tile      = $Tile
    Block     = $Block
    Levels    = $tif2Levels
    KduLevels = $kduLevels
    Order     = $Order
    AVX2      = $Avx2.IsPresent
    SOP       = $Sop.IsPresent
    EPH       = $Eph.IsPresent
    Precincts = $Precincts.IsPresent
    TpR       = $TpR.IsPresent
    MCT       = $Mct.IsPresent
  }
  $results += [PSCustomObject]@{
    File      = $f.Name
    Tool      = "Kakadu"
    Runs      = $Runs
    Time_s    = [double]$avg2
    Size_MB   = [double]($size2 / 1MB)
    Tile      = $Tile
    Block     = $Block
    Levels    = $tif2Levels
    KduLevels = $kduLevels
    Order     = $Order
    AVX2      = $false
    SOP       = $Sop.IsPresent
    EPH       = $Eph.IsPresent
    Precincts = $Precincts.IsPresent
    TpR       = $TpR.IsPresent
    MCT       = $false
  }
}

# Summary ----------------------------------------------------------------------
$summary = $results |
  Group-Object Tool |
  ForEach-Object {
    $avgTime = ($_.Group | Measure-Object -Property Time_s -Average).Average
    $avgSize = ($_.Group | Measure-Object -Property Size_MB -Average).Average
    [PSCustomObject]@{
      Tool       = $_.Name
      Files      = $_.Count
      AvgTime_s  = [double]$avgTime
      AvgSize_MB = [double]$avgSize
    }
  }

# Output tables ----------------------------------------------------------------
Write-Host ""
Write-Host "Per-file results:" -ForegroundColor Green
$results |
  Select-Object File,Tool,Runs,
    @{n='Time_s';e={'{0:N3}' -f $_.Time_s}},
    @{n='Size_MB';e={'{0:N2}' -f $_.Size_MB}},
    Tile,Block,Levels,KduLevels,Order,AVX2,SOP,EPH,Precincts,TpR,MCT |
  Format-Table -AutoSize

Write-Host ""
Write-Host "Summary:" -ForegroundColor Green
$summary |
  Select-Object Tool,Files,
    @{n='AvgTime_s';e={'{0:N3}' -f $_.AvgTime_s}},
    @{n='AvgSize_MB';e={'{0:N2}' -f $_.AvgSize_MB}} |
  Format-Table -AutoSize

# CSV export (always on; create default file if not provided)
if (-not $Csv -or $Csv -eq "") {
  $stamp = Get-Date -Format "yyyyMMdd-HHmmss"
  $Csv = ".\bench-$stamp.csv"
}
$results | Export-Csv -Path $Csv -NoTypeInformation -Encoding UTF8
Write-Host "Saved CSV to $Csv" -ForegroundColor DarkGreen

Write-Host ""
Write-Host "Tip: Archival preset:" -ForegroundColor DarkGray
Write-Host "  .\bench.ps1 -InputDir . -Recursive -Runs 3 -Archival -Avx2" -ForegroundColor DarkGray

```
