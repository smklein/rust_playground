use crate::identity::Identity;
use failure::Error;
use nix::sys::socket::{getsockopt, sockopt};
use std::fs::remove_file;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};

fn authenticate(stream: &UnixStream, expected: Identity) -> Result<(), Error> {
    let peer_creds = getsockopt(stream.as_raw_fd(), sockopt::PeerCredentials)?;
    println!("ECHO SERVER: Peer credentials: {:#?}", peer_creds);
    let identity = Identity::new(&peer_creds);
    if identity != expected {
        bail!("Invalid identity");
    }
    Ok(())
}

fn echo(mut stream: UnixStream) -> Result<(), Error> {
    println!("ECHO SERVER: Echo time!");
    let mut buffer = vec![0; 1024];
    let n = stream.read(buffer.as_mut_slice())?;
    stream.write_all(&buffer[..n])?;
    Ok(())
}

pub struct EchoServer {
    listener: UnixListener,
    identity: Identity,
    path: PathBuf,
}

// TODO(smklein): Could we refactor the identity-parsing bits to be generic?
impl EchoServer {
    /// Creates a new echo server for the specified identity.
    ///
    /// Only incoming clients which can pass the identity check
    /// are served.
    pub fn new(path: &Path, identity: Identity) -> Result<EchoServer, Error> {
        if path.exists() {
            remove_file(&path)?;
        }

        let listener = UnixListener::bind(path)?;
        Ok(EchoServer { listener, identity, path: path.to_path_buf() })
    }

    /// Accepts and dispatches a single incoming connection.
    pub fn run_one(&self) -> Result<(), Error> {
        if let Ok((stream, _)) = self.listener.accept() {
            println!("ECHO SERVER: Accepted stream");
            authenticate(&stream, self.identity)?;
            echo(stream)?;
        }
        Ok(())
    }
}

impl Drop for EchoServer {
    fn drop(&mut self) {
        let _  = remove_file(&self.path);
    }
}
