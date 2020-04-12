use failure::Error;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};

pub struct DelegatorClient {
    stream: UnixStream,
}

impl DelegatorClient {
    pub fn new(path: &Path) -> Result<DelegatorClient, Error> {
        let stream = UnixStream::connect(path)?;
        Ok(DelegatorClient { stream })
    }

    /// Sends a request to the server, asking for a new path
    pub fn acquire(&mut self, message: &str) -> Result<PathBuf, Error> {
        println!("DELEGATION CLIENT: Requesting access to {}", message);
        self.stream.write_all(message.as_bytes())?;

        let mut buffer = vec![0; 4096];
        let _ = self.stream.read(buffer.as_mut_slice())?;
        let output = std::str::from_utf8(&buffer)?.trim_matches(char::from(0));

        println!("DELEGATION CLIENT: Received [{}]", output);

        Ok(PathBuf::from(output))
    }
}
