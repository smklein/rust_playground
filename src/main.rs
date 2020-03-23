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

// TODO: Figure out, PEERCREDS also work for remote sockets?
//
// Client --> Server
//      Asks for new connection.
//      Asks for it to be accessible to CRED (PID/UID/GID).
//
// Server
//      Creates new UnixListener bound to a new domain socket.
//      Saves the requested CRED, to be checked.
//
// Server --> Client
//      Sure, here's your stuff @ path.
//
// Client may connect via path
//      On connection, peer credentials validated against CRED.
//      XXX is this possible? Would be server-side gRPC...
//          (haven't seen any APIs to do this :/ )

fn main() -> Result<(), Error> {
    let path = Path::new(SOCKET_PATH);

    let mut server = DelegatorServer::new(path)?;
    thread::spawn(move || server.run());


    loop {
        let mut client = DelegatorClient::new(path)?;
        let echo_socket = client.acquire("echo")?;

        let mut echo_client = EchoClient::new(&echo_socket)?;
        let message = echo_client.echo("hello")?;
        println!("CLIENT: Read [{}]", message);
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
}
