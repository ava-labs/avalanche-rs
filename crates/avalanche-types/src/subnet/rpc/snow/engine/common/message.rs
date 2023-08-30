//! Consensus engine message type.
use num_derive::{FromPrimitive, ToPrimitive};

/// Message is an enum of the message types that vms can send to consensus.
///
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#Message>
#[derive(FromPrimitive, ToPrimitive, PartialEq, Eq, Debug)]
#[repr(u32)]
pub enum Message {
    /// Notifies a consensus engine that its VM has pending transactions
    /// (i.e. it would like to add a new block/vertex to consensus)
    PendingTxs = 1, // 0 is reserved for grpc unspecified.

    /// Notifies the state syncer engine that the VM has finishing
    /// syncing the requested state summary.
    StateSyncDone,

    /// StopVertex notifies a consensus that it has a pending stop vertex.
    StopVertex,
}

impl Message {
    /// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#Message.String>
    pub fn as_str(&self) -> &str {
        match self {
            Message::PendingTxs => "Pending Transactions",
            Message::StateSyncDone => "State Sync Done",
            Message::StopVertex => "Pending Stop Vertex",
        }
    }
}

impl TryFrom<u32> for Message {
    type Error = String;
    fn try_from(v: u32) -> Result<Self, Self::Error> {
        match v {
            x if x == Message::PendingTxs as u32 => Ok(Message::PendingTxs),
            x if x == Message::StateSyncDone as u32 => Ok(Message::StateSyncDone),
            x if x == Message::StopVertex as u32 => Ok(Message::StopVertex),
            _ => Err("invalid message enum".to_string()),
        }
    }
}

#[test]
fn test_message() {
    let m = Message::try_from(1).unwrap();
    assert_eq!(m, Message::PendingTxs);
    assert!(m.as_str().contains("Pending Transactions"));
    assert!(Message::try_from(5).is_err());
}
