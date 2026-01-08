# ffms2info

A Rust CLI tool that indexes video files using [FFMS2](https://github.com/FFMS/ffms2) (via [ffms2-rs](https://github.com/rust-av/ffms2-rs)) and prints video metadata (resolution, color properties, framerate, etc.).

## Prerequisites

- [Rust](https://www.rust-lang.org/) (latest stable/nightly)
- FFMS2 libraries/headers
- FFmpeg libraries

## Build Instructions

### Standard Build

Run the standard Cargo build command. This will use the default system linker and libraries.

```bash
cargo build --release
```

### Static Build (Windows)

This project is configured with a custom alias to easily create static builds using `vcpkg` dependencies for FFmpeg and static CRT.

**Requirements:**
1.  vcpkg installed and integrated.
2.  `ffmpeg` installed via vcpkg: `vcpkg install ffmpeg[avcodec,avdevice,avfilter,avformat,swresample,swscale,zlib,bzip2,core,dav1d,gpl,version3,lzma,openssl,xml2]:x64-windows-static` (or your target arch).
3.  `FFMS_LIB_DIR` environment variable pointing to the directory containing the **static** `ffms2.lib`.
4.  `FFMS_INCLUDE_DIR` environment variable pointing to the directory containing `ffms2.h`.

Note: You need to compile your own static `ffms2.lib`.

**Command:**

```bash
cargo b-static --release
```

or running directly:

```bash
cargo r-static --release -- <video_file>
```

## Usage

Pass the path to a video file as an argument. The tool will index the file (creating a `.ffindex` file next to it if one doesn't exist) and print the video information.

```bash
ffms2info <path/to/video>
```

Output Example:
```text
VidInf {
    width: 1920,
    height: 1080,
    fps_num: 60,
    fps_den: 1,
    frames: 391,
    color_primaries: Some(
        6,
    ),
    transfer_characteristics: Some(
        6,
    ),
    matrix_coefficients: Some(
        6,
    ),
    is_10bit: false,
    ...
}
```

## Credits

Special thanks to the [xav](https://github.com/emrakyz/xav) project, as the code in this tool is derived from it.
