use std::io::{Error, ErrorKind, Read, Result};

pub struct ReadToCtrlZ<R> {
    inner: R,
}

impl<R> ReadToCtrlZ<R> {
    pub fn new(inner: R) -> Self {
        Self { inner }
    }
}

impl<R> Read for ReadToCtrlZ<R>
where
    R: Read,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let n = self.inner.read(buf)?;
        for i in 0..n {
            if *buf.get(i).ok_or_else(|| {
                Error::new(ErrorKind::Other, "buffer smaller than amount of bytes read")
            })? == b'\x1a'
            {
                return Ok(i);
            }
        }
        Ok(n)
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
}
