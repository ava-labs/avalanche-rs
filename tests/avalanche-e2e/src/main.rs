pub mod c;
pub mod command;
pub mod common;
pub mod default_spec;
pub mod flags;
pub mod logs;
pub mod p;
pub mod spec;
pub mod x;

use clap::{crate_version, Arg, Command};

pub const APP_NAME: &str = "avalanche-e2e";

#[tokio::main]
async fn main() {
    let matches = Command::new(APP_NAME)
        .version(crate_version!())
        .about("Runs e2e tests against Avalanche node")
        .long_about(
            "

e.g.,

$ avalanche-e2e \
--network-id 1337 \
--log-level=info \
--spec-path=\"/tmp/avalanche.e2e.spec.yaml\" \
default-spec

$ avalanche-e2e \
--log-level=info \
--spec-path=\"/tmp/avalanche.e2e.spec.yaml\"


",
        )
        .arg(
            Arg::new("LOG_LEVEL")
                .long("log-level")
                .short('l')
                .help("Sets the log level")
                .required(false)
                .num_args(1)
                .value_parser(["debug", "info"])
                .default_value("info"),
        )
        .arg(
            Arg::new("SPEC_PATH")
                .long("spec-path")
                .short('p')
                .help("Sets the spec file path")
                .required(true)
                .num_args(1),
        )
        .arg(
            Arg::new("SKIP_PROMPT")
                .long("skip-prompt")
                .short('s')
                .help("Skips prompt mode")
                .required(false)
                .num_args(0),
        )
        .subcommands(vec![default_spec::command()])
        .get_matches();

    match matches.subcommand() {
        Some((default_spec::NAME, sub_matches)) => {
            let keys_to_generate = sub_matches
                .get_one::<usize>("KEYS_TO_GENERATE")
                .unwrap_or(&30)
                .clone();

            let network_id = sub_matches
                .get_one::<u32>("NETWORK_ID")
                .unwrap_or(&1337)
                .clone();

            let network_runner_grpc_endpoint = {
                if let Some(s) = sub_matches.get_one::<String>("NETWORK_RUNNER_GRPC_ENDPOINT") {
                    Some(String::from(s))
                } else {
                    None
                }
            };

            let avalanchego_path = {
                if let Some(s) = sub_matches.get_one::<String>("NETWORK_RUNNER_AVALANCHEGO_PATH") {
                    Some(String::from(s))
                } else {
                    None
                }
            };

            let avalanchego_rpc_endpoint = {
                if let Some(s) = sub_matches.get_one::<String>("AVALANCHEGO_RPC_ENDPOINT") {
                    Some(String::from(s))
                } else {
                    None
                }
            };

            default_spec::execute(
                flags::Options {
                    log_level: matches
                        .get_one::<String>("LOG_LEVEL")
                        .unwrap_or(&String::from("info"))
                        .clone(),
                    spec_path: matches
                        .get_one::<String>("SPEC_PATH")
                        .unwrap_or(&String::new())
                        .clone(),
                    skip_prompt: matches.get_flag("SKIP_PROMPT"),
                },
                default_spec::Options {
                    randomize: sub_matches.get_flag("RANDOMIZE"),
                    parallelize: sub_matches.get_flag("PARALLELIZE"),
                    ignore_errors: sub_matches.get_flag("IGNORE_ERRORS"),
                    network_id,
                    keys_to_generate,
                    sign_with_kms_aws: sub_matches.get_flag("SIGN_WITH_KMS_AWS"),
                    network_runner_grpc_endpoint,
                    network_runner_avalanchego_path: avalanchego_path,
                    avalanchego_rpc_endpoint,
                },
            )
            .await
            .expect("failed to execute 'default-spec'");
        }
        _ => {
            command::execute(flags::Options {
                log_level: matches
                    .get_one::<String>("LOG_LEVEL")
                    .unwrap_or(&String::from("info"))
                    .clone(),
                spec_path: matches
                    .get_one::<String>("SPEC_PATH")
                    .unwrap_or(&String::new())
                    .clone(),
                skip_prompt: matches.get_flag("SKIP_PROMPT"),
            })
            .await
            .expect("failed to execute command");
        }
    }
}
