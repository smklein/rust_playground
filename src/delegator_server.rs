use crate::echo_server::EchoServer;
use crate::identity::Identity;
use failure::Error;
use nix::sys::socket::{getsockopt, sockopt};
use std::collections::HashMap;
use std::fs::remove_file;
use std::io::{Read, Write};
use std::os::unix::io::AsRawFd;
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::Path;
use std::str;
use std::thread;

/// Encapsulates the collection of services which are provided
/// for a client with a particular identity.
///
/// In the current implementation, this merely contains a single
/// Service, but in a more complex scenario, this could be a
/// dynamic list of services granted to a client.
struct ServiceLedger {
    echo_runner: Option<thread::JoinHandle<Result<(), Error>>>,
}

impl Drop for ServiceLedger {
    fn drop(&mut self) {
        let _ = self.echo_runner.take().unwrap().join();
    }
}

/// Represents the accounting for all active services.
///
/// Maps the socket name to the corresponding running server.
struct ServiceMapping {
    mapping: HashMap<String, ServiceLedger>,
}

impl ServiceMapping {
    fn new() -> ServiceMapping {
        ServiceMapping {
            mapping: HashMap::new(),
        }
    }
}

pub struct DelegatorServer {
    listener: UnixListener,
    services: ServiceMapping,
}

impl DelegatorServer {
    pub fn new(path: &Path) -> Result<DelegatorServer, Error> {
        if path.exists() {
            remove_file(&path)?;
        }

        let listener = UnixListener::bind(path)?;
        let services = ServiceMapping::new();
        Ok(DelegatorServer { listener, services })
    }

    fn handle_client(&mut self, mut stream: UnixStream) -> Result<(), Error> {
        println!("DELEGATION SERVER: Delegating client request...");
        let mut buffer = vec![0; 1024];
        let n = stream.read(buffer.as_mut_slice())?;

        let service_name = str::from_utf8(&buffer[..n])?.trim();
        let peer_creds = getsockopt(stream.as_raw_fd(), sockopt::PeerCredentials)?;
        let identity = Identity::new(&peer_creds);

        if service_name != "echo" {
            bail!("DELEGATION SERVER: Unexpected service");
        }
        let socket_name = format!("{}-{}", service_name, identity);

        println!("DELEGATION SERVER: Creating [{}]", socket_name);

        if self.services.mapping.contains_key(&socket_name) {
            bail!("DELEGATION SERVER: Service already exists");
        }

        // In this implementation, we always create an echo server.
        // In a more "realistic" example, different services could be provided,
        // depending on the client's request.
        let echo_server = EchoServer::new(Path::new(&socket_name), identity)?;

        println!("DELEGATION SERVER: Created echo server");
        let service_ledger = ServiceLedger {
            echo_runner: Some(thread::spawn(move || echo_server.run_one())),
        };
        self.services.mapping.insert(socket_name.clone(), service_ledger);

        stream.write_all(socket_name.as_bytes())?;
        Ok(())
    }

    /// Accepts and dispatches a single incoming connection.
    pub fn run_one(&mut self) -> Result<(), Error> {
        // Accept connections and process them, spawning a new thread for each one.
        let (stream, _) = self.listener.accept()?;
        let result = self.handle_client(stream);
        if let Err(err) = &result {
            bail!("DELEGATION SERVER: Failed to delegate: {}", err);
        }
        Ok(())
    }
}
