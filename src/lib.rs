#![deny(missing_docs)]
#![forbid(unsafe_code)]

//! Given two things, one of which implements `std::io::Read` and other implements `std::io::Write`, make a single socket-like object which implmenets `Read + Write`. Note that you can't write to it while waiting for data to come from read part.
//!
//! There is also AsyncRead / AsyncWrite analogue, see `ReadWriteAsync` struct.

use std::io::{Read, Result, Write};

/// Combined reader and writer
pub struct ReadWrite<R: Read, W: Write>(pub R, pub W);

impl<R: Read, W: Write> From<(R, W)> for ReadWrite<R, W> {
    fn from((r, w): (R, W)) -> Self {
        ReadWrite(r, w)
    }
}
impl<R: Read, W: Write> ReadWrite<R, W> {
    /// Bundle separate reader and writer into a combined pseudo-socket
    pub fn new(r: R, w: W) -> Self {
        ReadWrite(r, w)
    }
    /// Borrow inner objects
    pub fn borrow(&self) -> (&R, &W) {
        (&self.0, &self.1)
    }
    /// Borrow the reader
    pub fn borrow_read(&self) -> &R {
        &self.0
    }
    /// Borrow the writer
    pub fn borrow_write(&self) -> &W {
        &self.1
    }
    /// Mutably borrow inner objects
    pub fn borrow_mut(&mut self) -> (&mut R, &mut W) {
        (&mut self.0, &mut self.1)
    }
    /// Mutably borrow the reader
    pub fn borrow_mut_read(&mut self) -> &mut R {
        &mut self.0
    }
    /// Mutably borrow the writer
    pub fn borrow_mut_write(&mut self) -> &mut W {
        &mut self.1
    }
    /// Convert ReadWrite back into individual reader and writer pair
    pub fn into_inner(self) -> (R, W) {
        (self.0, self.1)
    }
    /// Convert ReadWrite back into the reader, dropping the writer
    pub fn into_reader(self) -> R {
        self.0
    }
    /// Convert ReadWrite back into the writer, dropping the reader
    pub fn into_writer(self) -> W {
        self.1
    }
}

impl<R: Read, W: Write> Read for ReadWrite<R, W> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }

    fn read_vectored(&mut self, bufs: &mut [std::io::IoSliceMut<'_>]) -> Result<usize> {
        self.0.read_vectored(bufs)
    }
}
impl<R: Read, W: Write> Write for ReadWrite<R, W> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        self.1.write(buf)
    }
    fn flush(&mut self) -> Result<()> {
        self.1.flush()
    }

    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> Result<usize> {
        self.1.write_vectored(bufs)
    }
}

#[cfg(all(feature = "tokio"))]
mod tokio {
    use tokio_dep::io::{AsyncRead, AsyncWrite};

    use std::pin::Pin;

    pin_project_lite::pin_project! {
        /// Combined async reader and writer, `tokio 1` version.
        /// Note that this struct is only present in `readwrite` if "tokio" Cargo feature is enabled.
        pub struct ReadWriteTokio<R, W> {
            #[pin]
            r: R,
            #[pin]
            w: W,
        }
    }

    impl<R: AsyncRead, W: AsyncWrite> From<(R, W)> for ReadWriteTokio<R, W> {
        fn from((r, w): (R, W)) -> Self {
            ReadWriteTokio { r, w }
        }
    }
    impl<R: AsyncRead, W: AsyncWrite> ReadWriteTokio<R, W> {
        /// Bundle separate async reader and writer into a combined pseudo-socket
        pub fn new(r: R, w: W) -> Self {
            ReadWriteTokio { r, w }
        }
        /// Borrow inner objects
        pub fn borrow(&self) -> (&R, &W) {
            (&self.r, &self.w)
        }
        /// Borrow the reader
        pub fn borrow_read(&self) -> &R {
            &self.r
        }
        /// Borrow the writer
        pub fn borrow_write(&self) -> &W {
            &self.w
        }
        /// Mutably borrow inner objects
        pub fn borrow_mut(&mut self) -> (&mut R, &mut W) {
            (&mut self.r, &mut self.w)
        }
        /// Mutably borrow the reader
        pub fn borrow_mut_read(&mut self) -> &mut R {
            &mut self.r
        }
        /// Mutably borrow the writer
        pub fn borrow_mut_write(&mut self) -> &mut W {
            &mut self.w
        }
        /// Convert ReadWrite back into individual reader and writer pair
        pub fn into_inner(self) -> (R, W) {
            (self.r, self.w)
        }
        /// Convert ReadWrite back into the reader, dropping the writer
        pub fn into_reader(self) -> R {
            self.r
        }
        /// Convert ReadWrite back into the writer, dropping the reader
        pub fn into_writer(self) -> W {
            self.w
        }

        /// Borrow pinned reader and writer
        pub fn borrow_pin(self: Pin<&mut Self>) -> (Pin<&mut R>, Pin<&mut W>) {
            let p = self.project();
            (p.r, p.w)
        }
        /// Borrow pinned reader
        pub fn borrow_pin_read(self: Pin<&mut Self>) -> Pin<&mut R> {
            self.project().r
        }
        /// Borrow pinned writer
        pub fn borrow_pin_write(self: Pin<&mut Self>) -> Pin<&mut W> {
            self.project().w
        }
    }

