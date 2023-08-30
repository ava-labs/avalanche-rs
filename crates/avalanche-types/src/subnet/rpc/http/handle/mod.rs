use std::io;

use bytes::Bytes;

use crate::proto::http::Element;

#[tonic::async_trait]
pub trait Handle: Send + Sync + Clone {
    /// Provides handling of HTTP requests.
    async fn request(&self, req: &Bytes, headers: &[Element]) -> io::Result<(Bytes, Vec<Element>)>;
}
