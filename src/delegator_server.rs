#![allow(dead_code)]

use crate::echo_server::EchoServer;
use failure::Error;
use libc::{pid_t, uid_t};
use nix::sys::socket::{getsockopt, sockopt, UnixCredentials};
use std::collections::HashMap;
use std::fs::remove_file;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::thread;

/// All information used to validate a client's identity.
///
/// In the current implementation, this is entirely based off of the
/// PeerCredentials on the incoming unix domain socket, but other
/// information could be required to instantiate an identity.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Identity {
    pid: pid_t,
    uid: uid_t,
}

impl Identity {
    fn new(creds: &UnixCredentials) -> Identity {
        // This is an arbitrary example; we're taking PID/UID
        // to encapsulate the identity, but we could take other
        // identifiers if we wanted (user-supplied args, GID, etc).
        Identity {
            pid: creds.pid(),
            uid: creds.uid(),
        }
    }
}

/// Encapsulates the collection of services which are provided
/// for a client with a particular identity.
///
/// In the current implementation, this merely contains a single
/// Service, but in a more complex scenario, this could be a
/// dynamic list of services granted to a client.
struct ServiceLedger {
    server: EchoServer,
}

struct DelegationState {
    // XXX XXX XXX
    //
    // Probably need to wrap in arc/mutex to share with multiple threads.
    //
    // INPUT: PID/UID/GID.
    // Need to:
    //  1) Create echo server
    //  2) Pass PID/UID/GID (maybe "identity" structure?)
    //  3) *Echo server* impl needs to check validation structure on access
    //      - Could refactor to be generic of echo impl
    //  4) Accounting??? How do we track created echo servers?
    //      - "client" (PID/UID/GID) --> EchoServer thread handle?
    //      - Can also check "identity" against a capacity
    //  5) Less threads, make async
    services: HashMap<Identity, ServiceLedger>,
}

fn handle_client(mut stream: UnixStream) -> Result<(), Error> {
    println!("SERVER: Handling client");
    let mut buffer = vec![0; 1024];
    let n = stream.read(buffer.as_mut_slice())?;
    let peer_creds = getsockopt(stream.as_raw_fd(), sockopt::PeerCredentials)?;
    println!("SERVER: Peer credentials: {:#?}", peer_creds);

    stream.write_all(&buffer[..n])?;
    Ok(())
}

pub struct DelegatorServer {
    listener: UnixListener,
}

impl DelegatorServer {
    pub fn new(path: &Path) -> Result<DelegatorServer, Error> {
        if path.exists() {
            remove_file(&path)?;
        }

        let listener = UnixListener::bind(path)?;
        Ok(DelegatorServer { listener })
    }

    pub fn run(&mut self) -> Result<(), Error> {
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
