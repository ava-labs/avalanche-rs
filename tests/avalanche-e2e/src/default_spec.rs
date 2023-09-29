use std::{
    collections::HashMap,
    io::{self, stdout},
};

use crate::{
    flags, logs,
    spec::{self, Spec},
};
use aws_manager::kms;
use clap::{value_parser, Arg, Command};
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};

pub const NAME: &str = "default-spec";

pub fn command() -> Command {
    Command::new(NAME)
        .about("Writes a default spec file")
        .arg(
            Arg::new("RANDOMIZE")
                .long("randomize")
                .help("Sets to randomize test order")
                .required(false)
                .num_args(0)
        )
        .arg(
            Arg::new("PARALLELIZE")
                .long("parallelize")
                .help("Sets to parallelize test runs")
                .required(false)
                .num_args(0)
        )
        .arg(
            Arg::new("IGNORE_ERRORS")
                .long("ignore-errors")
                .help("Sets to ignore errors from test runs")
                .required(false)
                .num_args(0)
        )
        .arg(
            Arg::new("NETWORK_ID")
                .long("network-id")
                .help("Sets the network Id")
                .required(false)
                .num_args(1)
                .value_parser(value_parser!(u32))
                .default_value("1337"),
        )
        .arg(
            Arg::new("KEYS_TO_GENERATE")
                .long("keys-to-generate")
                .help("Sets the number of keys to generate")
                .required(false)
                .num_args(1)
                .value_parser(value_parser!(usize))
                .default_value("30"),
        )
        .arg(
            Arg::new("SIGN_WITH_KMS_AWS")
                .long("sign-with-kms-aws")
                .help("Sets to sign transactions with AW KMS")
                .required(false)
                .num_args(0)
        )
        .arg(
            Arg::new("NETWORK_RUNNER_GRPC_ENDPOINT")
                .long("network-runner-grpc-endpoint")
                .help("Sets the gRPC endpoint for network-runner RPC server, only required for network runner runs")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new("NETWORK_RUNNER_AVALANCHEGO_PATH")
                .long("network-runner-avalanchego-path")
                .help("Sets the AvalancheGo binary path, only required for network runner runs")
                .required(false)
                .num_args(1),
        )
        .arg(
            Arg::new("AVALANCHEGO_RPC_ENDPOINT")
                .long("avalanchego-rpc-endpoint")
                .help("Sets the AvalancheGo RPC endpoint, only required for running against existing clusters")
                .required(false)
                .num_args(1),
        )
}

pub struct Options {
    pub randomize: bool,
    pub parallelize: bool,
    pub ignore_errors: bool,

    pub network_id: u32,

    pub keys_to_generate: usize,
    pub sign_with_kms_aws: bool,

    pub network_runner_grpc_endpoint: Option<String>,
    pub network_runner_avalanchego_path: Option<String>,

    pub avalanchego_rpc_endpoint: Option<String>,
}

pub async fn execute(opts: flags::Options, sub_opts: Options) -> io::Result<()> {
    logs::setup_logger(opts.log_level);
    log::info!("executing 'avalanche-e2e default-spec'");

    assert!(sub_opts.keys_to_generate > 0);
    let signing_key_file = random_manager::tmp_path(10, Some(".yaml"))?;

    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!(
            "Generating {} keys to '{}'\n",
            sub_opts.keys_to_generate, signing_key_file
        )),
        ResetColor
    )?;

    let keys_to_generate = if sub_opts.sign_with_kms_aws {
        log::info!(
            "overwriting keys_to_generate {} to 2",
            sub_opts.keys_to_generate
        );
        2
    } else {
        sub_opts.keys_to_generate
    };

    let mut key_infos: Vec<avalanche_types::key::secp256k1::Info> = Vec::new();
    for i in 0..keys_to_generate {
        let ki = {
            // first key, just use hot "ewoq" key
            // to use the prefunds from network runner genesis
            if i == 0 {
                avalanche_types::key::secp256k1::TEST_KEYS[i]
                    .clone()
                    .to_info(sub_opts.network_id)
                    .unwrap()
            } else if sub_opts.sign_with_kms_aws {
                let shared_config = aws_manager::load_config(None, None, None).await;
                let kms_manager = kms::Manager::new(&shared_config);

                let mut tags = HashMap::new();
                tags.insert(String::from("Name"), format!("avalanche-e2e-kms-key-{i}"));

                let key = avalanche_types::key::secp256k1::kms::aws::Key::create(
                    kms_manager.clone(),
                    tags,
                )
                .await
                .unwrap();

                let key_info = key.to_info(sub_opts.network_id).unwrap();
                println!("key_info: {}", key_info);
                key_info
            } else if i < avalanche_types::key::secp256k1::TEST_KEYS.len() {
                avalanche_types::key::secp256k1::TEST_KEYS[i]
                    .clone()
                    .to_info(sub_opts.network_id)
                    .unwrap()
            } else {
                avalanche_types::key::secp256k1::private_key::Key::generate()
                    .expect("unexpected key generate failure")
                    .to_info(sub_opts.network_id)
                    .unwrap()
            }
        };
        key_infos.push(ki);
    }

    let mut spec = Spec {
        randomize: sub_opts.randomize,
        parallelize: sub_opts.parallelize,
        ignore_errors: sub_opts.ignore_errors,
        key_infos,
        ..Default::default()
    };

    if let Some(v) = sub_opts.network_runner_grpc_endpoint {
        spec.rpc_endpoint_kind = String::from(spec::RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER);
        spec.rpc_endpoints = vec![v];
        log::info!(
            "using network runner gRPC server at '{}'",
            spec.rpc_endpoints[0]
        );
    }
    if let Some(v) = sub_opts.network_runner_avalanchego_path {
        log::info!("using avalanchego path '{}'", v);
        spec.avalanchego_path = Some(v);
    }
    if let Some(v) = sub_opts.avalanchego_rpc_endpoint {
        spec.rpc_endpoint_kind = String::from(spec::RPC_ENDPOINT_KIND_AVALANCHEGO_RPC_ENDPOINT);
        spec.rpc_endpoints = vec![v];
        log::info!(
            "using AvalancheGo RPC server at '{}'",
            spec.rpc_endpoints[0]
        );
    }

    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!("Writing to '{}'\n", opts.spec_path)),
        ResetColor
    )?;
    spec.sync(&opts.spec_path)?;

    execute!(
        stdout(),
        SetForegroundColor(Color::Green),
        Print(format!("\n\ncat {}\n", opts.spec_path)),
        ResetColor
    )?;
    let spec_contents = spec.encode_yaml()?;
    println!("{}", spec_contents);

    Ok(())
}
