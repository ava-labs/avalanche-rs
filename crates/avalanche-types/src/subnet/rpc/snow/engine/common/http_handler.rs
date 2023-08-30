/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#LockOption>
#[derive(Debug, Clone, Copy)]
pub enum LockOptions {
    WriteLock = 0,
    ReadLock,
    NoLock,
}

/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/snow/engine/common#HTTPHandler>
#[derive(Debug, Clone)]
pub struct HttpHandler<T> {
    pub lock_option: LockOptions,
    pub handler: T,
    pub server_addr: Option<String>,
}
