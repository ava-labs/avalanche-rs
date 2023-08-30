use std::{collections::HashMap, env::args, io};

use avalanche_types::key;
use aws_manager::{self, kms};
use tokio::time::{self, sleep};

/// cargo run --example key_secp256k1_kms_aws --features="kms_aws"
/// cargo run --example key_secp256k1_kms_aws --features="kms_aws" -- arn:aws:sts::[ACCOUNT_ID]:assumed-role/[NAME]/[EMAIL]
#[tokio::main]
async fn main() -> io::Result<()> {
    // ref. <https://github.com/env-logger-rs/env_logger/issues/47>
    env_logger::init_from_env(
        env_logger::Env::default().filter_or(env_logger::DEFAULT_FILTER_ENV, "debug"),
    );

    log::info!("creating AWS KMS resources!");
    let shared_config = aws_manager::load_config(None, None, None).await;
    let kms_manager = kms::Manager::new(&shared_config);

    let key_name = id_manager::time::with_prefix("test");
    let mut tags = HashMap::new();
    tags.insert(String::from("Name"), key_name);

    let mut key = key::secp256k1::kms::aws::Key::create(kms_manager.clone(), tags)
        .await
        .unwrap();

    let grant_id = if let Some(arn_to_grant) = args().nth(1) {
        log::info!("creating kms grant");
        let (grant_id, grant_token) = kms_manager
            .create_grant_for_sign_reads(&key.id, &arn_to_grant)
            .await
            .unwrap();
        key.grant_token = Some(grant_token);
        Some(grant_id)
    } else {
        None
    };

    let key_info = key.to_info(1).unwrap();
    println!("key_info:\n{}", key_info);

    let key2 = key::secp256k1::kms::aws::Key::from_arn(kms_manager.clone(), &key.arn)
        .await
        .unwrap();
    let key_info2 = key2.to_info(1).unwrap();
    println!("key_info2:\n{}", key_info2);

    let digest = [0u8; ring::digest::SHA256_OUTPUT_LEN];
    match key.sign_digest(&digest).await {
        Ok(sig) => {
            log::info!(
                "successfully signed with signature output {} bytes",
                sig.to_vec().len()
            );
        }
        Err(e) => {
            log::warn!("failed to sign, error: {:?}", e);
        }
    }

    if let Some(grant_id) = &grant_id {
        log::info!("revoking kms grant");
        kms_manager.revoke_grant(&key.id, grant_id).await.unwrap();
    }

    sleep(time::Duration::from_secs(5)).await;
    key.delete(7).await.unwrap();

    // error should be ignored if it's already scheduled for delete
    sleep(time::Duration::from_secs(5)).await;
    key.delete(7).await.unwrap();

    Ok(())
}
