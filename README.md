# lines-lossy

This crate provices an extension trait [`LinesLossyExt`], blanketly implemented for any type
that implements `BufRead`. It has a single member function [`LinesLossyExt::lines_lossy`], that
works just like `BufRead::lines` except that in presence of invalid UTF-8 data, it returns a
lossy representation of that string, with `�` replacing invalid characters, just like
`String::from_utf8_lossy` does.

# When to use `lines_lossy`
You'll want to use `lines_lossy` to read the lines of a UTF-8 encoded text file that you
suspect may have non UTF-8 byte sequences embedded, but that you want to process the correct
text anyway.

The obvious example is a log file: it should be UTF-8 encoded, but it lives for a long time,
it may be corrupted, it may dump text read from untrusted sources... You don't want to
discard lines just because they are not UTF-8 valid, just read what you can. Any other
human-readable text file might also fit this use-case.

Other example would be files using a computer language that uses ASCII only caracters, but that
may have human readable comments using an uspecified encoding. Since that encoding may be
anything you will not be able to succesfully decode them, but they are comments and maybe can
be discarded. This use-case would include languages such as assembly, g-code, dxf, nmea...

# When not to use `lines_lossy`
You should not use this crate to read text files that should always have correct UTF-8 lines,
such as configuration files or API replies. If you have such a file with a non UTF-8 byte
sequence, then the responsible thing to do is probably to discard the whole file; or the whole
line if the format is line oriented.
