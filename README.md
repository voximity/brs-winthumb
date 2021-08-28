# brs-winthumb

A Brickadia save (*.brs) thumbnail handler for Windows File Explorer.

### An important note

This is a fork of [saschanaz](https://github.com/saschanaz)'s [jxl-winthumb](https://github.com/saschanaz/jxl-winthumb) tool
to add a thumbnail handler for JXL files. Windows interop with Rust is hard!

## How to install

1. Clone and build using `cargo build --release`.
2. Open a terminal window as administrator.
3. Move to `target/release`.
4. `regsvr32 brs_winthumb.dll`, or to uninstall, `regsvr32 /u brs_winthumb.dll`.

## Why not Windows Imaging Component?

It was useful before Windows 10 era when Windows Photo Gallery existed, but not anymore as Microsoft now uses its own undocumented way to add system codecs for UWP apps.

## Helpful resources

* [Thumbnail Handlers](https://docs.microsoft.com/en-us/windows/win32/shell/thumbnail-providers)
* [IThumbnailProvider::GetThumbnail method (thumbcache.h)](https://docs.microsoft.com/en-us/windows/win32/api/thumbcache/nf-thumbcache-ithumbnailprovider-getthumbnail)
* [IInitializeWithStream::Initialize method (propsys.h)](https://docs.microsoft.com/en-us/windows/win32/api/propsys/nf-propsys-iinitializewithstream-initialize)

## Inspired by

* [Intercom thumbnail provider example](https://github.com/Rantanen/intercom/tree/88d6a3c0470150805740b75ed23ec15131ec7469/samples/thumbnail_provider)
* [FlifWICCodec](https://github.com/peirick/FlifWICCodec/)
