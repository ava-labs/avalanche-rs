pub mod grpc;

use std::{
    io::{Error, Result},
    net::{SocketAddr, UdpSocket},
};

/// Returns a localhost address with next available port.
pub fn new_socket_addr() -> SocketAddr {
    UdpSocket::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
}

/// Persists first \[`io::Error`\] collected.
#[derive(Debug)]
pub struct Errors {
    err: Option<Error>,
}

impl Default for Errors {
    fn default() -> Self {
        Self::new()
    }
}

impl Errors {
    pub fn new() -> Self {
        Self { err: None }
    }

    /// Persists the error if no error currently exists.
    pub fn add(&mut self, error: &Error) {
        if self.err.is_none() {
            self.err = Some(Error::new(error.kind(), error.to_string()))
        }
    }

    /// Returns an io::Error if collected.
    pub fn err(&self) -> Result<()> {
        if let Some(e) = &self.err {
            return Err(Error::new(e.kind(), e.to_string()));
        }
        Ok(())
    }

    /// Returns true an error has been collected.
    pub fn is_some(&self) -> bool {
        self.err.is_some()
    }
}
