#![allow(unused_variables)]

use common::SOCKET_PATH;
use std::fs::remove_file;
use std::io::Write;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

mod common;

fn handle_client(stream: UnixStream) {
    println!("handle client");
    // ...
}

fn server() -> std::io::Result<()> {
    let socket_path = Path::new(SOCKET_PATH);

    if socket_path.exists() {
        remove_file(&socket_path)?;
    }

    let listener = UnixListener::bind(socket_path)?;

    // accept connections and process them, spawning a new thread for each one
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                /* connection succeeded */
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                /* connection failed */
                break;
            }
        }
    }
    Ok(())
}

fn client() -> std::io::Result<()> {
    let socket = Path::new(SOCKET_PATH);

    let message = "hello";

    // Connect to socket
    let mut stream = UnixStream::connect(&socket)?;

    // Send message
    match stream.write(message.as_bytes()) {
        Err(err) => Err(std::io::Error::new(std::io::ErrorKind::Other, "Bad write")),
        Ok(_) => Ok(()),
    }
}

fn main() -> std::io::Result<()> {
    thread::spawn(server);
    client()
}
