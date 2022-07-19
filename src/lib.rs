use std::io::{BufRead, Error, ErrorKind, Read, Result};

pub struct ReadToCtrlZ<R> {
    inner: R,
    terminated: bool,
}

impl<R> ReadToCtrlZ<R> {
    pub fn new(inner: R) -> Self {
        Self {
            inner,
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

        let n = self.inner.read(buf)?;
        for i in 0..n {
            if *buf.get(i).ok_or_else(|| {
                Error::new(ErrorKind::Other, "buffer smaller than amount of bytes read")
            })? == b'\x1a'
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

        let buf = self.inner.fill_buf()?;
        for i in 0..buf.len() {
            // SAFETY: `i` is guaranteed to be a valid index into `buf`.
            if *unsafe { buf.get_unchecked(i) } == b'\x1a' {
                if i == 0 {
                    self.terminated = true;
                }
                // SAFETY: The range `..i` is guaranteed to be a valid index into `buf`.
                return Ok(unsafe { buf.get_unchecked(..i) });
            }
        }
        return Ok(buf);
    }

    fn consume(&mut self, amount: usize) {
        self.inner.consume(amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::{assert_err, assert_ok_eq};
    use std::io::{ErrorKind, Read, Result};

    #[test]
    fn read_exclude_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo\x1a".as_slice()).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_no_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo".as_slice()).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_stop_at_ctrl_z() {
        let mut output = String::new();

        assert_ok_eq!(
            ReadToCtrlZ::new(b"foo\x1abar".as_slice()).read_to_string(&mut output),
            3
        );
        assert_eq!(output, "foo");
    }

    #[test]
    fn read_after_ctrl_z() {
        let mut output = String::new();
        let mut reader = ReadToCtrlZ::new(b"foo\x1abar".as_slice());

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
            error.into_inner().unwrap().to_string(),
            "buffer smaller than amount of bytes read"
        );
    }

    #[test]
    fn buf_read_exclude_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo\x1a".as_slice()).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_no_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo".as_slice()).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_stop_at_ctrl_z() {
        assert_ok_eq!(ReadToCtrlZ::new(b"foo\x1abar".as_slice()).fill_buf(), b"foo");
    }

    #[test]
    fn buf_read_after_ctrl_z() {
        let mut reader = ReadToCtrlZ::new(b"foo\x1abar".as_slice());

        assert_ok_eq!(reader.fill_buf(), b"foo");
        reader.consume(3);

        // The reader should return nothing else, since the EOF `0x1A` was reached.
        assert_ok_eq!(reader.fill_buf(), b"");
    }
}
