use std::io::{self, ErrorKind, Read, Seek, SeekFrom};

use crate::bindings::Windows::Win32::Storage::StructuredStorage::{
    IStream, STREAM_SEEK_CUR, STREAM_SEEK_END, STREAM_SEEK_SET,
};

pub struct WinStream {
    stream: IStream,
}

impl WinStream {
    pub fn from(stream: IStream) -> Self {
        Self { stream }
    }
}

impl Read for WinStream {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, std::io::Error> {
        let mut bytes_read = 0u32;
        unsafe {
            self.stream
                .Read(buf.as_mut_ptr() as _, buf.len() as u32, &mut bytes_read)
        }
        .map_err(|err| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("IStream::Read failed: {}", err.code().0),
            )
        })?;
        Ok(bytes_read as usize)
    }
}

impl Seek for WinStream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let (stream_seek, amount) = match pos {
            SeekFrom::Current(a) => (STREAM_SEEK_CUR, a),
            SeekFrom::End(a) => (STREAM_SEEK_END, a),
            SeekFrom::Start(a) => (STREAM_SEEK_SET, a as i64),
        };

        let seeked = unsafe { self.stream.Seek(amount, stream_seek) }.map_err(|err| {
            std::io::Error::new(
                ErrorKind::Other,
                format!("IStream::Seek failed: {}", err.code().0),
            )
        })?;

        Ok(seeked)
    }
}
