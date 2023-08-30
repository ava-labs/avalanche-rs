pub mod client;
pub mod handle;
pub mod server;

/// ref: <https://pkg.go.dev/net/http#Handler>
#[tonic::async_trait]
pub trait Handler {
    async fn serve_http(
        &mut self,
        req: http::Request<Vec<u8>>,
    ) -> std::io::Result<http::Response<Vec<u8>>>;

    async fn serve_http_simple(
        &mut self,
        req: http::Request<Vec<u8>>,
    ) -> std::io::Result<http::Response<Vec<u8>>>;
}
