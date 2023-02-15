# stickeroid

Python script to convert stuff into WhatsApp stickers

## Dependencies

- `FFmpeg`
  - [Download](https://ffmpeg.org/download.html)
  - [Tested with](https://www.gyan.dev/ffmpeg/builds/ffmpeg-git-full.7z)
- `ImageMagick`
  - [Download](https://imagemagick.org/script/download.php)
  - [Mirrors](https://imagemagick.org/script/mirror.php)
  - [Tested with](https://mirror.dogado.de/imagemagick/binaries/ImageMagick-7.1.0-61-portable-Q8-x64.zip)
- `WebP Utilities`
  - [How to compile](https://developers.google.com/speed/webp/docs/compiling)
  - [Download](https://developers.google.com/speed/webp/download)
  - [Tested with](https://storage.googleapis.com/downloads.webmproject.org/releases/webp/libwebp-1.3.0-windows-x64.zip)
  - [Overview](https://developers.google.com/speed/webp/docs/using)
  - Useful tools: [vwebp](https://developers.google.com/speed/webp/docs/vwebp), [webpinfo](https://developers.google.com/speed/webp/docs/webpinfo), [webpmux](https://developers.google.com/speed/webp/docs/webpmux), [img2webp](https://developers.google.com/speed/webp/docs/img2webp)

Make sure `ffmpeg`, `magick`, `anim_dump` and `img2webp` are accessible through
the `PATH` environment variable.

## Add stickers to WhatsApp

See [github.com/WhatsApp/stickers](https://github.com/WhatsApp/stickers)

## Change WebP duration

See [github.com/ace9934/webp-duration-changer](https://ace9934.github.io/webp-duration-changer/) ([source](https://github.com/ace9934/webp-duration-changer/blob/master/js/webpScript.js))

## Environment variables

- `ANIM_DUMP_BIN`: `C:\[...]\anim_dump.exe`
- `WEBP_INFO_BIN`: `C:\[...]\webpinfo.exe`
- `FFMPEG_BIN`: `C:\[...]\ffmpeg.exe`
- `MAGICK_BIN`: `C:\[...]\magick.exe`
- `IMG2WEBP_BIN`: `C:\[...]\img2webp.exe`
- `VWEBP_BIN`: `C:\[...]\vwebp.exe`

## Tokio feature flags

[tokio/#feature-flags](https://docs.rs/tokio/latest/tokio/#feature-flags)
