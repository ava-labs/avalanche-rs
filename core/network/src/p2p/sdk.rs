#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullGossipRequest {
    #[prost(bytes = "vec", tag = "1")]
    pub filter: ::prost::alloc::vec::Vec<u8>,
    #[prost(bytes = "vec", tag = "2")]
    pub salt: ::prost::alloc::vec::Vec<u8>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PullGossipResponse {
    #[prost(bytes = "vec", repeated, tag = "1")]
    pub gossip: ::prost::alloc::vec::Vec<::prost::alloc::vec::Vec<u8>>,
}
