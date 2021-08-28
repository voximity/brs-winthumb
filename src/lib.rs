#![crate_type = "dylib"]

use std::io::Cursor;

use brickadia::read::SaveReader;
use image::GenericImageView;
use intercom::{prelude::*, raw::HRESULT};

mod registry;
mod winstream;
use winstream::WinStream;

mod bindings;

use bindings::{
    Windows::Win32::Foundation::WINCODEC_ERR_WRONGSTATE,
    Windows::Win32::Graphics::Gdi::{CreateBitmap, DeleteObject, HBITMAP},
    Windows::Win32::Storage::StructuredStorage::IStream,
    Windows::Win32::UI::Shell::{WTSAT_ARGB, WTS_ALPHATYPE},
};

com_library! {
    on_load=on_load,
    on_register=registry::register_provider,
    on_unregister=registry::unregister_provider,
    class ThumbnailProvider
}

const MAX_DIMENSION: u32 = 200;

/// Called when the DLL is loaded.
///
/// Sets up logging to the Cargo.toml directory for debug purposes.
fn on_load() {
    #[cfg(debug_assertions)]
    {
        // Set up logging to the project directory.
        use log::LevelFilter;
        simple_logging::log_to_file(
            &format!("{}\\debug.log", env!("CARGO_MANIFEST_DIR")),
            LevelFilter::Trace,
        )
        .unwrap();
    }
}

#[com_class(
    // A unique identifier solely for jxl-winthumb
    clsid = "5f85282f-0cb4-4e7f-a7f1-09fa662b0af0",
    IInitializeWithStream,
    IThumbnailProvider
)]
#[derive(Default)]
struct ThumbnailProvider {
    stream: Option<WinStream>,
    bitmap: Option<HBITMAP>,
}

impl IInitializeWithStream for ThumbnailProvider {
    fn initialize(&mut self, stream: ComIStream, _mode: u32) -> ComResult<()> {
        self.stream = Some(WinStream::from(stream.0.clone()));
        std::mem::forget(stream); // Prevent dropping, will happen later

        Ok(())
    }
}

// TODO: Use encoder channel order option when available. Not yet as of 0.3.0
fn reorder(vec: &mut Vec<u8>) {
    assert_eq!(vec.len() % 4, 0);
    for i in 0..vec.len() / 4 {
        // Windows expects BGRA (ARGB in reverse order) while JXL emits RGBA
        let r = vec[i * 4];
        let b = vec[i * 4 + 2];
        vec[i * 4] = b;
        vec[i * 4 + 2] = r;
    }
}

impl IThumbnailProvider for ThumbnailProvider {
    fn get_thumbnail(&mut self, _cx: u32) -> ComResult<(ComHbitmap, ComWtsAlphatype)> {
        let stream = match &self.stream {
            Some(_) => self.stream.take().unwrap(),
            None => return Err(HRESULT::new(WINCODEC_ERR_WRONGSTATE.0 as i32).into()),
        };

        let mut reader = SaveReader::new(stream).expect("Failed to create BRS reader");

        let _header1 = reader.read_header1().expect("Failed to read header 1");
        let _header2 = reader.read_header2().expect("Failed to read header 2");
        let preview = reader.read_preview().expect("Failed to read preview");

        let img = image::io::Reader::new(Cursor::new(
            preview.into_bytes().expect("Save has no preview"),
        ))
        .with_guessed_format()
        .expect("Failed to guess format")
        .decode()
        .expect("Failed to decode preview");

        let (cw, ch) = (img.width(), img.height());
        let (w, h) = if cw >= ch && cw > MAX_DIMENSION {
            let nw = MAX_DIMENSION;
            let nh = (nw as f32 / cw as f32 * ch as f32) as u32;
            (nw, nh)
        } else if ch > cw && ch > MAX_DIMENSION {
            let nh = MAX_DIMENSION;
            let nw = (nh as f32 / ch as f32 * cw as f32) as u32;
            (nw, nh)
        } else {
            (cw, ch)
        };

        let resized = image::imageops::resize(&img, w, h, image::imageops::Triangle);

        let mut bytes = resized.to_vec();
        reorder(&mut bytes);

        let bitmap = unsafe {
            CreateBitmap(
                w as i32,
                h as i32,
                1,
                32,
                bytes.as_ptr() as *const _,
            )
        };
        self.bitmap = Some(bitmap);

        Ok((ComHbitmap(bitmap), ComWtsAlphatype(WTSAT_ARGB)))
    }
}

impl Drop for ThumbnailProvider {
    fn drop(&mut self) {
        // Delete the bitmap once it's not needed anymore.
        if let Some(bitmap) = self.bitmap {
            unsafe { DeleteObject(bitmap) };
        }
    }
}

// New types for deriving Intercom traits.

#[derive(intercom::ForeignType, intercom::ExternType, intercom::ExternOutput)]
#[allow(non_camel_case_types)]
#[repr(transparent)]
struct ComHbitmap(HBITMAP);

#[derive(
    intercom::ForeignType, intercom::ExternType, intercom::ExternOutput, intercom::ExternInput,
)]
#[repr(transparent)]
struct ComIStream(IStream);

#[derive(
    intercom::ForeignType, intercom::ExternType, intercom::ExternOutput, intercom::ExternInput,
)]
#[allow(non_camel_case_types)]
#[repr(transparent)]
struct ComWtsAlphatype(WTS_ALPHATYPE);

// COM interface definitions.

#[com_interface(com_iid = "e357fccd-a995-4576-b01f-234630154e96")]
trait IThumbnailProvider {
    fn get_thumbnail(&mut self, cx: u32) -> ComResult<(ComHbitmap, ComWtsAlphatype)>;
}

#[com_interface(com_iid = "b824b49d-22ac-4161-ac8a-9916e8fa3f7f")]
trait IInitializeWithStream {
    fn initialize(&mut self, stream: ComIStream, mode: u32) -> ComResult<()>;
}
