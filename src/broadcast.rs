use std::io;
use std::io::{Error, Write};

use shout::ShoutConn;

pub struct ShoutConnWriter(pub ShoutConn);

impl Write for ShoutConnWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.0.send(buf) {
            Ok(..) => {
                self.0.sync();
                return Ok(buf.len());
            }
            Err(..) => Err(Error::other("Error writing to Shoutcast Connection")),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.0.sync();
        Ok(())
    }
}
