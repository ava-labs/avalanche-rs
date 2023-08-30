//! Custom database errors and helpers.
use std::io;

use tonic::Status;

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/database#ErrClosed>
#[derive(Copy, Clone, Debug)]
pub enum Error {
    DatabaseClosed = 1, // 0 is reserved for grpc unspecified.
    NotFound,
    HeightIndexedVMNotImplemented,
    IndexIncomplete,
    StateSyncableVMNotImplemented,
}

impl Error {
    pub fn as_str(&self) -> &'static str {
        match *self {
            Error::DatabaseClosed => "database closed",
            Error::NotFound => "not found",
            Error::HeightIndexedVMNotImplemented => {
                "vm does not implement HeightIndexedChainVM interface"
            }
            Error::IndexIncomplete => "query failed because height index is incomplete",
            Error::StateSyncableVMNotImplemented => {
                "vm does not implement StateSyncableVM interface"
            }
        }
    }

    pub fn to_i32(&self) -> i32 {
        match self {
            Error::DatabaseClosed => 1,
            Error::NotFound => 2,
            Error::HeightIndexedVMNotImplemented => 3,
            Error::IndexIncomplete => 4,
            Error::StateSyncableVMNotImplemented => 5,
        }
    }

    /// Returns coresponding io::Error.
    pub fn to_err(&self) -> io::Error {
        match *self {
            Error::DatabaseClosed => {
                io::Error::new(io::ErrorKind::Other, Error::DatabaseClosed.as_str())
            }
            Error::NotFound => io::Error::new(io::ErrorKind::NotFound, Error::NotFound.as_str()),
            Error::HeightIndexedVMNotImplemented => io::Error::new(
                io::ErrorKind::Other,
                Error::HeightIndexedVMNotImplemented.as_str(),
            ),
            Error::IndexIncomplete => {
                io::Error::new(io::ErrorKind::Other, Error::IndexIncomplete.as_str())
            }
            Error::StateSyncableVMNotImplemented => io::Error::new(
                io::ErrorKind::Other,
                Error::StateSyncableVMNotImplemented.as_str(),
            ),
        }
    }
}

pub fn from_i32(err: i32) -> io::Result<()> {
    match err {
        0 => Ok(()),
        1 => Err(Error::DatabaseClosed.to_err()),
        2 => Err(Error::NotFound.to_err()),
        3 => Err(Error::HeightIndexedVMNotImplemented.to_err()),
        4 => Err(Error::IndexIncomplete.to_err()),
        5 => Err(Error::StateSyncableVMNotImplemented.to_err()),
        _ => panic!("invalid error type"),
    }
}

/// Accepts an error and returns a corruption error if the original error is not "database closed"
/// or "not found".
pub fn is_corruptible(error: &io::Error) -> bool {
    match error {
        e if e.kind() == io::ErrorKind::NotFound => false,
        e if e.to_string() == Error::DatabaseClosed.as_str() => false,
        _ => true,
    }
}

/// Returns true if the io::Error is ErrorKind::NotFound and contains a string "not found".
pub fn is_not_found(error: &io::Error) -> bool {
    error.kind() == io::ErrorKind::NotFound && error.to_string() == Error::NotFound.as_str()
}

/// Returns an io::Error with ErrorKind::Other from a string.
pub fn from_string(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::Other, message)
}

/// Returns a common database error from a tonic Status or .
pub fn from_status(status: Status) -> io::Error {
    match status.message() {
        m if m.contains("database closed") => Error::DatabaseClosed.to_err(),
        m if m.contains("not found") => Error::NotFound.to_err(),
        _ => io::Error::new(io::ErrorKind::Other, status.message()),
    }
}
