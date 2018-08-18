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
    fn read(&mut self, buf:&mut [u8]) -> Result<usize> {
        self.0.read(buf)
    }
}
impl<R: Read, W: Write> Write for ReadWrite<R, W> {
    fn write(&mut self, buf:&[u8]) -> Result<usize> {
        self.1.write(buf)
    }
    fn flush(&mut self,) -> Result<()> {
        self.1.flush()
    }
}

#[cfg(all(feature="tokio-io",feature="futures"))]
mod async {
    use std::io::{Read, Result, Write,Error};
    extern crate tokio_io;
    extern crate futures;
    use self::tokio_io::{AsyncRead,AsyncWrite};

    /// Combined async reader and writer.
    /// Note that this struct is only present in `readwrite` if "tokio" Cargo feature is enabled.
    ///
    /// Waiting for [RFC 1787](https://github.com/rust-lang/rfcs/pull/1787) to make it better.
    pub struct ReadWriteAsync<R: AsyncRead, W: AsyncWrite>(pub R, pub W);
    
    impl<R: AsyncRead, W: AsyncWrite> From<(R, W)> for ReadWriteAsync<R, W> {
        fn from((r, w): (R, W)) -> Self {
            ReadWriteAsync(r, w)
        }
    }
    impl<R: AsyncRead, W: AsyncWrite> ReadWriteAsync<R, W> {
        /// Bundle separate async reader and writer into a combined pseudo-socket
        pub fn new(r: R, w: W) -> Self {
            ReadWriteAsync(r, w)
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
    
    impl<R: AsyncRead, W: AsyncWrite> Read for ReadWriteAsync<R, W> {
        fn read(&mut self, buf:&mut [u8]) -> Result<usize> {
            self.0.read(buf)
        }
    }
    impl<R: AsyncRead, W: AsyncWrite> AsyncRead for ReadWriteAsync<R, W> {}
    impl<R: AsyncRead, W: AsyncWrite> Write for ReadWriteAsync<R, W> {
        fn write(&mut self, buf:&[u8]) -> Result<usize> {
            self.1.write(buf)
        }
        fn flush(&mut self,) -> Result<()> {
            self.1.flush()
        }
    }
    impl<R: AsyncRead, W: AsyncWrite> AsyncWrite for ReadWriteAsync<R, W> {
        fn shutdown(&mut self) -> futures::Poll<(), Error> {
            self.1.shutdown()
        }
    }
}
#[cfg(all(feature="tokio-io",feature="futures"))]
pub use async::ReadWriteAsync;
