use libc::{pid_t, uid_t};
use nix::sys::socket::UnixCredentials;
use std::fmt;

/// All information used to validate a client's identity.
///
/// In the current implementation, this is entirely based off of the
/// PeerCredentials on the incoming unix domain socket, but other
/// information could be required to instantiate an identity.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Identity {
    pid: pid_t,
    uid: uid_t,
}

impl Identity {
    pub fn new(creds: &UnixCredentials) -> Identity {
        // This is an arbitrary example; we're taking PID/UID
        // to encapsulate the identity, but we could take other
        // identifiers if we wanted (user-supplied args, GID, etc).
        Identity {
            pid: creds.pid(),
            uid: creds.uid(),
        }
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.pid, self.uid)
    }
}
