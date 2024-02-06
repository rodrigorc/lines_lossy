/// Like `BufRead::lines` but lossy.
///
/// This crate provices an extension trait [`LinesLossyExt`], blanketly implemented for any type
/// that implements `BufRead`. It has a single member function [`LinesLossyExt::lines_lossy`], that
/// works just like `BufRead::lines` except that in presence of invalid UTF-8 data, it returns a
/// lossy representation of that string, with `�` replacing invalid characters, just like
/// `String::from_utf8_lossy` does.
///
/// # When to use `lines_lossy`
/// You'll want to use `lines_lossy` to read the lines of a UTF-8 encoded text file that you
/// suspect may have non UTF-8 byte sequences embedded, but that you want to process the correct
/// text anyway.
///
/// The obvious example is a log file: it should be UTF-8 encoded, but it lives for a long time,
/// it may be corrupted, it may dump text read from untrusted sources... You don't want to
/// discard lines just because they are not UTF-8 valid, just read what you can. Any other
/// human-readable text file might also fit this use-case.
///
/// Other example would be files using a computer language that uses ASCII only caracters, but that
/// may have human readable comments using an uspecified encoding. Since that encoding may be
/// anything you will not be able to succesfully decode them, but they are comments and maybe can
/// be discarded. This use-case would include languages such as assembly, g-code, dxf, nmea...
///
/// # When not to use `lines_lossy`
/// You should not use this crate to read text files that should always have correct UTF-8 lines,
/// such as configuration files or API replies. If you have such a file with a non UTF-8 byte
/// sequence, then the responsible thing to do is probably to discard the whole file; or the whole
/// line if the format is line oriented.
use std::io::{BufRead, Result};

pub trait LinesLossyExt: Sized {
    fn lines_lossy(self) -> LinesLossy<Self>;
}

impl<T: BufRead> LinesLossyExt for T {
    fn lines_lossy(self) -> LinesLossy<Self> {
        LinesLossy { buf: self }
    }
}

#[derive(Debug)]
pub struct LinesLossy<B> {
    buf: B,
}

impl<B: BufRead> Iterator for LinesLossy<B> {
    type Item = Result<String>;

    fn next(&mut self) -> Option<Result<String>> {
        let mut buf = Vec::new();
        match self.buf.read_until(b'\n', &mut buf) {
            Ok(0) => None,
            Ok(_n) => {
                if buf.last() == Some(&b'\n') {
                    buf.pop();
                    if buf.last() == Some(&b'\r') {
                        buf.pop();
                    }
                }
                let text = match String::from_utf8(buf) {
                    Ok(s) => s,
                    Err(e) => String::from_utf8_lossy(e.as_bytes()).into_owned(),
                };
                Some(Ok(text))
            }
            Err(e) => Some(Err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    fn test_case(input: &[u8], output: &[&str]) {
        let rdr = Cursor::new(input);
        let lines1 = rdr.lines_lossy().collect::<Result<Vec<String>>>().unwrap();
        let lines2 = output
            .iter()
            .map(|s| s.to_string())
            .collect::<Vec<String>>();
        assert_eq!(lines1, lines2);
    }

    #[test]
    fn copied_from_std_io() {
        test_case(b"12\r", &["12\r"]);
        test_case(b"12\r\n\n", &["12", ""]);
    }
    #[test]
    fn basic() {
        test_case(b"", &[]);
        test_case(b"hello\nworld", &["hello", "world"]);
        test_case(b"hello\r\nworld", &["hello", "world"]);
        test_case(b"hello\nworld\n", &["hello", "world"]);
        test_case(b"hello\r\nworld\r\n", &["hello", "world"]);
    }
    #[test]
    fn lossy() {
        test_case(b"what\xaas\nup", &["what\u{fffd}s", "up"]);
        test_case(
            b"\xaa\xbb\xcc\r\n\xee \xff",
            &["\u{fffd}\u{fffd}\u{fffd}", "\u{fffd} \u{fffd}"],
        );
    }
}
