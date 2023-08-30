use std::{
    io::{self, stdout, Error, ErrorKind},
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{
    c, flags, logs,
    spec::{self, Spec, Status},
    x,
};
use aws_manager::kms;
use crossterm::{
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
};
use dialoguer::{theme::ColorfulTheme, Select};
use rand::{seq::SliceRandom, thread_rng};
use tokio::{sync::RwLock, time::sleep};

pub async fn execute(opts: flags::Options) -> io::Result<()> {
    logs::setup_logger(opts.log_level);

    log::info!(
        "executing 'avalanche-e2e' with spec file '{}'",
        opts.spec_path
    );

    let mut spec = Spec::load(&opts.spec_path).expect("failed to load spec");
    spec.validate()?;
    execute!(
        stdout(),
        SetForegroundColor(Color::Blue),
        Print(format!("\nLoaded Spec: '{}'\n", opts.spec_path)),
        ResetColor
    )?;
    let spec_contents = spec.encode_yaml()?;
    println!("{}\n", spec_contents);
    log::info!("loaded {} key infos from spec", spec.key_infos.len());

    if !opts.skip_prompt {
        let options = &[
            "No, I am not ready to run e2e tests!",
            "Yes, let's run e2e tests!",
        ];
        let selected = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select your option")
            .items(&options[..])
            .default(0)
            .interact()
            .unwrap();
        if selected == 0 {
            return Ok(());
        }
    }

    // set up local network if necessary
    match spec.rpc_endpoint_kind.as_str() {
        spec::RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER => {
            log::info!("network runner RPC server given, launching a new local network with gRPC client...");
            if spec.avalanchego_path.is_none() {
                // download "avalanchego" and plugins
                let bin =
                    avalanche_installer::avalanchego::github::download_latest(None, None).await?;
                spec.avalanchego_path = Some(bin.clone());
                spec.avalanchego_plugin_dir =
                    Some(avalanche_installer::avalanchego::get_plugin_dir(bin));
            } else {
                spec.avalanchego_plugin_dir =
                    Some(avalanche_installer::avalanchego::get_plugin_dir(
                        spec.avalanchego_path.clone().unwrap(),
                    ));
            }
            spec.sync(&opts.spec_path)?;

            launch_anr_network(
                &spec.rpc_endpoints[0],
                &spec.avalanchego_path.clone().unwrap(),
                &spec.avalanchego_plugin_dir.clone().unwrap(),
            )
            .await?;
        }
        spec::RPC_ENDPOINT_KIND_AVALANCHEGO_RPC_ENDPOINT => {
            log::info!("avalanchego already running, connecting to RPC servers...");
        }
        _ => {
            return Err(Error::new(
                ErrorKind::Other,
                format!("unknown RPC endpoint kind '{}'", spec.rpc_endpoint_kind),
            ));
        }
    }

    // fetches RPC endpoints
    let rpc_eps = match spec.rpc_endpoint_kind.as_str() {
        spec::RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER => {
            log::info!("waiting some time for network-runner to get ready...");
            sleep(Duration::from_secs(30)).await;
            check_anr_network(&spec.rpc_endpoints[0]).await?
        }

        // assume the nodes are ready
        spec::RPC_ENDPOINT_KIND_AVALANCHEGO_RPC_ENDPOINT => spec.rpc_endpoints.clone(),

        _ => unreachable!("unknown rpc_endpoint_kind"),
    };

    let orig_rpc_endpoints = spec.rpc_endpoints.clone();

    spec.rpc_endpoints = rpc_eps.clone();
    log::info!("running with RPC endpoints {:?}", rpc_eps);

    let resp = avalanche_types::jsonrpc::client::info::get_network_id(&rpc_eps[0])
        .await
        .unwrap();
    let network_id = resp.result.unwrap().network_id;

    let mut randomized_scenarios = spec.scenarios.clone();
    randomized_scenarios.shuffle(&mut thread_rng());

    spec.status = Some(Status {
        network_id,
        network_runner_endpoint: None,
        randomized_scenarios,
    });
    spec.sync(&opts.spec_path)?;

    let (is_anr, anr_ep) = (
        spec.rpc_endpoint_kind
            .eq(spec::RPC_ENDPOINT_KIND_NETWORK_RUNNER_RPC_SERVER),
        orig_rpc_endpoints[0].clone(),
    );
    if is_anr {
        let mut status = spec.status.clone().unwrap();
        status.network_runner_endpoint = Some(anr_ep.clone());
        spec.status = Some(status);
        spec.sync(&opts.spec_path)?;
    }

    // runs test cases
    let scenerios = {
        if !spec.randomize {
            spec.scenarios.clone()
        } else {
            spec.status.clone().unwrap().randomized_scenarios.clone()
        }
    };
    let parallelize = spec.parallelize;
    let spec_arc = Arc::new(RwLock::new(spec.clone()));

    if !parallelize {
        for (i, s) in scenerios.iter().enumerate() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Cyan),
                Print(format!("\n\n[{:03}] scenerio '{}':\n", i, s.as_str())),
                ResetColor
            )?;
            match s.as_str() {
                x::simple_transfers::NAME => {
                    x::simple_transfers::run(spec_arc.clone()).await?;
                }
                x::exports::NAME => {
                    x::exports::run(spec_arc.clone()).await?;
                }
                x::byzantine::conflicting_transfers::NAME => {
                    x::byzantine::conflicting_transfers::run(spec_arc.clone()).await?;
                }
                x::byzantine::conflicting_parallel_transfers::NAME => {
                    x::byzantine::conflicting_parallel_transfers::run(spec_arc.clone()).await?;
                }
                c::simple_transfers::NAME => {
                    c::simple_transfers::run(spec_arc.clone()).await?;
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("unknown scenario '{}'", s),
                    ))
                }
            }
        }
    } else {
        let mut handles = vec![];
        for (i, s) in scenerios.iter().enumerate() {
            execute!(
                stdout(),
                SetForegroundColor(Color::Cyan),
                Print(format!("\n\n[{:03}] scenerio '{}':\n", i, s.as_str())),
                ResetColor
            )?;
            match s.as_str() {
                x::simple_transfers::NAME => {
                    handles.push(tokio::spawn(x::simple_transfers::run(spec_arc.clone())));
                }
                x::byzantine::conflicting_transfers::NAME => {
                    handles.push(tokio::spawn(x::byzantine::conflicting_transfers::run(
                        spec_arc.clone(),
                    )));
                }
                x::byzantine::conflicting_parallel_transfers::NAME => {
                    handles.push(tokio::spawn(
                        x::byzantine::conflicting_parallel_transfers::run(spec_arc.clone()),
                    ));
                }
                c::simple_transfers::NAME => {
                    handles.push(tokio::spawn(c::simple_transfers::run(spec_arc.clone())));
                }
                _ => {
                    return Err(Error::new(
                        ErrorKind::Other,
                        format!("unknown scenario '{}'", s),
                    ))
                }
            }
        }

        log::info!("blocking on handles via JoinHandle");
        for handle in handles {
            let r = handle.await.map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed await on JoinHandle {}", e),
                )
            })?;
            match r {
                Ok(_) => {}
                Err(e) => return Err(Error::new(ErrorKind::Other, format!("case failed {}", e))),
            }
        }
    }

    if is_anr {
        stop_anr_network(&anr_ep).await?;
    }

    for key_info in spec.key_infos.iter() {
        if key_info.key_type != avalanche_types::key::secp256k1::KeyType::AwsKms {
            continue;
        }
        let shared_config = aws_manager::load_config(None, None, None).await;
        let kms_manager = kms::Manager::new(&shared_config);
        kms_manager
            .schedule_to_delete(key_info.id.clone().unwrap().as_str(), 7)
            .await
            .unwrap();
    }

    Ok(())
}

