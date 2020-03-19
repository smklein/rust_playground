#![allow(unused_variables)]

extern crate failure;

use common::SOCKET_PATH;
use failure::Error;
use std::fs::remove_file;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

mod common;

fn handle_client(mut stream: UnixStream) -> Result<(), Error> {
    println!("SERVER: Handling client");

    let mut buffer = vec![0; 1024];
    let n = stream.read(buffer.as_mut_slice())?;
    let m = stream.write(&buffer[..n])?;
    Ok(())
}

struct Server {
    listener: UnixListener,
}

impl Server {
    fn new() -> Result<Server, Error> {
        let socket_path = Path::new(SOCKET_PATH);

        if socket_path.exists() {
            remove_file(&socket_path)?;
        }

        let listener = UnixListener::bind(socket_path)?;

        Ok(Server { listener })
    }

    fn run(&self) -> Result<(), Error> {
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

fn client() -> Result<(), Error> {
    let socket = Path::new(SOCKET_PATH);
    let mut stream = UnixStream::connect(&socket)?;

    // Send message
    let message = "hello";
    stream.write_all(message.as_bytes())?;

    // Read response.
    let mut buffer = vec![0; message.len()];
    let n = stream.read(buffer.as_mut_slice())?;
    let output = std::str::from_utf8(&buffer)?;
    println!("CLIENT: Read [{}]", output);

    Ok(())
}

fn main() -> Result<(), Error> {
    let server = Server::new()?;
    thread::spawn(move || server.run());

    loop {
        if let Err(err) = client() {
            eprintln!("CLIENT: Failure: {}", err);
        }
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
