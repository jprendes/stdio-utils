use std::fs::File;
use std::io::{read_to_string, stderr, stdin, stdout, Result, Seek, Write};

use stdio_utils::*;

fn seek_to_start(f: &mut File) -> Result<()> {
    f.seek(std::io::SeekFrom::Start(0))?;
    Ok(())
}

fn with_stdout(sink: impl StdioOverride, f: impl FnOnce() -> Result<()>) {
    let _guard = sink.override_stdout().unwrap();
    f().unwrap()
}

fn with_stderr(sink: impl StdioOverride, f: impl FnOnce() -> Result<()>) {
    let _guard = sink.override_stderr().unwrap();
    f().unwrap()
}

fn with_stdin(source: impl StdioOverride, f: impl FnOnce() -> Result<()>) {
    let _guard = source.override_stdin().unwrap();
    f().unwrap()
}

fn with_stdout_str(buf: impl AsRef<str>, f: impl FnOnce() -> Result<()>) {
    let mut sink = tempfile::tempfile().unwrap();
    with_stdout(&mut sink, f);
    seek_to_start(&mut sink).unwrap();
    assert_eq!(read_to_string(sink).unwrap(), buf.as_ref());
}

fn with_stderr_str(buf: impl AsRef<str>, f: impl FnOnce() -> Result<()>) {
    let mut sink = tempfile::tempfile().unwrap();
    with_stderr(&mut sink, f);
    seek_to_start(&mut sink).unwrap();
    assert_eq!(read_to_string(sink).unwrap(), buf.as_ref());
}

fn with_stdin_str(buf: impl AsRef<[u8]>, f: impl FnOnce() -> Result<()>) {
    let mut source = tempfile::tempfile().unwrap();
    source.write_all(buf.as_ref()).unwrap();
    seek_to_start(&mut source).unwrap();
    with_stdin(source, f);
}

#[test]
fn redirect_stdout() {
    let mut stdout = stdout().lock();
    with_stdout_str("hello world!\n", || {
        // use writeln! instad of println! as println! gets captured in tests
        writeln!(stdout, "hello world!")
    })
}

#[test]
fn redirect_stderr() {
    let mut stderr = stderr().lock();
    with_stderr_str("hello world!\n", || {
        // use writeln! instad of println! as println! gets captured in tests
        writeln!(stderr, "hello world!")
    })
}

#[test]
fn redirect_stdin() {
    let mut stdin = stdin().lock();
    with_stdin_str("hello world!\n", || {
        assert_eq!(read_to_string(&mut stdin)?, "hello world!\n");
        Ok(())
    })
}

#[test]
fn write_data_to_null() -> Result<()> {
    let mut f = null()?;
    // This should be writing 1e13 bytes (1 TB) of data.
    // It should be enough for it to fail if the data is not discarded.
    let data = vec![42u8; 10_000_000];
    for _ in 0..100_000 {
        f.write_all(&data)?;
    }
    Ok(())
}

#[test]
fn null_stdout() {
    let mut stdout = stdout().lock();
    let sink = null().unwrap();
    with_stdout(sink, || {
        // when run with "--nocapture" this output should not be seen
        writeln!(stdout, "hello world!")?;
        // unfortunately there's not much we can assert here
        Ok(())
    })
}

#[test]
fn null_stdin() {
    let mut stdin = stdin().lock();
    let source = null().unwrap();
    with_stdin(source, || {
        let data = read_to_string(&mut stdin).unwrap_or_default();
        assert_eq!(data, "");
        Ok(())
    })
}

#[test]
fn stdin_guard() {
    let mut stdin = stdin().lock();
    with_stdin_str("hello world", || {
        with_stdin_str("bye world", || {
            assert_eq!(read_to_string(&mut stdin)?, "bye world");
            Ok(())
        });
        assert_eq!(read_to_string(&mut stdin)?, "hello world");
        Ok(())
    });
}
