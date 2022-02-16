use std::{
    fs::{File, OpenOptions},
    io::Write,
};

pub(crate) struct FileAppender {
    inner: File,
}

impl FileAppender {
    pub fn new(filename: impl Into<String>) -> Self {
        let file = OpenOptions::new()
            .append(true)
            .create(true)
            .open(filename.into())
            .expect("Could not open file!");
        Self { inner: file }
    }
}

impl Write for FileAppender {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
