#[macro_use] extern crate failure;

use common::SOCKET_PATH;
use delegator_client::DelegatorClient;
use delegator_server::DelegatorServer;
use echo_client::EchoClient;
use failure::Error;
use std::thread;
use std::path::Path;

mod common;
mod delegator_client;
mod delegator_server;
mod echo_client;
mod echo_server;
mod identity;

fn main() -> Result<(), Error> {
    let path = Path::new(SOCKET_PATH);

    let mut server = DelegatorServer::new(path)?;
    let server_thread = thread::spawn(move || server.run_one());

    let mut client = DelegatorClient::new(path)?;
    let echo_socket = client.acquire("echo")?;

    let mut echo_client = EchoClient::new(&echo_socket)?;
    println!("ECHO CLIENT: Sending message...");
    let message = echo_client.echo("hello")?;
    println!("ECHO CLIENT: Read [{}]", message);

    let _ = server_thread.join();

    Ok(())
}
