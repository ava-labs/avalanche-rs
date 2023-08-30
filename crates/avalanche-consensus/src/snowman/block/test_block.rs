use crate::snowman::block::Block;
use avalanche_types::{
    choices::{decidable::Decidable, status::Status, test_decidable::TestDecidable},
    errors::Result,
    ids::Id,
    verify::Verifiable,
};
use bytes::Bytes;

/// Implements test block.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/consensus/snowman#TestBlock>
#[derive(Clone, Debug)]
pub struct TestBlock {
    decidable: TestDecidable,
    parent_id: Id,
    verify_result: Result<()>,
    bytes: Bytes,
    height: u64,
    timestamp: u64,
}

impl TestBlock {
    pub fn new(
        decidable: TestDecidable,
        parent_id: Id,
        verify_result: Result<()>,
        bytes: Bytes,
        height: u64,
        timestamp: u64,
    ) -> Self {
        Self {
            decidable,
            parent_id,
            verify_result,
            bytes,
            height,
            timestamp,
        }
    }

    /// Returns a new instantiation of "Block" trait
    /// which itself implies "Decidable" trait.
    /// Must import "Decidable" trait for use.
    pub fn new_trait(
        decidable: TestDecidable,
        parent_id: Id,
        verify_result: Result<()>,
        bytes: Bytes,
        height: u64,
        timestamp: u64,
    ) -> impl Block {
        Self {
            decidable,
            parent_id,
            verify_result,
            bytes,
            height,
            timestamp,
        }
    }
}

impl Verifiable for TestBlock {
    fn verify(&self) -> Result<()> {
        self.verify_result.clone()
    }
}

impl Decidable for TestBlock {
    fn id(&self) -> Id {
        self.decidable.id()
    }

    fn status(&self) -> Status {
        self.decidable.status()
    }

    fn accept(&mut self) -> Result<()> {
        self.decidable.accept()
    }

    fn reject(&mut self) -> Result<()> {
        self.decidable.reject()
    }
}

impl Block for TestBlock {
    fn parent(&self) -> Id {
        self.parent_id
    }

    fn bytes(&self) -> Bytes {
        self.bytes.clone()
    }

    fn height(&self) -> u64 {
        self.height
    }

    fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

/// RUST_LOG=debug cargo test --package avalanche-consensus --lib -- snowman::block::test_block::test_block --exact --show-output
#[test]
fn test_block() {
    use avalanche_types::errors::Error;

    let id = Id::from_slice(&[4, 5, 6]);
    let decidable = TestDecidable::new(id, Status::Processing);
    assert_eq!(decidable.id(), id);
    assert_eq!(decidable.status(), Status::Processing);

    let parent_id = Id::from_slice(&[1, 2, 3]);
    let height = 1_u64;
    let timestamp = 2_u64;
    let mut block = TestBlock::new_trait(
        decidable,
        parent_id,
        Ok(()),
        Bytes::new(),
        height,
        timestamp,
    );
    assert_eq!(block.id(), id);
    assert_eq!(block.status(), Status::Processing);
    assert_eq!(block.parent(), parent_id);
    assert!(block.verify().is_ok());
    assert_eq!(block.bytes().len(), 0);
    assert_eq!(block.height(), height);
    assert_eq!(block.timestamp(), timestamp);

    assert!(block.accept().is_ok());
    assert_eq!(block.status(), Status::Accepted);

    // fail accept
    let mut decidable = TestDecidable::new(id, Status::Processing);
    decidable.set_accept_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let mut block = TestBlock::new_trait(
        decidable,
        parent_id,
        Ok(()),
        Bytes::new(),
        height,
        timestamp,
    );
    assert_eq!(block.status(), Status::Processing);
    assert!(block.accept().is_err());
    assert_eq!(block.status(), Status::Processing);

    // fail reject
    let mut decidable = TestDecidable::new(id, Status::Processing);
    decidable.set_reject_result(Err(Error::Other {
        message: "test error".to_string(),
        retryable: false,
    }));
    let mut block = TestBlock::new_trait(
        decidable,
        parent_id,
        Ok(()),
        Bytes::new(),
        height,
        timestamp,
    );
    assert_eq!(block.status(), Status::Processing);
    assert!(block.reject().is_err());
    assert_eq!(block.status(), Status::Processing);
}
