# Benchmarks

These results were measured on 2026-06-08 with `tif2jp2` v0.3.0 on Windows using a native release build:

```powershell
$env:RUSTFLAGS="-C target-cpu=native"
cargo build --release --offline
```

The input files were synthetic uncompressed 24-bit RGB TIFFs generated for repeatable local testing. Kakadu was tested against the same inputs because the available `kdu_compress` build could not read compressed TIFF directly.

## Throughput

| File | Input | Tool | Mode | Runs | Average | Min | Max | Output |
| --- | ---: | --- | --- | ---: | ---: | ---: | ---: | ---: |
| `uncompressed_noisy_rgb_2048.tif` | 12 MB | tif2jp2 | default | 3 | 0.350 s | 0.327 s | 0.366 s | 12.94 MB |
| `uncompressed_noisy_rgb_2048.tif` | 12 MB | Kakadu | default | 3 | 1.347 s | 1.338 s | 1.354 s | 13.20 MB |
| `uncompressed_noisy_rgb_4096.tif` | 48 MB | tif2jp2 | default | 3 | 0.840 s | 0.821 s | 0.858 s | 51.78 MB |
| `uncompressed_noisy_rgb_4096.tif` | 48 MB | Kakadu | default | 3 | 5.297 s | 5.274 s | 5.317 s | 52.81 MB |
| `uncompressed_noisy_rgb_2048.tif` | 12 MB | tif2jp2 | single-thread | 3 | 1.498 s | 1.467 s | 1.547 s | 12.94 MB |
| `uncompressed_noisy_rgb_2048.tif` | 12 MB | Kakadu | single-thread | 3 | 1.344 s | 1.329 s | 1.367 s | 13.20 MB |
| `uncompressed_noisy_rgb_4096.tif` | 48 MB | tif2jp2 | single-thread | 3 | 5.300 s | 5.246 s | 5.330 s | 51.78 MB |
| `uncompressed_noisy_rgb_4096.tif` | 48 MB | Kakadu | single-thread | 3 | 5.290 s | 5.244 s | 5.320 s | 52.81 MB |

The default tif2jp2 run uses OpenJPEG threading plus the current Rust-side preprocessing path, so it is much faster than the single-thread run. In the single-thread comparison, tif2jp2 and Kakadu are effectively tied on the 4096 test image, while Kakadu is slightly faster on the smaller input.

Raw CSV files are included next to this document:

- `tif2jp2-kakadu-benchmark-uncompressed.csv`
- `tif2jp2-kakadu-benchmark-single-thread.csv`
- `valid2000-summary.csv`

## Validation

All benchmark JP2 files were decoded successfully with `kdu_expand` and inspected with `tif2jp2 --info`.

`valid2000` was also run with marker scanning enabled. For tif2jp2 outputs it reported `OK=26 WARN=0 FAIL=1`; the only failure was the missing ICC profile, expected because these synthetic TIFFs did not contain one. For Kakadu outputs it reported `OK=21 WARN=3 FAIL=3`, including coding bypass disabled, precinct warnings, TLM not before the first SOT, incomplete tile-part sequence, and missing ICC.

This benchmark is intentionally small and local. Real archival scans can differ substantially depending on image content, bit depth, ICC profile size, tile layout, storage speed, CPU, and OpenJPEG/Kakadu builds.
