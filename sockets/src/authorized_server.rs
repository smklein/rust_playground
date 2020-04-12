use crate::identity::Identity;

trait AuthorizedServer {
    fn authenticate(stream: &UnixStream, expected: Identity) -> Result<(), Error> {
        let peer_creds = getsockopt(stream.as_raw_fd(), sockopt::PeerCredentials)?;
        let identity = Identity::new(&peer_creds);
        if identity != expected {
            bail!("Invalid identity");
        }
        Ok(())
    }
}

