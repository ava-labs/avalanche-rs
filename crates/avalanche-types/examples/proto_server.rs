use avalanche_types::{
    proto::pb::helloworld::{
        self,
        greeter_server::{Greeter, GreeterServer},
        HelloReply, HelloRequest,
    },
    subnet::rpc::utils,
};
use tonic::{Request, Response, Status};

#[derive(Default)]
struct MyGreeter;

#[tonic::async_trait]
impl Greeter for MyGreeter {
    async fn say_hello(
        &self,
        request: Request<HelloRequest>,
    ) -> Result<Response<HelloReply>, Status> {
        println!("Got a request from {:?}", request.remote_addr());

        let reply = HelloReply {
            message: format!("Hello {}!", request.into_inner().name),
        };
        Ok(Response::new(reply))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "[::1]:50051".parse().unwrap();
    println!("server listening on {}", addr);

    // ref. https://github.com/hyperium/tonic/blob/v0.8.1/examples/src/reflection/server.rs
    let reflection_svc = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(helloworld::FILE_DESCRIPTOR_SET)
        .build()
        .expect("failed to build reflection service");

    let greeter_svc = GreeterServer::new(MyGreeter::default());

    utils::grpc::default_server()
        .add_service(reflection_svc)
        .add_service(greeter_svc)
        .serve(addr)
        .await?;

    Ok(())
}
