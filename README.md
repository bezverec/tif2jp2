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

### jpylyzer (tif2jp2 v0.2.1 fix for NDK archival profile - added CBLK bypass and precincts fix)

```
PS C:\temp\tif2jp2\target\release\jpylyzer> ./jpylyzer.exe .\Vlaltxt.jp2
<?xml version='1.0' encoding='UTF-8'?>
<jpylyzer xmlns="http://openpreservation.org/ns/jpylyzer/v2/" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:schemaLocation="http://openpreservation.org/ns/jpylyzer/v2/ http://jpylyzer.openpreservation.org/jpylyzer-v-2-2.xsd">
<toolInfo>
    <toolName>jpylyzer.exe</toolName>
    <toolVersion>2.2.1</toolVersion>
</toolInfo>
<file>
    <fileInfo>
        <fileName>Vlaltxt.jp2</fileName>
        <filePath>C:\temp\tif2jp2\target\release\jpylyzer\Vlaltxt.jp2</filePath>
        <fileSizeInBytes>4382364</fileSizeInBytes>
        <fileLastModified>2025-10-03T15:28:13.717107</fileLastModified>
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
                <height>2142</height>
                <width>1344</width>
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
                <xsiz>1344</xsiz>
                <ysiz>2142</ysiz>
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
                <codingBypass>yes</codingBypass>
                <resetOnBoundaries>no</resetOnBoundaries>
                <termOnEachPass>no</termOnEachPass>
                <vertCausalContext>no</vertCausalContext>
                <predTermination>no</predTermination>
                <segmentationSymbols>no</segmentationSymbols>
                <transformation>5-3 reversible</transformation>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>256</precinctSizeX>
                <precinctSizeY>256</precinctSizeY>
                <precinctSizeX>128</precinctSizeX>
                <precinctSizeY>128</precinctSizeY>
                <precinctSizeX>128</precinctSizeX>
                <precinctSizeY>128</precinctSizeY>
                <precinctSizeX>128</precinctSizeX>
                <precinctSizeY>128</precinctSizeY>
                <precinctSizeX>128</precinctSizeX>
                <precinctSizeY>128</precinctSizeY>
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
                        <psot>4733</psot>
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
                        <psot>14172</psot>
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
                        <psot>63216</psot>
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
                        <psot>277809</psot>
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
                        <psot>1070399</psot>
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
                        <psot>2951737</psot>
                        <tpsot>5</tpsot>
                        <tnsot>6</tnsot>
                    </sot>
                    <pltCount>0</pltCount>
                    <pptCount>0</pptCount>
                </tilePart>
            </tileParts>
        </contiguousCodestreamBox>
        <compressionRatio>1.97</compressionRatio>
    </properties>
    <warnings/>
</file>
</jpylyzer>
PS C:\temp\tif2jp2\target\release\jpylyzer>
```

### opj_dump (tif2jp2 v0.2.1)

```
PS C:\tif2jp2\target\release\openjpeg-v2.5.4-windows-x64\bin> .\opj_dump.exe -i .\Vlaltxt.jp2

[INFO] Start to read j2k main header (129).
[INFO] Main header has been correctly decoded.
Image info {
         x0=0, y0=0
         x1=1344, y1=2142
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
                         cblksty=0x1
                         qmfbid=1
                         preccintsize (w,h)=(8,8) (8,8) (7,7) (7,7) (7,7) (7,7)
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
                         cblksty=0x1
                         qmfbid=1
                         preccintsize (w,h)=(8,8) (8,8) (7,7) (7,7) (7,7) (7,7)
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
                         cblksty=0x1
                         qmfbid=1
                         preccintsize (w,h)=(8,8) (8,8) (7,7) (7,7) (7,7) (7,7)
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
                         tile-part[0]: star_pos=296, end_header=0, end_pos=5029.
                         tile-part[1]: star_pos=5029, end_header=0, end_pos=19201.
                         tile-part[2]: star_pos=19201, end_header=0, end_pos=82417.
                         tile-part[3]: star_pos=82417, end_header=0, end_pos=360226.
                         tile-part[4]: star_pos=360226, end_header=0, end_pos=1430625.
                         tile-part[5]: star_pos=1430625, end_header=0, end_pos=4382362.
         }
}
PS C:\tif2jp2\target\release\openjpeg-v2.5.4-windows-x64\bin>

```

