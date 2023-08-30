//! Implements a database exposed by RPC.
pub mod client;
pub mod server;

use std::{collections::HashMap, io};

use lazy_static::lazy_static;

use crate::proto::pb;

lazy_static! {
    static ref ERROR_TO_ERROR_CODE: HashMap<&'static str, i32> = {
        let mut m = HashMap::new();
        m.insert("database closed", pb::rpcdb::Error::Closed.into());
        m.insert("not found", pb::rpcdb::Error::NotFound.into());
        m
    };
}

pub fn error_to_error_code(msg: &str) -> io::Result<i32> {
    match ERROR_TO_ERROR_CODE.get(msg) {
        None => Ok(0),
        Some(code) => Ok(*code),
    }
}

#[test]
fn database_errors() {
    assert_eq!(
        *ERROR_TO_ERROR_CODE.get("database closed").unwrap(),
        pb::rpcdb::Error::Closed as i32
    );
    assert_eq!(
        *ERROR_TO_ERROR_CODE.get("not found").unwrap(),
        pb::rpcdb::Error::NotFound as i32
    );
    assert!(ERROR_TO_ERROR_CODE.get("ohh snap!").is_none());
}
