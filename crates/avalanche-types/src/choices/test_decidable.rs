//! Test for decidable consensus operations.
use crate::{
    choices::{decidable::Decidable, status::Status},
    errors::{Error, Result},
    ids::Id,
};

#[derive(Clone, Debug)]
pub struct TestDecidable {
    pub id: Id,

    /// "Status" enum uses String
    /// so cannot implement/derive "Copy" to use "Cell"
    /// ref. <https://stackoverflow.com/questions/38215753/how-do-i-implement-copy-and-clone-for-a-type-that-contains-a-string-or-any-type>
    ///
    /// Use "Box" instead to overwrite.
    pub status: Box<Status>,

    pub accept_result: Result<()>,
    pub reject_result: Result<()>,
}

impl Default for TestDecidable {
    fn default() -> Self {
        Self::default()
    }
}

impl TestDecidable {
    pub fn default() -> Self {
        Self {
            id: Id::empty(),

            status: Box::new(Status::Processing),

            accept_result: Ok(()),
            reject_result: Ok(()),
        }
    }
}

impl TestDecidable {
    pub fn new(id: Id, status: Status) -> Self {
        Self {
            id,
            status: Box::new(status),
            accept_result: Ok(()),
            reject_result: Ok(()),
        }
    }

    pub fn set_accept_result(&mut self, rs: Result<()>) {
        self.accept_result = rs;
    }

    pub fn set_reject_result(&mut self, rs: Result<()>) {
        self.reject_result = rs;
    }

    pub fn create_decidable(
        id: Id,
        status: Status,
        accept_result: Result<()>,
        reject_result: Result<()>,
    ) -> impl Decidable {
        Self {
            id,
            status: Box::new(status),
            accept_result,
            reject_result,
        }
    }
}

impl Decidable for TestDecidable {
    fn id(&self) -> Id {
        self.id
    }

    fn status(&self) -> Status {
        Status::from(self.status.as_str())
    }

    fn accept(&mut self) -> Result<()> {
        let status = self.status.as_ref();
        if matches!(status, Status::Unknown(_) | Status::Rejected) {
            return Err(Error::Other {
                message: format!(
                    "invalid state transaction from {} to {}",
                    status,
                    Status::Accepted
                ),
                retryable: false,
            });
        }
        if self.accept_result.is_ok() {
            self.status = Box::new(Status::Accepted);
        }

        self.accept_result.clone()
    }

    fn reject(&mut self) -> Result<()> {
        let status = self.status.as_ref();
        if matches!(status, Status::Unknown(_) | Status::Accepted) {
            return Err(Error::Other {
                message: format!(
                    "invalid state transaction from {} to {}",
                    status,
                    Status::Rejected
                ),
                retryable: false,
            });
        }
        if self.reject_result.is_ok() {
            self.status = Box::new(Status::Rejected);
        }

        self.reject_result.clone()
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- decidable::test_decidable::test_decidable --exact --show-output
#[test]
fn test_decidable() {
    let id = Id::from_slice(&[1, 2, 3]);

    let mut decidable = TestDecidable::create_decidable(id, Status::Processing, Ok(()), Ok(()));
    assert_eq!(decidable.id(), id);
    assert_eq!(decidable.status(), Status::Processing);
    assert!(decidable.accept().is_ok());
    assert_eq!(decidable.status(), Status::Accepted);

    let mut decidable = TestDecidable::create_decidable(id, Status::Processing, Ok(()), Ok(()));
    assert_eq!(decidable.id(), id);
    assert_eq!(decidable.status(), Status::Processing);
    assert!(decidable.reject().is_ok());
    assert_eq!(decidable.status(), Status::Rejected);

    let mut decidable = TestDecidable::new(id, Status::Processing);
    decidable.set_accept_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    assert_eq!(decidable.id(), id);
    assert_eq!(decidable.status(), Status::Processing);
    assert!(decidable.accept().is_err());
    assert_eq!(decidable.status(), Status::Processing);

    let mut decidable = TestDecidable::create_decidable(
        id,
        Status::Processing,
        Ok(()),
        Err(Error::Other {
            message: "test error".to_string(),
            retryable: false,
        }),
    );
    assert_eq!(decidable.id(), id);
    assert_eq!(decidable.status(), Status::Processing);
    assert!(decidable.reject().is_err());
    assert_eq!(decidable.status(), Status::Processing);
}
