#![cfg(windows)]
use crate::Error;
use std::io::Error as IoError;
use std::os::windows::ffi::OsStrExt;
use std::path::PathBuf;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::windows::named_pipe::NamedPipeClient;
use winapi::um::fileapi::*;
use winapi::um::winbase::*;
use winapi::um::winnt::*;

/// Wrapper around a tokio [`NamedPipeClient`]
pub struct NamedPipe {
    io: NamedPipeClient,
}

impl NamedPipe {
    pub async fn connect(path: PathBuf) -> Result<Self, Error> {
        let win_path = path
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();

        let handle = unsafe {
            CreateFileW(
                win_path.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                0,
                std::ptr::null_mut(),
                OPEN_EXISTING,
                FILE_FLAG_OVERLAPPED,
                std::ptr::null_mut(),
            )
        };
        if handle.is_null() {
            let err = IoError::last_os_error();
            return Err(Error::Connect {
                endpoint: path,
                source: Box::new(err),
            });
        }

        let io = unsafe { NamedPipeClient::from_raw_handle(handle)? };
        Ok(Self { io })
    }
}

impl AsyncRead for NamedPipe {
    fn poll_read(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut ReadBuf,
    ) -> Poll<Result<(), IoError>> {
        AsyncRead::poll_read(Pin::new(&mut self.get_mut().io), ctx, buf)
    }
}

impl AsyncWrite for NamedPipe {
    fn poll_write(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &[u8],
    ) -> Poll<Result<usize, IoError>> {
        AsyncWrite::poll_write(Pin::new(&mut self.get_mut().io), ctx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Result<(), IoError>> {
        AsyncWrite::poll_flush(Pin::new(&mut self.get_mut().io), ctx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Result<(), IoError>> {
        AsyncWrite::poll_shutdown(Pin::new(&mut self.get_mut().io), ctx)
    }
}

impl crate::ReadWriteStream for NamedPipe {}
