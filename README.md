# tif2jp2 
![version](https://img.shields.io/badge/dynamic/toml?url=https://raw.githubusercontent.com/bezverec/tif2jp2/main/Cargo.toml&query=$.package.version&label=version&prefix=v) ![GitHub top language](https://img.shields.io/github/languages/top/bezverec/tif2jp2) ![GitHub last commit](https://img.shields.io/github/last-commit/bezverec/tif2jp2) ![GitHub commit activity](https://img.shields.io/github/commit-activity/m/bezverec/tif2jp2) ![GitHub repo size](https://img.shields.io/github/repo-size/bezverec/tif2jp2) ![LoC](https://tokei.rs/b1/github/bezverec/tif2jp2) ![Dependencies](https://deps.rs/repo/github/bezverec/tif2jp2/status.svg)


TIFF to JPEG2000 (JP2) lossless converter built in Rust with a thin FFI layer over OpenJPEG.

**Goals:** a practical, fast, no-nonsense archival path from TIFF to JP2 primarily for x86_64 machines with AVX2 SIMD; while staying compatible with common JP2 readers, while not sacrificing archival level of quality (FADGI, Metamorfoze, primarily Czech national standard: [NDK](https://standardy.ndk.cz/ndk/standardy-digitalizace/standardy-pro-obrazova-data))

# Notice

Only Windows version has been somewhat tested so far. I have also been able to compile it on Linux, macOS (ARM) and Android 16. Only parts of the desired goals have been achieved. This is a hobby project written for educational purposes. In it's current state it is not intended as a production use tool. The validity of the output files has not yet been fully established, I have only achieved jhove and jpylyzer validity (see below). You may experience various bugs, inconsistencies, Czech language left overs, AI slop, unsafe code (this is by design), crashes and other problems.

---

## Quick Start

On Windows you can use absolute path (e.g. `.\tif2jp2.exe input.tif`), if tif2jp2 is not in PATH.

**Convert a single TIFF to JP2 (same folder/name)**

```bash
./tif2jp2 input.tif
```

**Convert into a specific output file**

```bash
./tif2jp2 input.tif -o output.jp2
```

**Convert all TIFFs in a folder (non-recursive)**

```bash
./tif2jp2 ./scans -o ./out
```

**Convert recursively with verbose progress**

```bash
./tif2jp2 ./archive --recursive -o ./out -v
```

**Force overwrite existing files**

```bash
./tif2jp2 ./scans -o ./out --force
```
---
## Compliance with the Czech Archival Standard (NDK)

This converter implements the parameters required by the Czech national standard for archival JPEG2000 masters:
- ON by default
- can be also enforced with flag `--archival-master-ndk` or `--archival`
- not officially validated, tested only on a few samples in a specific workflow (scanner settings, postprocessing)

| Parameter | Standard | Implemented |
|-----------|----------|-------------|
| Compression | Lossless | ✅ |
| Transform | 5-3 filter | ✅ |
| Layers | 1 | ✅ |
| Tiling | 4096×4096 | ✅ |
| Progression order | RPCL | ✅ |
| Decomposition levels | 5 or 6 | ✅ |
| Code-block size | 64×64 | ✅ |
| Precincts | 2× 256×256, 4× 128×128 | ✅ |
| SOP markers | Yes | ✅ |
| EPH markers | Yes | ✅ |
| Tile-parts (R) | Yes | ✅ |
| ICC profiles | Yes | ✅ |
| ROI | No | ✅ |
| Embedded metadata | No | ✅ |
| TLM markers | Yes | ✅ |
| CBLK bypass | Yes | ✅ |

---

## Build from Source

### Prerequisites
1. install [Git](https://git-scm.com/) *(optional, you can also download ZIP/files from this repo instead of using `git clone`)*
2. install [**Rust** (stable)](https://www.rust-lang.org/tools/install) and Cargo

---

**Linux (Debian / Ubuntu / Android Terminal):**

```bash
sudo apt-get update
sudo apt-get upgrade
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

**macOS (ARM):**
```zsh
brew update
brew upgrade
git clone https://github.com/bezverec/tif2jp2.git
cd tif2jp2
RUSTFLAGS="-C target-cpu=native" cargo build --release
# binary at target/release/tif2jp2
```

---

## Usage

Run:

```bash
./tif2jp2 --help
```

```
TIFF to JPEG2000 (JP2) lossless via OpenJPEG FFI

Usage: ./tif2jp2 [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Input file or directory (use --recursive for subdirectories)

Options:
  -o, --output <OUTPUT>      Output file or directory (mirrors input structure if directory)
      --recursive            Recursively traverse the input directory
      --tile <WxH>           Tile size, e.g. 1024x1024 [default: 4096x4096]
      --block <WxH>          Code-block size, e.g. 64x64 [default: 64x64]
      --levels <NUM|auto>    Number of resolutions [default: 6]
      --force                Overwrite existing output files
      --threads <N>          OpenJPEG threads (0 = auto = all cores) [default: 0]
      --icc <PATH>           Path to ICC profile (overrides ICC detected in TIFF)
      --order <ORDER>        Progression order (LRCP|RLCP|RPCL|PCRL|CPRL) [default: RPCL]
      --archival-master-ndk  Archival master NDK preset (alias: --archival). Forces RPCL, 4096x4096 tiles, 64x64 blocks, levels=6, SOP/EPH on, precincts on (256..128), tile-parts R, reversible MCT on, TLM on
      --dpi-box              Write DPI into JP2 'res' box [default: on]
      --no-dpi-box           Disable Write DPI into JP2 'res' box
      --xmp-dpi              Write DPI into XMP 'uuid' box [default: off]
      --no-xmp-dpi           Disable Write DPI into XMP 'uuid' box
      --avx2                 Enable AVX2 fast path if supported [default: off]
      --no-avx2              Force no AVX2
      --tp-r                 Enable tile-parts split by Resolution (R) [default: on]
      --no-tp-r              Disable tile-parts split by Resolution (R)
      --precincts            Enable precinct partitioning (256x256 … 128x128) [default: on]
      --no-precincts         Disable precinct partitioning
      --sop                  Enable SOP markers (Start of Packet) [default: on]
      --no-sop               Disable SOP markers
      --eph                  Enable EPH markers (End of Packet Header) [default: on]
      --no-eph               Disable EPH markers
      --mct                  Enable reversible MCT for RGB [default: on]
      --no-mct               Disable reversible MCT for RGB
      --tlm                  Enable TLM markers (Tile-part Length) [NDK preset: on]
      --no-tlm               Disable TLM markers
      --plt                  Enable PLT markers (Packet Length in TPH) [default: off]
      --no-plt               Disable PLT markers
  -h, --help                 Print help
  -V, --version              Print version
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

### ICC Profiles
- **Automatic**: extracted from TIFF if present  
- **Manual override**: `--icc profile.icc`  

⚠️ Some TIFF ICCs may be incomplete → for archival use, supply a known good profile.

---

## Performance Tips
- **Threading**: `--threads 0` (default, auto-detect cores)    
- **AVX2**: enable `--avx2` for faster buffer processing  
- **Batch**: skips already processed files unless `--force`  

---

## Troubleshooting

- `"No input TIFFs found"` → check path or use `--recursive`  
- `"File is not a TIFF"` → only `.tif`/`.tiff` supported  
- **Unsupported** → CMYK & alpha channels (RGBA) not supported (convert to RGB first)  

---

## Limitations
❌ CMYK color space not supported  
❌ Alpha channels (RGBA) not supported  
❌ Limited to 8/16-bit grayscale or RGB images  
❌ Progressive decoding not implemented  

## AI generated code disclosure
The code is AI generated using ChatGPT model 5 and Deepseek v3.x.

---

## Benchmark

My benchmark attempt on Windows 11 *(not reliable, your results may vary significantly; I will be happy to see any benchmark and/or real-world results :))*
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

---

## Basic validity

Results may differ with different tif2jp2 flags. Mostly default settings have been tested.

### jhove

```
Jhove (Rel. 1.34.0, 2025-07-02)
 Date: 2025-09-22 20:33:31 SELČ
 RepresentationInformation: .\ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.jp2
  ReportingModule: JPEG2000-hul, Rel. 1.4.5 (2025-03-12)
  LastModified: 2025-09-03 21:45:41 SELČ
  Size: 12563820
  Format: JPEG 2000
  Status: Well-Formed and valid
  SignatureMatches:
   JPEG2000-hul
  MIMEtype: image/jp2
  Profile: JP2
  JPEG2000Metadata:
   Brand: jp2
   MinorVersion: 0
   Compatibility: jp2
   ColorspaceUnknown: false
   CaptureResolution:
    HorizResolution:
     Numerator: 46
     Denominator: 8960
     Exponent: 0
    VertResolution:
     Numerator: 11811
     Denominator: 1
     Exponent: 1
   ColorSpecs:
    ColorSpec:
     Method: Enumerated Colorspace
     Precedence: 0
     Approx: 0
     EnumCS: sRGB
```

### jpylyzer

```
<?xml version='1.0' encoding='UTF-8'?>
<jpylyzer xmlns="http://openpreservation.org/ns/jpylyzer/v2/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://openpreservation.org/ns/jpylyzer/v2/ http://jpylyzer.openpreservation.org/jpylyzer-v-2-2.xsd">
<toolInfo>
    <toolName>jpylyzer.exe</toolName>
    <toolVersion>2.2.1</toolVersion>
</toolInfo>
<file>
    <fileInfo>
        <fileName>ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.jp2</fileName>
        <filePath>G:\temp\jpeg2000\jpylyzer\ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.jp2</filePath>
        <fileSizeInBytes>12563540</fileSizeInBytes>
        <fileLastModified>2025-09-05T22:27:45.752050</fileLastModified>
    </fileInfo>
    <statusInfo>
        <success>True</success>
    </statusInfo>
    <isValid format="jp2">True</isValid>
    <tests/>
    <properties>
        <signatureBox/>
        <fileTypeBox>
            <br>jp2 </br>
            <minV>0</minV>
            <cL>jp2 </cL>
        </fileTypeBox>
        <jp2HeaderBox>
            <imageHeaderBox>
                <height>3129</height>
                <width>3481</width>
                <nC>3</nC>
                <bPCSign>unsigned</bPCSign>
                <bPCDepth>8</bPCDepth>
                <c>jpeg2000</c>
                <unkC>no</unkC>
                <iPR>no</iPR>
            </imageHeaderBox>
            <colourSpecificationBox>
                <meth>Enumerated</meth>
                <prec>0</prec>
                <approx>0</approx>
                <enumCS>sRGB</enumCS>
            </colourSpecificationBox>
            <resolutionBox>
                <captureResolutionBox>
                    <vRcN>11811</vRcN>
                    <vRcD>1</vRcD>
                    <hRcN>11811</hRcN>
                    <hRcD>1</hRcD>
                    <vRcE>0</vRcE>
                    <hRcE>0</hRcE>
                    <vRescInPixelsPerMeter>11811.0</vRescInPixelsPerMeter>
                    <hRescInPixelsPerMeter>11811.0</hRescInPixelsPerMeter>
                    <vRescInPixelsPerInch>300.0</vRescInPixelsPerInch>
                    <hRescInPixelsPerInch>300.0</hRescInPixelsPerInch>
                </captureResolutionBox>
                <displayResolutionBox>
                    <vRdN>11811</vRdN>
                    <vRdD>1</vRdD>
                    <hRdN>11811</hRdN>
                    <hRdD>1</hRdD>
                    <vRdE>0</vRdE>
                    <hRdE>0</hRdE>
                    <vResdInPixelsPerMeter>11811.0</vResdInPixelsPerMeter>
                    <hResdInPixelsPerMeter>11811.0</hResdInPixelsPerMeter>
                    <vResdInPixelsPerInch>300.0</vResdInPixelsPerInch>
                    <hResdInPixelsPerInch>300.0</hResdInPixelsPerInch>
                </displayResolutionBox>
            </resolutionBox>
        </jp2HeaderBox>
        <contiguousCodestreamBox>
            <siz>
                <lsiz>47</lsiz>
                <rsiz>0</rsiz>
                <capability>ISO/IEC 15444-1</capability>
                <xsiz>3481</xsiz>
                <ysiz>3129</ysiz>
                <xOsiz>0</xOsiz>
                <yOsiz>0</yOsiz>
                <xTsiz>4096</xTsiz>
                <yTsiz>4096</yTsiz>
                <xTOsiz>0</xTOsiz>
                <yTOsiz>0</yTOsiz>
                <numberOfTiles>1</numberOfTiles>
                <csiz>3</csiz>
                <ssizSign>unsigned</ssizSign>
                <ssizDepth>8</ssizDepth>
                <xRsiz>1</xRsiz>
                <yRsiz>1</yRsiz>
                <ssizSign>unsigned</ssizSign>
                <ssizDepth>8</ssizDepth>
                <xRsiz>1</xRsiz>
                <yRsiz>1</yRsiz>
                <ssizSign>unsigned</ssizSign>
                <ssizDepth>8</ssizDepth>
                <xRsiz>1</xRsiz>
                <yRsiz>1</yRsiz>
            </siz>
            <cod>
                <lcod>18</lcod>
                <precincts>user defined</precincts>
                <sop>yes</sop>
                <eph>yes</eph>
                <order>RPCL</order>
                <layers>1</layers>
                <multipleComponentTransformation>yes</multipleComponentTransformation>
                <levels>5</levels>
                <codeBlockWidth>64</codeBlockWidth>
                <codeBlockHeight>64</codeBlockHeight>
                <codingBypass>no</codingBypass>
                <resetOnBoundaries>no</resetOnBoundaries>
                <termOnEachPass>no</termOnEachPass>
                <vertCausalContext>no</vertCausalContext>
                <predTermination>no</predTermination>
                <segmentationSymbols>no</segmentationSymbols>
                <transformation>5-3 reversible</transformation>
                <precinctSizeX>128</precinctSizeX>
                <precinctSizeY>128</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
            </cod>
            <qcd>
                <lqcd>19</lqcd>
                <qStyle>no quantization</qStyle>
                <guardBits>2</guardBits>
                <epsilon>8</epsilon>
                <epsilon>9</epsilon>
                <epsilon>9</epsilon>
                <epsilon>10</epsilon>
                <epsilon>9</epsilon>
            </qcd>
            <tlm/>
            <com>
                <lcom>37</lcom>
                <rcom>ISO/IEC 8859-15 (Latin)</rcom>
                <comment>Created by OpenJPEG version 2.5.3</comment>
            </com>
            <ppmCount>0</ppmCount>
            <plmCount>0</plmCount>
            <tileParts>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>17402</psot>
                        <tpsot>0</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>50608</psot>
                        <tpsot>1</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>209372</psot>
                        <tpsot>2</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>833287</psot>
                        <tpsot>3</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>3018447</psot>
                        <tpsot>4</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
                <tilePart>
                    <sot>
                        <lsot>10</lsot>
                        <isot>0</isot>
                        <psot>8434126</psot>
                        <tpsot>5</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
            </tileParts>
        </contiguousCodestreamBox>
        <compressionRatio>2.6</compressionRatio>
    </properties>
    <warnings/>
</file>
</jpylyzer>
```

### opj_dump


```
PS G:\temp\jpeg2000\openjpeg-v2.5.3-windows-x64\bin> .\opj_dump.exe -i .\ilustrovany_zpravodaj_1938-06-23_cislo25_strana3.jp2

[INFO] Start to read j2k main header (129).
[INFO] Main header has been correctly decoded.
Image info {
         x0=0, y0=0
         x1=3481, y1=3129
         numcomps=3
                 component 0 {
                 dx=1, dy=1
                 prec=8
                 sgnd=0
        }
                 component 1 {
                 dx=1, dy=1
                 prec=8
                 sgnd=0
        }
                 component 2 {
                 dx=1, dy=1
                 prec=8
                 sgnd=0
        }
}
Codestream info from main header: {
         tx0=0, ty0=0
         tdx=4096, tdy=4096
         tw=1, th=1
         default tile {
                 csty=0x7
                 prg=0x2
                 numlayers=1
                 mct=1
                 comp 0 {
                         csty=0x1
                         numresolutions=6
                         cblkw=2^6
                         cblkh=2^6
                         cblksty=0
                         qmfbid=1
                         preccintsize (w,h)=(7,7) (8,8) (8,8) (8,8) (8,8) (8,8)
                         qntsty=0
                         numgbits=2
                         stepsizes (m,e)=(0,8) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10)
                         roishift=0
                 }
                 comp 1 {
                         csty=0x1
                         numresolutions=6
                         cblkw=2^6
                         cblkh=2^6
                         cblksty=0
                         qmfbid=1
                         preccintsize (w,h)=(7,7) (8,8) (8,8) (8,8) (8,8) (8,8)
                         qntsty=0
                         numgbits=2
                         stepsizes (m,e)=(0,8) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10)
                         roishift=0
                 }
                 comp 2 {
                         csty=0x1
                         numresolutions=6
                         cblkw=2^6
                         cblkh=2^6
                         cblksty=0
                         qmfbid=1
                         preccintsize (w,h)=(7,7) (8,8) (8,8) (8,8) (8,8) (8,8)
                         qntsty=0
                         numgbits=2
                         stepsizes (m,e)=(0,8) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10) (0,9) (0,9) (0,10)
                         roishift=0
                 }
         }
}
Codestream index from main header: {
         Main header start position=129
         Main header end position=296
         Marker list: {
                 type=0xff4f, pos=129, len=2
                 type=0xff51, pos=131, len=49
                 type=0xff52, pos=180, len=20
                 type=0xff5c, pos=200, len=21
                 type=0xff55, pos=221, len=36
                 type=0xff64, pos=257, len=39
         }
         Tile index: {
                 nb of tile-part in tile [0]=6
                         tile-part[0]: star_pos=296, end_header=0, end_pos=17698.
                         tile-part[1]: star_pos=17698, end_header=0, end_pos=68306.
                         tile-part[2]: star_pos=68306, end_header=0, end_pos=277678.
                         tile-part[3]: star_pos=277678, end_header=0, end_pos=1110965.
                         tile-part[4]: star_pos=1110965, end_header=0, end_pos=4129412.
                         tile-part[5]: star_pos=4129412, end_header=0, end_pos=12563538.
         }
}

```