## HexDump (tif2jp2 v0.2.1 - TLM presence check)

```
PS C:\tif2jp2\target\release\hexdump-2.1.0> .\hexdump.exe -C .\Vlaltxt.jp2 | Select-String -Pattern 'FF 55'

0000d0  50 48 48 50 48 48 50 48 48 50 48 48 50 ff 55 00  PHHPHHPHHPHHP.U.
010b60  ff 55 db e4 56 de 44 bf a0 8a 37 75 a6 53 67 26  .U..V.D...7u.Sg&
012a70  ff 55 61 f9 12 6d ba bf aa 7f 45 f3 14 4c 66 6e  .Ua..m....E..Lfn
019270  16 07 ca 9f ab 9f df 73 08 c5 98 ff 55 c1 70 1d  .......s....U.p.
0308d0  74 ab ee 14 b1 67 41 6f ae ff 55 7f 26 5a 39 5b  t....gAo..U.&Z9[
031200  7e 4f af 2b 97 fd 5d 75 ff 55 68 8a 3a c0 15 d7  ~O.+..]u.Uh.:...
0317e0  f7 b1 41 d5 ff 55 6e b3 4e 01 63 2a 9e 5c b2 49  ..A..Un.N.c*.\.I
033960  5f 2a 97 da f8 bb 95 37 60 73 ec ab 86 6c ff 55  _*.....7`s...l.U
038ac0  a1 19 4b 4f 2c bf ca 8b 1e 8a ee 07 f0 ff 55 d9  ..KO,.........U.
039220  66 9b 82 02 18 4f 0f 69 ce 4f e7 bb 3f 1f ff 55  f....O.i.O..?..U
0484b0  a8 0f 18 42 5a 47 9a f6 31 81 d1 a3 d8 20 ff 55  ...BZG..1.... .U
04b4a0  75 f1 74 f9 07 c9 e7 67 dc ff 55 8d f2 6d ba 72  u.t....g..U..m.r
04eee0  04 92 81 62 f0 05 6c 00 56 42 ff 7f fa ff 55 47  ...b..l.VB....UG
04f480  d9 3f be 15 e3 ff 55 ce d2 56 e0 0b 68 3a 84 e4  .?....U..V..h:..
051920  ce 41 5e dd 77 ec e4 7a b9 61 f4 ee 49 ff 55 d8  .A^.w..z.a..I.U.
0566a0  c8 7b ea bb c1 ff 55 e7 6c b3 0d 86 95 db 93 d2  .{....U.l.......
059ef0  0d fa 7b 8f 03 9b 8a 9b ff 55 ea 27 2a a2 3f 28  ..{......U.'*.?(
05ae40  32 6a f8 9d 65 95 1f 76 be ff 55 d7 9d 6d 54 42  2j..e..v..U..mTB
05b160  ff 55 f9 4a a1 2c 91 bc 7a 62 88 ef 57 18 cb cc  .U.J.,..zb..W...
05cd30  cb 6b ea ee fb fe df 1b 7f f7 7e bb ff 55 ae ca  .k........~..U..
05e0e0  c8 01 46 93 0f ee ef bc ff 55 5f f7 5e 7e 2e e5  ..F......U_.^~..
068410  b5 69 e3 e1 6e 9b 67 02 53 9b 36 1e 72 ff 55 6c  .i..n.g.S.6.r.Ul
06cd80  d0 31 22 66 6c 9e 86 6d ff 55 8a 9c 88 88 36 7b  .1"fl..m.U....6{
086420  e3 a7 39 65 ae 85 7f a0 a5 c7 c1 a8 39 ff 55 2d  ..9e........9.U-
091410  d5 e2 c0 a2 bc 34 a1 82 16 78 1b 12 ff 55 d1 48  .....4...x...U.H
0a58e0  5f 57 1c e3 eb ea 7a 77 d7 ff 2f a7 2e a8 ff 55  _W....zw../....U
0a8670  c3 39 c1 25 da 7f 8f d2 3f 38 28 ff 55 cb d2 16  .9.%....?8(.U...
0b33f0  7f 13 6a f5 f7 59 38 a9 d6 ff 55 04 2e 4a 1a ca  ..j..Y8...U..J..
0b4f20  20 1b e2 0a 3c 0d fa 1a 20 7c cc 8a 68 e9 ff 55   ...<... |..h..U
0b77e0  96 17 a5 56 2a af f1 2d 5f 5e ff 55 a6 ae d5 94  ...V*..-_^.U....
0c26f0  71 27 f6 ff 55 6c e5 77 49 e8 1f d7 8b 54 e6 77  q'..Ul.wI....T.w
0c3280  7a 1d 35 19 34 f4 86 e3 ff 55 88 45 20 a5 83 a6  z.5.4....U.E ...
0c5a20  f4 ab d4 ae 12 ff 55 ce 52 ad 63 02 c0 b1 5d 9c  ......U.R.c...].
0c6040  ce ff 55 a3 7c f5 38 19 8c cd c3 64 38 53 e8 bb  ..U.|.8....d8S..
0cafb0  54 45 5b ff 55 94 d7 1d 16 58 62 7a 7f 56 ce 93  TE[.U....Xbz.V..
0d0820  5d 4e bb ef d7 74 ea 8e 94 fe ff 55 a0 62 f7 55  ]N...t.....U.b.U
0d4c30  c6 9b a6 c3 bb 0d 43 b1 ff 55 3d f1 c7 0f 0d b4  ......C..U=.....
0da440  75 29 da 75 80 ce 99 df ff 55 be d0 cc c6 e5 b9  u).u.....U......
0dea90  ff 55 42 54 14 99 d1 b9 df fa af e3 4d d7 4e e1  .UBT........M.N.
0ea620  1a 67 16 37 1d a2 9d 07 63 39 f9 4d 1f 04 ff 55  .g.7....c9.M...U
0eb3f0  88 5e 76 82 55 b4 fb ff 55 a2 2d e2 df a0 ee a8  .^v.U...U.-.....
0f7b80  df c6 95 4e ff 55 b5 2b 43 6f f9 ea a8 bb b7 53  ...N.U.+Co.....S
0f7f50  5e ff 55 6a 51 3e 88 8f ad 3a de 57 ca af a0 3a  ^.UjQ>...:.W...:
0fde20  da 27 13 ff 26 f9 8e dc e6 ea 67 dc f9 ef ff 55  .'..&.....g....U
1013f0  53 a3 b8 29 77 c4 b1 ff 55 5e 57 72 09 60 8a c0  S..)w...U^Wr.`..
103040  47 82 e2 92 55 63 e7 27 31 c6 cd 7a 43 ff 55 50  G...Uc.'1..zC.UP
106930  73 37 c7 68 0a 30 97 76 ff 55 78 2f 7b f4 a9 65  s7.h.0.v.Ux/{..e
10acd0  16 0e ff 55 e3 d7 e2 cc 81 b0 19 f6 ef e6 87 57  ...U...........W
1103c0  ba bb e7 7a 85 ca 2d a5 44 ff 55 ea e2 7c 81 d0  ...z..-.D.U..|..
117330  ce ff 55 7a f1 8d 78 59 93 6b 95 ac 3c 31 58 27  ..Uz..xY.k..<1X'
11d640  3f cc a3 6e d5 60 87 c9 b0 4b ba ff 55 61 ec 73  ?..n.`...K..Ua.s
11f0b0  ae 6e a8 74 50 9f ff 77 f2 6a f2 ff 55 e9 f5 f1  .n.tP..w.j..U...
120d00  c8 37 b3 8c e6 ff 55 2f 0b d9 5a bf d5 7e fb df  .7....U/..Z..~..
133970  ee 49 97 56 95 ff 55 5e 8a 59 7a d6 53 28 e3 72  .I.V..U^.Yz.S(.r
13e430  73 16 5e 3b 2e 34 b3 ff 55 1e 61 4f ee a4 80 e4  s.^;.4..U.aO....
140790  91 a7 d4 d6 ff 55 1d 1d b5 a9 30 a3 62 e3 20 74  .....U....0.b. t
143e50  6b d6 ff 55 3f c7 cf d1 ff 77 49 7a be ce ef ae  k..U?....wIz....
14dbb0  a7 05 ba 3a ab 1d 67 0a ff 53 81 23 ff 55 54 9b  ...:..g..S.#.UT.
15c260  43 47 c6 da 65 8b 1d 6b 71 6d 36 c8 67 ff 55 65  CG..e..kqm6.g.Ue
15cda0  34 2b bf 1c ff 55 a4 3f 66 6b 9e 02 9e 47 da 8c  4+...U.?fk...G..
161fc0  21 ec e1 fe b4 6a 80 ff 55 9f cc 66 ef 3e 70 fa  !....j..U..f.>p.
169090  99 ff 55 0b e4 46 50 bb ae 09 78 49 ab 06 eb 1f  ..U..FP...xI....
1870c0  0f cf 0d 23 c2 99 8b 1c 07 3c 8e 08 53 b9 ff 55  ...#.....<..S..U
18de30  d2 ff 55 e6 12 f7 f9 e6 97 4a 6b a9 38 06 86 70  ..U......Jk.8..p
195b00  9e 84 ae 8b 63 ff 55 98 ad 61 e5 f9 12 fb ab 17  ....c.U..a......
19a770  78 31 5e 65 66 3b 47 a8 88 ff 55 7b 7d 71 33 a5  x1^ef;G...U{}q3.
19fbb0  4c fb a5 fb ff 55 96 a5 dd 34 a5 50 85 26 bb aa  L....U...4.P.&..
1a07a0  b7 24 ff 55 d1 29 9c 63 da 4c 99 5f dc 76 55 cc  .$.U.).c.L._.vU.
1a69e0  6d a6 72 4f a5 67 14 a5 fe 92 90 a6 ff 55 d7 41  m.rO.g.......U.A
1a8130  e9 ff 55 7b 5f d6 d3 94 72 e9 7b 15 3a de bf ee  ..U{_...r.{.:...
1a8480  6e c8 6d 87 6c 8a ac 2f c2 51 71 ff 55 e6 68 1e  n.m.l../.Qq.U.h.
1aa5d0  4d 66 95 94 95 6c 10 72 10 c7 24 1d 07 ff 55 a8  Mf...l.r..$...U.
1ae290  19 2b 55 5b ff 55 c4 8b 7c 4a de ab ef a2 fc bd  .+U[.U..|J......
1b01e0  5f a2 d0 46 93 10 e1 ac da 40 fd 89 ff 55 bd f6  _..F.....@...U..
1b52d0  89 a4 a9 be ba a9 ff 55 29 d4 dd 5c 4f c4 ec 73  .......U)..\O..s
1b52f0  87 48 ff 55 19 cb af ba 3b 53 8e bf d4 bb 72 d7  .H.U....;S....r.
1b5bb0  f1 23 4b 13 f8 93 8d 48 66 22 ff 55 c1 81 05 2c  .#K....Hf".U...,
1b9680  ff 55 fe e7 64 af 05 f9 f7 57 25 d7 52 29 d4 a7  .U..d....W%.R)..
1c3e10  5f cb b7 f2 0a 37 3a 96 ff 55 53 d2 98 25 61 5b  _....7:..US..%a[
1c9ef0  7b 44 ee 9f 7b a9 21 2b be 35 b5 ef 2e ff 55 25  {D..{.!+.5....U%
1cc550  3d cf 9b ff 55 40 eb df 54 92 37 df f3 e9 da 7c  =...U@..T.7....|
1cfb30  c1 99 da ff 55 6d 31 62 18 ca 53 9c 42 d1 20 cc  ....Um1b..S.B. .
1d5800  f2 ff 55 d8 3a bb ba 59 6e 87 d4 bf 28 ef 7d 5f  ..U.:..Yn...(.}_
1d61b0  ab 76 41 42 87 86 6c c9 78 c0 2e c3 79 ff 55 14  .vAB..l.x...y.U.
1dbb60  c0 6b e2 86 c6 c0 f2 f5 a5 ff 55 05 c0 90 6d 0f  .k........U...m.
1e0110  29 ff 55 07 9d 77 a8 24 fc 69 52 9b e7 f2 40 84  ).U..w.$.iR...@.
1e3ae0  25 0c ef af fe 7c be ff 55 eb ae 92 ad d6 97 22  %....|..U......"
1f65f0  27 5b bf f6 f9 19 b7 6a 3e a5 9b ff 55 94 ca e8  '[.....j>...U...
1fbf60  65 bc fe 51 4a a4 fd d5 07 7f ff 55 53 47 77 c7  e..QJ......USGw.
200b20  22 6c 20 d1 ff 55 21 09 31 79 21 43 7f 08 b9 6e  "l ..U!.1y!C...n
205260  a2 16 85 37 24 c8 d4 8d de 0e d4 ff 55 d1 a7 5a  ...7$.......U..Z
20c020  20 9d 68 94 5a e3 ff 55 4b 5e ce 25 14 21 16 f6   .h.Z..UK^.%.!..
20f060  57 3b af 90 4e 07 7a c4 61 11 60 79 ff 55 f3 68  W;..N.z.a.`y.U.h
213050  b6 a1 3f eb cf da 95 ba af ce bb d2 ff 55 7f 1f  ..?..........U..
219530  44 ff 55 77 bf cf 77 17 85 08 e2 c3 18 78 03 3b  D.Uw..w......x.;
21c040  ff 55 d1 79 f3 7c 30 04 f0 7f bc 19 00 96 15 1f  .U.y.|0.........
21ddb0  c1 ff 55 8a d6 42 5c ac 3c e9 af 05 2a 4e 30 55  ..U..B\.<...*N0U
227740  b4 72 2d ea 8a 4c 00 85 59 7f ff 55 af ac 05 ea  .r-..L..Y..U....
229b90  fd 44 69 46 a1 0d 4a e3 ff 55 70 f4 fa 63 4a e9  .DiF..J..Up..cJ.
230880  4d 9c bf aa 9e 28 34 ff 55 82 16 dc 70 35 1c 34  M....(4.U...p5.4
23cde0  4a 3b ff 55 5e bb af ea e8 be 7d 52 c8 99 6d 57  J;.U^.....}R..mW
24d760  36 b8 56 f7 20 27 ff 55 55 12 5f f0 33 e0 ac fd  6.V. '.UU._.3...
253be0  85 94 2f b5 63 96 ff 55 64 b9 08 07 16 42 61 a1  ../.c..Ud....Ba.
25cf00  ae ee a1 b9 29 2c e0 b8 ff 55 91 ec b1 ac 23 ad  ....),...U....#.
268450  bb 21 ff 55 ab f1 6d c4 ad 57 f6 46 97 ac 65 ff  .!.U..m..W.F..e.
26b6b0  ff 55 ad 27 89 40 e3 04 d6 d9 e4 92 ed f1 fc fd  .U.'.@..........
26c1c0  4c e0 fc b1 ff 55 0f 54 4f 5f 12 52 ad 30 1d 4f  L....U.TO_.R.0.O
2731b0  6c f5 b4 3e 5f a8 ff 55 a3 bb ef 97 6d 72 eb 2f  l..>_..U....mr./
275330  23 09 76 65 1a 48 3f ff 55 55 7c 40 dd 26 a1 6b  #.ve.H?.UU|@.&.k
28d730  8f 59 ff 55 79 d9 07 91 76 8f 23 dc 9f d9 60 12  .Y.Uy...v.#...`.
29ab60  9c 3e a2 ff 55 12 a0 be 06 7f 8f fa 4c 98 ce fa  .>..U.......L...
29df00  9e 97 fa 19 1a b6 6e ad c0 fa 18 ff 55 30 69 ba  ......n.....U0i.
2ad3b0  6d b0 ff 55 a7 5b 24 f7 a7 ec 5f ea 6d fa 44 d5  m..U.[$..._.m.D.
2b0ac0  ed 59 df 62 67 97 61 e9 0c 74 c1 31 5c 68 ff 55  .Y.bg.a..t.1\h.U
2b7a70  ff 55 8d 9c bc 5d 1c 1f c1 85 65 f8 09 58 65 c6  .U...]....e..Xe.
2c3140  3e 8d ff 69 44 1f e3 78 d8 0b ff 55 57 0e 6b 38  >..iD..x...UW.k8
2d26b0  d3 0a 3b ff 55 5f 7c ed 37 bb 9d 25 8b e6 aa b7  ..;.U_|.7..%....
2d9030  b2 12 de 8b 7c dd 4f ee b4 be 4d ff 55 63 e2 57  ....|.O...M.Uc.W
2dead0  be e1 04 41 e9 61 9c 65 c8 a8 a8 a1 03 ff 55 ff  ...A.a.e......U.
2e4270  59 52 3f 53 f4 5c 59 d4 67 46 5c ff 55 43 6b 5f  YR?S.\Y.gF\.UCk_
2e8180  b3 07 86 d9 ff 55 bf d1 c0 c9 84 5a 76 c7 ab b0  .....U.....Zv...
2e8ce0  fb 50 8a dd 55 fe ff 55 da 96 55 a4 cf 5d 8d 6e  .P..U..U..U..].n
2ead80  52 6a 3d f2 4f b1 ff 55 3c 80 94 98 1e b3 8d 9c  Rj=.O..U<.......
2f70d0  bf 75 7a a9 15 92 67 ff 55 2a fb 67 4e ac 1d c3  .uz...g.U*.gN...
2f9130  82 ce ff 55 e7 25 92 52 df 45 fa 12 8e e9 43 54  ...U.%.R.E....CT
2f9fb0  ec e7 cb 95 59 57 4d 51 10 2f ff 55 d9 8f 7a 45  ....YWMQ./.U..zE
2fa390  d1 7c 9c 92 ec 83 d2 79 ba e4 a0 e0 ff 55 90 1d  .|.....y.....U..
2ffa10  ff 75 15 b7 19 49 c9 ff 17 52 ff 55 75 3f fb a3  .u...I...R.Uu?..
301b70  ff 55 a4 75 93 9a 17 f8 b9 38 df 8b 03 34 9f 9c  .U.u.....8...4..
306f00  7e 60 ee aa ff 55 b1 9d d6 ef 9c 2c ad 72 25 9a  ~`...U.....,.r%.
309020  71 93 66 ff 55 6a bc cd d2 68 43 cd 7a c5 da 02  q.f.Uj...hC.z...
30ac70  af 2f 55 b8 96 d4 ee 7a 5b eb 69 cf ff 55 57 33  ./U....z[.i..UW3
313510  80 0e 11 77 ff 55 a0 6b 25 b2 ae 10 94 a2 a3 55  ...w.U.k%......U
318040  d7 41 d9 4a f6 74 48 f7 d5 4c 3e 3e a2 e6 ff 55  .A.J.tH..L>>...U
31a100  a8 65 dc 74 68 bb 6a a0 a9 5f 6e f5 ff 55 77 b7  .e.th.j.._n..Uw.
31a5f0  53 9d c1 f2 cc 1d 7c 01 ff 55 e7 90 d1 b3 ca 88  S.....|..U......
3289d0  33 a7 f1 93 b9 45 d3 a2 71 79 33 68 4b ff 55 54  3....E..qy3hK.UT
32c230  15 80 75 14 d4 95 a0 18 5d 86 63 45 1a b1 ff 55  ..u.....].cE...U
33a0e0  ff 55 a6 44 22 d2 55 de b5 2b 5d 69 df be 42 5f  .U.D".U..+]i..B_
341060  dd 52 ae 7c 84 ff 55 61 da ba 7e fb e7 ed a4 e9  .R.|..Ua..~.....
344c60  85 3a 05 d6 bf df d2 8b 8e bd dc fb aa ff 55 51  .:............UQ
351080  ea 2d f2 40 83 0a 8e a1 f6 cb b7 3a a6 d7 ff 55  .-.@.......:...U
3535c0  ff 55 0c 74 ea 53 ab 99 74 ab 88 05 7e ee f8 95  .U.t.S..t...~...
35f1c0  5b 54 43 e8 ff 55 5e 5e 95 9a 7c a5 a2 21 ee 39  [TC..U^^..|..!.9
36ab90  87 10 4c 58 4c d7 ac cd 38 6d ff 55 b3 e2 f2 73  ..LXL...8m.U...s
36b550  03 9e 19 b4 17 42 58 4a 17 b5 32 b5 ff 55 75 43  .....BXJ..2..UuC
370190  29 9c 97 4b 2e ff 55 2d d2 7e 54 b7 7f 1e d4 37  )..K..U-.~T....7
384240  ff 55 75 74 ce 9f 67 cd b9 e3 ae f9 89 20 44 57  .Uut..g...... DW
389b20  0e 79 f9 07 96 f4 e6 ff 55 80 ea e9 8e b3 6b 2b  .y......U.....k+
389e60  e9 4b 9c e5 9a eb a1 0e ff 55 3b ce cb 5b 63 fb  .K.......U;..[c.
38adc0  bb 8f fa af b4 ff 55 f5 9a bd fd 5f 7c bd dd 02  ......U...._|...
38f670  36 e6 b0 3c eb 45 ff 55 7b 7f 20 ef 33 d9 cc 66  6..<.E.U{. .3..f
3953d0  d7 5e fe 7a a3 6b ff 55 51 3d 6a a0 ab a5 3b a4  .^.z.k.UQ=j...;.
395780  8f 0f 70 b9 99 e3 2f c8 ff 55 27 13 a5 00 e3 c3  ..p.../..U'.....
399b80  ff 55 7d dc 3c da 6e 37 4c 19 c1 2e 42 65 90 b2  .U}.<.n7L...Be..
39c900  b5 e7 d0 8b ff 55 c8 6f 87 7d fb 02 c8 a1 a8 ae  .....U.o.}......
3a50a0  35 dd af 36 ff 55 df 4b 8d 2a 55 f2 ec ea e0 b9  5..6.U.K.*U.....
3a7000  43 52 b8 23 ff 55 82 fd 4b 99 57 56 6d 11 de 0e  CR.#.U..K.WVm...
3a9de0  fd 26 ff 55 95 df 1a 6d ae ab 72 7b 47 ad 59 a9  .&.U...m..r{G.Y.
3ab720  05 16 e9 ff 55 be 83 cf 27 96 3c 1c aa e8 fd db  ....U...'.<.....
3bc400  89 dc 62 ff 55 3e 63 41 78 98 8e e3 fc e4 f9 db  ..b.U>cAx.......
3cc370  f2 16 fe 3b a8 f4 dc 7c fa da e2 2f 6d 24 ff 55  ...;...|.../m$.U
3d6c90  35 ac fb ff 55 9b 77 66 e2 8f c9 6b a7 6b b5 f9  5...U.wf...k.k..
3d9e30  50 28 ca ff 55 8a 2f d5 92 db 39 6c 39 48 ac 07  P(..U./...9l9H..
3e0060  1d b0 7c 20 ff 55 7c 13 08 19 a0 cc 78 3c 3a d1  ..| .U|.....x<:.
3ed170  e6 68 54 ff 55 d6 22 c2 5e 83 5a ba a1 5a ca 92  .hT.U.".^.Z..Z..
3f6260  2b 9b 7f 9b e2 7b 2d 5e 12 83 88 ff 55 80 3a e5  +....{-^....U.:.
3fac90  94 c8 4b 54 49 14 f0 1c 19 29 cb a7 ff 55 66 8a  ..KTI....)...Uf.
3fd500  fc 3c f5 ff 55 b5 66 c5 72 a0 20 be cb 2b 01 88  .<..U.f.r. ..+..
414c90  05 71 ff 55 8f ba 0b 9d 94 5d 69 2d e5 1d 33 e8  .q.U.....]i-..3.
41b700  11 d9 6c 5a af 5a 11 9a 81 69 ff 55 66 f4 39 2d  ..lZ.Z...i.Uf.9-
41e7d0  89 35 4d ec fc 77 24 b1 f5 93 9d 5d ff 55 74 a1  .5M..w$....].Ut.
4268f0  a0 d5 22 60 ff 55 d6 09 6c f1 80 47 06 a1 b9 ff  .."`.U..l..G....


PS C:\tif2jp2\target\release\hexdump-2.1.0>
```
