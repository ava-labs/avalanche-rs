use crate::errors::{Error, Result};
use bip32::{DerivationPath, Language, Mnemonic, XPrv};
use rand_core::OsRng;

/// ref. <https://github.com/ava-labs/avax-js-cli-tools/blob/3e3f714e4227aca83dc3978fcb6a4fd698e09065/address_gen.js>
pub const AVAX_ACCOUNT_DERIV_PATH: &str = "m/44'/9000'/0'";
pub const AVAX_ACCOUNT_DERIV_PATH_0: &str = "m/44'/9000'/0'/0/0";

/// ref. <https://github.com/ava-labs/avalanche-wallet/blob/v0.3.8/src/js/wallets/MnemonicWallet.ts>
pub const AVAX_ACCOUNT_EXT_PUB_KEY_DERIV_PATH: &str = "m/44'/9000'/0'";
pub const ETH_ACCOUNT_EXT_PUB_KEY_DERIV_PATH: &str = "m/44'/60'/0'/0/0";

/// Only supports "English" for now.
/// ref. <https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki>
/// ref. <https://github.com/rust-bitcoin/rust-bitcoin/blob/master/src/util/bip32.rs>
/// ref. <https://github.com/bitcoin/bips/blob/master/bip-0039/bip-0039-wordlists.md>
/// ref. <https://iancoleman.io/bip39/>
pub fn gen_24() -> String {
    let m = Mnemonic::random(&mut OsRng, Language::English);
    let s = m.phrase();
    assert_eq!(s.split(' ').count(), 24);
    String::from(s)
}

impl crate::key::secp256k1::private_key::Key {
    /// Loads the private key from the mnemonic phrase.
    pub fn from_mnemonic_phrase<S>(phrase: S, derive_path: S) -> Result<Self>
    where
        S: AsRef<str>,
    {
        let deriv: DerivationPath = derive_path.as_ref().parse().map_err(|e| Error::Other {
            message: format!("failed to parse derive path ({})", e),
            retryable: false,
        })?;

        let mnemonic = Mnemonic::new(phrase, Language::English).map_err(|e| Error::Other {
            message: format!("failed to read mnemonic phrase ({})", e),
            retryable: false,
        })?;
        let seed = mnemonic.to_seed("password");

        // ref. https://github.com/ava-labs/avalanche-wallet/blob/v0.3.8/src/js/wallets/MnemonicWallet.ts
        let child_xprv = XPrv::derive_from_path(&seed, &deriv).map_err(|e| Error::Other {
            message: format!("failed to derive AVAX account path ({})", e),
            retryable: false,
        })?;

        let pk = child_xprv.private_key().to_bytes();
        Self::from_bytes(&pk)
    }
}