    impl<R: AsyncRead, W> AsyncRead for ReadWriteTokio<R, W> {
        fn poll_read(
            self: std::pin::Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut tokio_dep::io::ReadBuf<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            AsyncRead::poll_read(self.project().r, cx, buf)
        }
    }

    impl<R, W: AsyncWrite> AsyncWrite for ReadWriteTokio<R, W> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            self.project().w.poll_write(cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            self.project().w.poll_flush(cx)
        }

        fn poll_shutdown(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Result<(), std::io::Error>> {
            self.project().w.poll_shutdown(cx)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> std::task::Poll<Result<usize, std::io::Error>> {
            self.project().w.poll_write_vectored(cx, bufs)
        }

        fn is_write_vectored(&self) -> bool {
            self.w.is_write_vectored()
        }
    }
}
#[cfg(all(feature = "tokio"))]
pub use tokio::ReadWriteTokio;

#[cfg(all(feature = "asyncstd"))]
mod asyncstd {
    use futures::io::{AsyncRead, AsyncWrite};

    use std::pin::Pin;

    pin_project_lite::pin_project! {
        /// Combined async reader and writer, `futures 0.3` version.
        /// Note that this struct is only present in `readwrite` if "asyncstd" Cargo feature is enabled.
        pub struct ReadWriteAsyncstd<R, W> {
            #[pin]
            r: R,
            #[pin]
            w: W,
        }
    }

    impl<R: AsyncRead, W: AsyncWrite> From<(R, W)> for ReadWriteAsyncstd<R, W> {
        fn from((r, w): (R, W)) -> Self {
            ReadWriteAsyncstd { r, w }
        }
    }
    impl<R: AsyncRead, W: AsyncWrite> ReadWriteAsyncstd<R, W> {
        /// Bundle separate async reader and writer into a combined pseudo-socket
        pub fn new(r: R, w: W) -> Self {
            ReadWriteAsyncstd { r, w }
        }
        /// Borrow inner objects
        pub fn borrow(&self) -> (&R, &W) {
            (&self.r, &self.w)
        }
        /// Borrow the reader
        pub fn borrow_read(&self) -> &R {
            &self.r
        }
        /// Borrow the writer
        pub fn borrow_write(&self) -> &W {
            &self.w
        }
        /// Mutably borrow inner objects
        pub fn borrow_mut(&mut self) -> (&mut R, &mut W) {
            (&mut self.r, &mut self.w)
        }
        /// Mutably borrow the reader
        pub fn borrow_mut_read(&mut self) -> &mut R {
            &mut self.r
        }
        /// Mutably borrow the writer
        pub fn borrow_mut_write(&mut self) -> &mut W {
            &mut self.w
        }
        /// Convert ReadWrite back into individual reader and writer pair
        pub fn into_inner(self) -> (R, W) {
            (self.r, self.w)
        }
        /// Convert ReadWrite back into the reader, dropping the writer
        pub fn into_reader(self) -> R {
            self.r
        }
        /// Convert ReadWrite back into the writer, dropping the reader
        pub fn into_writer(self) -> W {
            self.w
        }

        /// Borrow pinned reader and writer
        pub fn borrow_pin(self: Pin<&mut Self>) -> (Pin<&mut R>, Pin<&mut W>) {
            let p = self.project();
            (p.r, p.w)
        }
        /// Borrow pinned reader
        pub fn borrow_pin_read(self: Pin<&mut Self>) -> Pin<&mut R> {
            self.project().r
        }
        /// Borrow pinned writer
        pub fn borrow_pin_write(self: Pin<&mut Self>) -> Pin<&mut W> {
            self.project().w
        }
    }

    impl<R: AsyncRead, W> AsyncRead for ReadWriteAsyncstd<R, W> {
        fn poll_read(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &mut [u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.project().r.poll_read(cx, buf)
        }

        fn poll_read_vectored(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &mut [std::io::IoSliceMut<'_>],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.project().r.poll_read_vectored(cx, bufs)
        }
    }

    impl<R, W: AsyncWrite> AsyncWrite for ReadWriteAsyncstd<R, W> {
        fn poll_write(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            buf: &[u8],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.project().w.poll_write(cx, buf)
        }

        fn poll_flush(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            self.project().w.poll_flush(cx)
        }

        fn poll_close(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
        ) -> std::task::Poll<std::io::Result<()>> {
            self.project().w.poll_close(cx)
        }

        fn poll_write_vectored(
            self: Pin<&mut Self>,
            cx: &mut std::task::Context<'_>,
            bufs: &[std::io::IoSlice<'_>],
        ) -> std::task::Poll<std::io::Result<usize>> {
            self.project().w.poll_write_vectored(cx, bufs)
        }
    }
}
#[cfg(all(feature = "asyncstd"))]
pub use asyncstd::ReadWriteAsyncstd;