/// Launches a local network via avalanche-network-runner RPC server.
async fn launch_anr_network(
    ep: &str,
    avalanchego_path: &str,
    _avalanchego_plugin_dir: &str,
) -> io::Result<()> {
    let cli = avalanche_network_runner_sdk::Client::new(ep).await;

    log::info!("ping network-runner RPC server...");
    let resp = cli.ping().await?;
    log::info!("network-runner is running (ping response {:?})", resp);

    log::info!("sending start request...");
    let resp = cli
        .start(avalanche_network_runner_sdk::StartRequest {
            exec_path: String::from(avalanchego_path),
            global_node_config: Some(
                serde_json::to_string(&avalanche_network_runner_sdk::GlobalConfig {
                    log_level: String::from("INFO"),
                })
                .unwrap(),
            ),
            ..Default::default()
        })
        .await?;
    log::info!(
        "started avalanchego cluster with network-runner: {:?}",
        resp
    );

    Ok(())
}

/// Checks the status of the local network and fetches its information.
async fn check_anr_network(ep: &str) -> io::Result<Vec<String>> {
    let cli = avalanche_network_runner_sdk::Client::new(ep).await;

    log::info!("checking cluster healthiness...");
    let mut ready = false;
    let timeout = Duration::from_secs(300);
    let interval = Duration::from_secs(15);
    let start = Instant::now();
    let mut cnt: u128 = 0;
    loop {
        let elapsed = start.elapsed();
        if elapsed.gt(&timeout) {
            break;
        }

        let itv = {
            if cnt == 0 {
                // first poll with no wait
                Duration::from_secs(1)
            } else {
                interval
            }
        };
        sleep(itv).await;

        ready = {
            match cli.health().await {
                Ok(_) => {
                    log::info!("healthy now!");
                    true
                }
                Err(e) => {
                    log::warn!("not healthy yet {}", e);
                    false
                }
            }
        };
        if ready {
            break;
        }

        cnt += 1;
    }
    assert!(ready);

    log::info!("checking status...");
    let status = cli.status().await.expect("failed status");
    assert!(status.cluster_info.is_some());
    let cluster_info = status.cluster_info.unwrap();
    let mut rpc_eps: Vec<String> = Vec::new();
    for (node_name, iv) in cluster_info.node_infos.into_iter() {
        log::info!("{}: {}", node_name, iv.uri);
        rpc_eps.push(iv.uri.clone());
    }
    log::info!("avalanchego RPC endpoints: {:?}", rpc_eps);

    Ok(rpc_eps)
}

/// Stops the local network via avalanche-network-runner RPC server.
async fn stop_anr_network(ep: &str) -> io::Result<()> {
    let cli = avalanche_network_runner_sdk::Client::new(ep).await;

    log::info!("stopping local network via network-runner RPC server..");
    cli.stop().await?;
    log::info!("successfully stopped network");

    Ok(())
}
