use failure::Error;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::Path;

pub struct EchoClient {
    stream: UnixStream,
}

impl EchoClient {
    pub fn new(path: &Path) -> Result<EchoClient, Error> {
        let socket = Path::new(path);
        let stream = UnixStream::connect(&socket)?;
        Ok(EchoClient { stream })
    }

    pub fn echo(&mut self, message: &str) -> Result<String, Error> {
        self.stream.write_all(message.as_bytes())?;

        let mut buffer = vec![0; message.len()];
        let _ = self.stream.read(buffer.as_mut_slice())?;
        let output = std::str::from_utf8(&buffer)?;
        Ok(output.to_string())
    }
}
