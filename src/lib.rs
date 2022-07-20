#![allow(deprecated)]

#[cfg(test)]
#[macro_use]
extern crate claim;

use std::io::BufRead;
use std::io::Error;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Result;
use std::slice;

/// A composable reader to read until a `0x1A` byte (commonly known as `CTRL-Z` or the "substitute
/// character") is encountered.
/// 
/// This `struct` is a wrapper around another type that implements [`Read`] or [`BufRead`]. Calls
/// to the methods of those traits will be forwarded to the interior type until a `0x1A` byte is
/// read, at which point reading will cease.
/// 
/// # Example
/// Here is an example of a `ReadToCrtlZ` wrapped around a [`&[u8]`], which implements [`Read`].
/// 
/// ```
/// use ctrl_z::ReadToCtrlZ;
/// use std::io::Read;
/// 
/// let mut reader = ReadToCtrlZ::new(b"foo\x1a".as_slice());
/// let mut output = String::new();
/// 
/// // Reading omits the final `0x1A` byte.
/// assert!(reader.read_to_string(&mut output).is_ok());
/// assert_eq!(output, "foo");
/// ```
pub struct ReadToCtrlZ<R> {
    inner: R,
    terminated: bool,
}

impl<R> ReadToCtrlZ<R> {
    pub fn new(inner: R) -> Self {
        ReadToCtrlZ {
            inner: inner,
            terminated: false,
        }
    }
}

impl<R> Read for ReadToCtrlZ<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        if self.terminated {
            return Ok(0);
        }

        let n = try!(self.inner.read(buf));
        for i in 0..n {
            if *try!(buf.get(i).ok_or_else(|| {
                Error::new(ErrorKind::Other, "buffer smaller than amount of bytes read")
            })) == b'\x1a'
            {
                self.terminated = true;
                return Ok(i);
            }
        }
        Ok(n)
    }
}

impl<R> BufRead for ReadToCtrlZ<R>
where
    R: BufRead,
{
    fn fill_buf(&mut self) -> Result<&[u8]> {
        if self.terminated {
            return Ok(&[]);
        }

        let buf = try!(self.inner.fill_buf());
        for i in 0..buf.len() {
            // SAFETY: `i` is guaranteed to be a valid index into `buf`.
            if *unsafe { buf.get_unchecked(i) } == b'\x1a' {
                if i == 0 {
                    self.terminated = true;
                }
                // SAFETY: The range `..i` is guaranteed to be a valid index into `buf`.
                return Ok(unsafe { slice::from_raw_parts(buf.as_ptr(), i) });
            }
        }
        Ok(buf)
    }

    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::BufRead;
    use std::io::ErrorKind;
    use std::io::Read;
    use std::io::Result;

    #[test]
    fn read_exclude_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo\x1a" as &[u8]).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_no_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo" as &[u8]).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_stop_at_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo\x1abar" as &[u8]).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_after_ctrl_z() {
        let mut output = String::new();
        let mut reader = ReadToCtrlZ::new(b"foo\x1abar" as &[u8]);

        assert_ok_eq!(reader.read_to_string(&mut output), 3);
        assert_eq!(output, "foo");

        // This indicates the reader has reached EOF.
        assert_ok_eq!(reader.read_to_string(&mut output), 0);
    }

    struct BadReader;

    impl Read for BadReader {
        fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
            Ok(buf.len() + 1)
        }
    }

    #[test]
    fn read_with_bad_inner() {
        let error = assert_err!(ReadToCtrlZ::new(BadReader).read(&mut []));

        assert_eq!(error.kind(), ErrorKind::Other);
        assert_eq!(
            error.to_string(),
            "buffer smaller than amount of bytes read"
        )
    }

    #[test]
    fn buf_read_exclude_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo\x1a" as &[u8]).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_no_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo" as &[u8]).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_stop_at_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo\x1abar" as &[u8]).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_after_ctrl_z() {
        let mut reader = ReadToCtrlZ::new(b"foo\x1abar" as &[u8]);

        assert_ok_eq!(reader.fill_buf(), b"foo");
        reader.consume(3);

        // The reader should return nothing else, since the EOF `0x1A` was reached.
        assert_ok_eq!(reader.fill_buf(), b"");
    }

    #[test]
    fn buf_read_after_starting_ctrl_z() {
        let mut reader = ReadToCtrlZ::new(b"\x1abar" as &[u8]);

        // Should stop before "bar".
        assert_ok_eq!(reader.fill_buf(), b"");

        // The reader should return nothing else, since the EOF `0x1A` was reached.
        assert_ok_eq!(reader.fill_buf(), b"");
    }
}
