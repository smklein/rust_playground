#![allow(dead_code)]

use failure::Error;
use nix::sys::socket::{getsockopt, sockopt};
use std::fs::remove_file;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

fn handle_client(mut stream: UnixStream) -> Result<(), Error> {
    println!("SERVER: Handling client");

    let mut buffer = vec![0; 1024];
    let n = stream.read(buffer.as_mut_slice())?;
    let peer_creds = getsockopt(stream.as_raw_fd(), sockopt::PeerCredentials)?;

    println!("SERVER: Peer credentials: {:#?}", peer_creds);

    stream.write_all(&buffer[..n])?;
    Ok(())
}

pub struct EchoServer {
    listener: UnixListener,
}

impl EchoServer {
    pub fn new(path: &Path) -> Result<EchoServer, Error> {
        if path.exists() {
            remove_file(&path)?;
        }

        let listener = UnixListener::bind(path)?;
        Ok(EchoServer { listener })
    }

    pub fn run(&self) -> Result<(), Error> {
        // Accept connections and process them, spawning a new thread for each one.
        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    thread::spawn(|| handle_client(stream));
                }
                Err(err) => {
                    eprintln!("Server refused to handle client: {}", err);
                    break;
                }
            }
        }
        Ok(())
    }
}
