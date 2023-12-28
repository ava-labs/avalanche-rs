use std::{
    fs::{self, File},
    io::{self, BufReader, Error, ErrorKind, Read, Write},
    path::Path,
};

use rcgen::{
    date_time_ymd, BasicConstraints, Certificate, CertificateParams, CertificateSigningRequest,
    DistinguishedName, DnType, IsCa, KeyPair,
};
use rsa::{pkcs1::LineEnding, pkcs8::EncodePrivateKey, RsaPrivateKey};
use rustls_pemfile::{read_one, Item};

/// Represents a certificate authoriry.
/// CA acts as a trusted third party.
/// ref. <https://en.wikipedia.org/wiki/Certificate_authority>
/// ref. <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
pub struct Ca {
    pub cert: Certificate,
}

impl Ca {
    pub fn new(common_name: &str) -> io::Result<Self> {
        let cert_params = default_params(None, Some(common_name.to_string()), true)?;
        let cert = generate(Some(cert_params))?;
        Ok(Self { cert })
    }

    pub fn new_with_parameters(cert_params: Option<CertificateParams>) -> io::Result<Self> {
        let cert = generate(cert_params)?;
        Ok(Self { cert })
    }

    /// Saves the certificate in PEM format.
    pub fn save(
        &self,
        overwrite: bool,
        key_path: Option<&str>,
        cert_path: Option<&str>,
    ) -> io::Result<(String, String)> {
        let key_path = if let Some(p) = key_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("key path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".key"))?
        };

        let cert_path = if let Some(p) = cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("cert path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".cert"))?
        };

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let key_contents = self.cert.serialize_private_key_pem();
        let mut key_file = File::create(&key_path)?;
        key_file.write_all(key_contents.as_bytes())?;
        log::info!("saved key '{key_path}' ({}-byte)", key_contents.len());

        let cert_contents = self
            .cert
            .serialize_pem()
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {}", e)))?;
        let mut cert_file = File::create(&cert_path)?;
        cert_file.write_all(cert_contents.as_bytes())?;
        log::info!("saved cert '{cert_path}' ({}-byte)", cert_contents.len());

        Ok((key_path, cert_path))
    }

    /// Issues a certificate in PEM format.
    /// And returns the issued certificate in PEM format.
    pub fn issue_cert(&self, csr_pem: &str) -> io::Result<String> {
        log::info!("issuing a cert for CSR");
        let csr = CertificateSigningRequest::from_pem(csr_pem).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed CertificateSigningRequest::from_pem {}", e),
            )
        })?;
        csr.serialize_pem_with_signer(&self.cert).map_err(|e| {
            Error::new(
                ErrorKind::Other,
                format!("failed serialize_pem_with_signer {}", e),
            )
        })
    }

    /// Issues and saves a certificate in PEM format.
    /// And returns the issued cert in PEM format, and the saved cert file path.
    pub fn issue_and_save_cert(
        &self,
        csr_pem: &str,
        overwrite: bool,
        cert_path: Option<&str>,
    ) -> io::Result<(String, String)> {
        let cert_path = if let Some(p) = cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        log::info!("saving the issued certificate in '{cert_path}'");
        let issued_cert = self.issue_cert(csr_pem)?;
        let mut issued_cert_file = File::create(&cert_path)?;
        issued_cert_file.write_all(issued_cert.as_bytes())?;
        log::info!("saved cert '{cert_path}' ({}-byte)", issued_cert.len());

        Ok((issued_cert, cert_path))
    }
}

/// Represents a certificate signing request entity.
/// ref. <https://en.wikipedia.org/wiki/Certificate_signing_request>
/// ref. <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
pub struct CsrEntity {
    pub cert: Certificate,
    pub csr_pem: String,
}

impl CsrEntity {
    pub fn new(common_name: &str) -> io::Result<Self> {
        let cert_params = default_params(None, Some(common_name.to_string()), false)?;
        let (cert, csr_pem) = generate_csr(cert_params)?;
        Ok(Self { cert, csr_pem })
    }

    pub fn new_with_parameters(cert_params: CertificateParams) -> io::Result<Self> {
        let (cert, csr_pem) = generate_csr(cert_params)?;
        Ok(Self { cert, csr_pem })
    }

    /// Saves the CSR in PEM format, and returns the file path.
    pub fn save_csr(&self, overwrite: bool, csr_path: Option<&str>) -> io::Result<String> {
        let csr_path = if let Some(p) = csr_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        let mut csr_file = File::create(&csr_path)?;
        csr_file.write_all(self.csr_pem.as_bytes())?;
        log::info!("saved CSR '{csr_path}' ({}-byte)", self.csr_pem.len());

        Ok(csr_path)
    }

    /// Saves the key, cert, and CSR, in PEM format.
    /// And returns the file paths.
    pub fn save(
        &self,
        overwrite: bool,
        csr_key_path: Option<&str>,
        csr_cert_path: Option<&str>,
        csr_path: Option<&str>,
    ) -> io::Result<(String, String, String)> {
        let csr_key_path = if let Some(p) = csr_key_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR key path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".key"))?
        };

        let csr_cert_path = if let Some(p) = csr_cert_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR cert path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".cert"))?
        };

        let csr_path = if let Some(p) = csr_path {
            if !overwrite && Path::new(p).exists() {
                return Err(Error::new(
                    ErrorKind::Other,
                    format!("CSR path '{p}' already exists"),
                ));
            }
            p.to_string()
        } else {
            random_manager::tmp_path(10, Some(".csr.pem"))?
        };

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let csr_key_contents = self.cert.serialize_private_key_pem();
        let mut csr_key_file = File::create(&csr_key_path)?;
        csr_key_file.write_all(csr_key_contents.as_bytes())?;
        log::info!(
            "saved key '{csr_key_path}' ({}-byte)",
            csr_key_contents.len()
        );

        let csr_cert_contents = self
            .cert
            .serialize_pem()
            .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {}", e)))?;
        let mut cert_file = File::create(&csr_cert_path)?;
        cert_file.write_all(csr_cert_contents.as_bytes())?;
        log::info!(
            "saved cert '{csr_cert_path}' ({}-byte)",
            csr_cert_contents.len()
        );

        // ref. "crypto/tls.parsePrivateKey"
        // ref. "crypto/x509.MarshalPKCS8PrivateKey"
        let mut csr_file = File::create(&csr_path)?;
        csr_file.write_all(self.csr_pem.as_bytes())?;
        log::info!("saved CSR '{csr_path}' ({}-byte)", self.csr_pem.len());

        Ok((csr_key_path, csr_cert_path, csr_path))
    }
}

/// RUST_LOG=debug cargo test --all-features --lib -- x509::test_csr --exact --show-output
#[test]
fn test_csr() {
    use std::process::{Command, Stdio};

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let ca = Ca::new("ca.hello.com").unwrap();
    let (ca_key_path, ca_cert_path) = ca.save(true, None, None).unwrap();
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        ca_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            )
        }
        Err(e) => {
            log::warn!("failed to run openssl {}", e)
        }
    }

    let csr_entity = CsrEntity::new("entity.hello.com").unwrap();
    log::info!("csr_entity.csr:\n\n{}", csr_entity.csr_pem);
    let (csr_key_path, csr_cert_path, csr_path) = csr_entity.save(true, None, None, None).unwrap();
    log::info!("csr_key_path: {csr_key_path}");
    log::info!("csr_cert_path: {csr_cert_path}");
    log::info!("csr_path: {csr_path}");
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        csr_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            )
        }
        Err(e) => {
            log::warn!("failed to run openssl {}", e)
        }
    }

    let issued_cert = ca.issue_cert(&csr_entity.csr_pem).unwrap();
    log::info!("issued_cert:\n\n{issued_cert}");

    let (issued_cert, issued_cert_path) = ca
        .issue_and_save_cert(&csr_entity.csr_pem, true, None)
        .unwrap();
    log::info!("issued_cert:\n\n{issued_cert}");
    log::info!("issued_cert issued_cert_path: {issued_cert_path}");
    let openssl_args = vec![
        "x509".to_string(),
        "-text".to_string(),
        "-noout".to_string(),
        "-in".to_string(),
        issued_cert_path.to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl x509 with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            println!(
                "openssl output {} bytes:\n{}\n",
                output.stdout.len(),
                String::from_utf8(output.stdout).unwrap()
            )
        }
        Err(e) => {
            log::warn!("failed to run openssl {}", e)
        }
    }

    fs::remove_file(&ca_key_path).unwrap();
    fs::remove_file(&ca_cert_path).unwrap();

    fs::remove_file(&csr_key_path).unwrap();
    fs::remove_file(&csr_cert_path).unwrap();
    fs::remove_file(&csr_path).unwrap();

    fs::remove_file(&issued_cert_path).unwrap();
}

/// Generates a X509 certificate pair.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
///
/// See https://github.com/ava-labs/avalanche-types/blob/ad1730ed193cf1cd5056f23d130c3defc897cab5/avalanche-types/src/cert.rs
/// to use "openssl" crate.
pub fn generate(params: Option<CertificateParams>) -> io::Result<Certificate> {
    let cert_params = if let Some(p) = params {
        p
    } else {
        default_params(None, None, false)?
    };
    Certificate::from_params(cert_params).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to generate certificate {}", e),
        )
    })
}

/// Generates a certificate and returns the certificate and the CSR.
/// ref. <https://github.com/djc/sign-cert-remote/blob/main/src/main.rs>
pub fn generate_csr(params: CertificateParams) -> io::Result<(Certificate, String)> {
    let cert = generate(Some(params))?;
    let csr = cert.serialize_request_pem().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to serialize_request_pem {}", e),
        )
    })?;
    Ok((cert, csr))
}

/// Generates a X509 certificate pair and writes them as PEM files.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
///
/// See https://github.com/ava-labs/avalanche-types/blob/ad1730ed193cf1cd5056f23d130c3defc897cab5/avalanche-types/src/cert.rs
/// to use "openssl" crate.
pub fn generate_and_write_pem(
    params: Option<CertificateParams>,
    key_path: &str,
    cert_path: &str,
) -> io::Result<()> {
    log::info!("generating key '{key_path}', cert '{cert_path}' (PEM format)");
    if Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("key path '{key_path}' already exists"),
        ));
    }
    if Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("cert path '{cert_path}' already exists"),
        ));
    }

    let cert = generate(params)?;

    // ref. "crypto/tls.parsePrivateKey"
    // ref. "crypto/x509.MarshalPKCS8PrivateKey"
    let key_contents = cert.serialize_private_key_pem();
    let mut key_file = File::create(key_path)?;
    key_file.write_all(key_contents.as_bytes())?;
    log::info!("saved key '{key_path}' ({}-byte)", key_contents.len());

    let cert_contents = cert
        .serialize_pem()
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {}", e)))?;

    let mut cert_file = File::create(cert_path)?;
    cert_file.write_all(cert_contents.as_bytes())?;
    log::info!("saved cert '{cert_path}' ({}-byte)", cert_contents.len());

    Ok(())
}

/// Loads the key and certificate from the PEM-encoded files.
pub fn load_pem_to_vec(key_path: &str, cert_path: &str) -> io::Result<(Vec<u8>, Vec<u8>)> {
    log::info!("loading PEM from key path '{key_path}' and cert '{cert_path}' (as PEM)");

    if !Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("key path '{key_path}' does not exist"),
        ));
    }
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::Other,
            format!("cert path '{cert_path}' does not exist"),
        ));
    }

    Ok((read_vec(key_path)?, read_vec(cert_path)?))
}

/// Use RSA for Apple M*.
/// ref. <https://github.com/sfackler/rust-native-tls/issues/225>
#[cfg(all(target_arch = "aarch64", target_os = "macos"))]
fn default_sig_algo() -> String {
    "PKCS_RSA_SHA256".to_string()
}

#[cfg(not(all(target_arch = "aarch64", target_os = "macos")))]
fn default_sig_algo() -> String {
    "PKCS_ECDSA_P256_SHA256".to_string()
}

pub fn default_params(
    sig_algo: Option<String>,
    common_name: Option<String>,
    is_ca: bool,
) -> io::Result<CertificateParams> {
    let mut cert_params = CertificateParams::default();

    let sa = if let Some(sg) = &sig_algo {
        sg.to_string()
    } else {
        default_sig_algo()
    };
    log::info!("generating parameter with signature algorithm '{sa}'");

    let key_pair = match sa.as_str() {
        "PKCS_RSA_SHA256" => {
            cert_params.alg = &rcgen::PKCS_RSA_SHA256;
            let mut rng = rand::thread_rng();
            let private_key = RsaPrivateKey::new(&mut rng, 2048).map_err(|e| {
                Error::new(ErrorKind::Other, format!("failed to generate key {}", e))
            })?;
            let key = private_key.to_pkcs8_pem(LineEnding::CRLF).map_err(|e| {
                Error::new(ErrorKind::Other, format!("failed to convert key {}", e))
            })?;
            KeyPair::from_pem(&key).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_RSA_SHA256 key pair {}", e),
                )
            })?
        }

        // this works on linux with avalanchego
        "PKCS_ECDSA_P256_SHA256" => {
            cert_params.alg = &rcgen::PKCS_ECDSA_P256_SHA256;
            KeyPair::generate(&rcgen::PKCS_ECDSA_P256_SHA256).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_ECDSA_P256_SHA256 key pair {}", e),
                )
            })?
        }

        // this fails avalanchego peer IP verification (e.g., incorrect signature)
        //
        // currently, "avalanchego" only signs the IP with "crypto.SHA256"
        // ref. "avalanchego/network/ip_signer.go.newIPSigner"
        // ref. "avalanchego/network/peer/ip.go UnsignedIP.Sign" with "crypto.SHA256"
        //
        // TODO: support sha384/512 signatures in avalanchego node
        "PKCS_ECDSA_P384_SHA384" => {
            cert_params.alg = &rcgen::PKCS_ECDSA_P384_SHA384;
            KeyPair::generate(&rcgen::PKCS_ECDSA_P384_SHA384).map_err(|e| {
                Error::new(
                    ErrorKind::Other,
                    format!("failed to generate PKCS_ECDSA_P384_SHA384 key pair {}", e),
                )
            })?
        }

        _ => {
            return Err(Error::new(
                ErrorKind::InvalidInput,
                format!("unknown signature algorithm {sa}"),
            ))
        }
    };
    cert_params.key_pair = Some(key_pair);

    cert_params.not_before = date_time_ymd(2023, 5, 1);
    cert_params.not_after = date_time_ymd(5000, 1, 1);

    cert_params.distinguished_name = DistinguishedName::new();
    cert_params
        .distinguished_name
        .push(DnType::CountryName, "US");
    cert_params
        .distinguished_name
        .push(DnType::StateOrProvinceName, "NY");
    cert_params
        .distinguished_name
        .push(DnType::OrganizationName, "Test Org");
    if let Some(cm) = &common_name {
        cert_params
            .distinguished_name
            .push(DnType::CommonName, cm.to_string());
    } else {
        cert_params
            .distinguished_name
            .push(DnType::CommonName, "test common name");
    }

    if is_ca {
        cert_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    }

    Ok(cert_params)
}

/// RUST_LOG=debug cargo test --all-features --lib -- x509::test_pem --exact --show-output
#[test]
fn test_pem() {
    use std::process::{Command, Stdio};

    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let tmp_dir = tempfile::tempdir().unwrap();

    let key_path = tmp_dir.path().join(random_manager::secure_string(20));
    let key_path = key_path.as_os_str().to_str().unwrap();
    let mut key_path = String::from(key_path);
    key_path.push_str(".key");

    let cert_path = tmp_dir.path().join(random_manager::secure_string(20));
    let cert_path = cert_path.as_os_str().to_str().unwrap();
    let mut cert_path = String::from(cert_path);
    cert_path.push_str(".cert");
    let cert_path = random_manager::tmp_path(10, Some(".pem")).unwrap();

    generate_and_write_pem(None, &key_path, &cert_path).unwrap();
    load_pem_to_vec(&key_path, &cert_path).unwrap();

    let key_contents = fs::read(&key_path).unwrap();
    let key_contents = String::from_utf8(key_contents.to_vec()).unwrap();
    log::info!("key {}", key_contents);
    log::info!("key: {} bytes", key_contents.len());

    // openssl x509 -in [cert_path] -text -noout
    let cert_contents = fs::read(&cert_path).unwrap();
    let cert_contents = String::from_utf8(cert_contents.to_vec()).unwrap();
    log::info!("cert {}", cert_contents);
    log::info!("cert: {} bytes", cert_contents.len());

    let openssl_args = vec![
        "x509".to_string(),
        "-in".to_string(),
        cert_path.to_string(),
        "-text".to_string(),
        "-noout".to_string(),
    ];
    let openssl_cmd = Command::new("openssl")
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .args(openssl_args)
        .spawn()
        .unwrap();
    log::info!("ran openssl with PID {}", openssl_cmd.id());
    let res = openssl_cmd.wait_with_output();
    match res {
        Ok(output) => {
            log::info!(
                "openssl output:\n{}\n",
                String::from_utf8(output.stdout).unwrap()
            )
        }
        Err(e) => {
            log::warn!("failed to run openssl {}", e)
        }
    }

    let (key, cert) = load_pem_key_cert_to_der(&key_path, &cert_path).unwrap();
    log::info!("loaded key: {:?}", key);
    log::info!("loaded cert: {:?}", cert);

    let serial = load_pem_cert_serial(&cert_path).unwrap();
    log::info!("serial: {:?}", serial);

    fs::remove_file(&key_path).unwrap();
    fs::remove_file(&cert_path).unwrap();
}

/// Loads the TLS key and certificate from the PEM-encoded files, as DER.
pub fn load_pem_key_cert_to_der(
    key_path: &str,
    cert_path: &str,
) -> io::Result<(rustls::PrivateKey, rustls::Certificate)> {
    log::info!("loading PEM from key path '{key_path}' and cert '{cert_path}' (to DER)");
    if !Path::new(key_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path {} does not exists", key_path),
        ));
    }
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path {} does not exists", cert_path),
        ));
    }

    // ref. "tls.Certificate.Leaf.Raw" in Go
    // ref. "tls.X509KeyPair"
    // ref. "x509.ParseCertificate/parseCertificate"
    // ref. "x509.Certificate.Leaf"
    //
    // use openssl::x509::X509;
    // let pub_key_contents = fs::read(cert_file_path)?;
    // let pub_key = X509::from_pem(&pub_key_contents.to_vec())?;
    // let pub_key_der = pub_key.to_der()?;
    //
    // use pem;
    // let pub_key_contents = fs::read(cert_file_path)?;
    // let pub_key = pem::parse(&pub_key_contents.to_vec()).unwrap();
    // let pub_key_der = pub_key.contents;

    let key_file = File::open(key_path)?;
    let mut reader = BufReader::new(key_file);
    let pem_read = read_one(&mut reader)?;
    let key = {
        match pem_read.unwrap() {
            Item::X509Certificate(_) => {
                log::warn!("key path {} has unexpected certificate", key_path);
                None
            }
            Item::RSAKey(key) => {
                log::info!("loaded RSA key");
                Some(key)
            }
            Item::PKCS8Key(key) => {
                log::info!("loaded PKCS8 key");
                Some(key)
            }
            Item::ECKey(key) => {
                log::info!("loaded EC key");
                Some(key)
            }
            _ => None,
        }
    };
    if key.is_none() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("key path '{key_path}' found no key"),
        ));
    }
    let key_der = key.unwrap();

    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let pem_read = read_one(&mut reader)?;
    let cert = {
        match pem_read.unwrap() {
            Item::X509Certificate(cert) => Some(cert),
            Item::RSAKey(_) | Item::PKCS8Key(_) | Item::ECKey(_) => {
                log::warn!("cert path '{cert_path}' has unexpected private key");
                None
            }
            _ => None,
        }
    };
    if cert.is_none() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' found no cert"),
        ));
    }
    let cert_der = cert.unwrap();

    Ok((rustls::PrivateKey(key_der), rustls::Certificate(cert_der)))
}

/// Loads the serial number from the PEM-encoded certificate.
pub fn load_pem_cert_serial(cert_path: &str) -> io::Result<Vec<u8>> {
    log::info!("loading PEM cert '{cert_path}'");
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' does not exists"),
        ));
    }

    let cert_raw = read_vec(cert_path)?;

    let (_, parsed) = x509_parser::pem::parse_x509_pem(&cert_raw)
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed parse_x509_pem {}", e)))?;
    let cert = parsed.parse_x509().map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed parse_x509_certificate {}", e),
        )
    })?;
    let serial = cert.serial.clone();

    Ok(serial.to_bytes_be())
}

/// Loads the PEM-encoded certificate as DER.
pub fn load_pem_cert_to_der(cert_path: &str) -> io::Result<rustls::Certificate> {
    log::info!("loading PEM cert '{cert_path}' (to DER)");
    if !Path::new(cert_path).exists() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' does not exists"),
        ));
    }

    let cert_file = File::open(cert_path)?;
    let mut reader = BufReader::new(cert_file);
    let pem_read = read_one(&mut reader)?;
    let cert = {
        match pem_read.unwrap() {
            Item::X509Certificate(cert) => Some(cert),
            Item::RSAKey(_) | Item::PKCS8Key(_) | Item::ECKey(_) => {
                log::warn!("cert path '{cert_path}' has unexpected private key");
                None
            }
            _ => None,
        }
    };
    if cert.is_none() {
        return Err(Error::new(
            ErrorKind::NotFound,
            format!("cert path '{cert_path}' found no cert"),
        ));
    }
    let cert_der = cert.unwrap();

    Ok(rustls::Certificate(cert_der))
}

/// Generates a X509 certificate pair and returns them in DER format.
/// ref. <https://pkg.go.dev/github.com/ava-labs/avalanchego/staking#NewCertAndKeyBytes>
pub fn generate_der(
    params: Option<CertificateParams>,
) -> io::Result<(rustls::PrivateKey, rustls::Certificate)> {
    log::info!("generating key and cert (DER format)");

    let cert_params = if let Some(p) = params {
        p
    } else {
        default_params(None, None, false)?
    };
    let cert = Certificate::from_params(cert_params).map_err(|e| {
        Error::new(
            ErrorKind::Other,
            format!("failed to generate certificate {}", e),
        )
    })?;
    let cert_der = cert
        .serialize_der()
        .map_err(|e| Error::new(ErrorKind::Other, format!("failed to serialize_pem {}", e)))?;
    // ref. "crypto/tls.parsePrivateKey"
    // ref. "crypto/x509.MarshalPKCS8PrivateKey"
    let key_der = cert.serialize_private_key_der();

    Ok((rustls::PrivateKey(key_der), rustls::Certificate(cert_der)))
}

/// Loads the TLS key and certificate from the DER-encoded files.
pub fn load_der_key_cert(
    key_path: &str,
    cert_path: &str,
) -> io::Result<(rustls::PrivateKey, rustls::Certificate)> {
    log::info!("loading DER from key path '{key_path}' and cert '{cert_path}'");
    let (key, cert) = fs::read(key_path).and_then(|x| Ok((x, fs::read(cert_path)?)))?;
    Ok((rustls::PrivateKey(key), rustls::Certificate(cert)))
}

/// RUST_LOG=debug cargo test --all-features --lib -- x509::test_generate_der --exact --show-output
#[test]
fn test_generate_der() {
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Info)
        .is_test(true)
        .try_init();

    let (key, cert) = generate_der(None).unwrap();
    log::info!("key: {} bytes", key.0.len());
    log::info!("cert: {} bytes", cert.0.len());
}

/// ref. <https://doc.rust-lang.org/std/fs/fn.read.html>
fn read_vec(p: &str) -> io::Result<Vec<u8>> {
    let mut f = File::open(p)?;
    let metadata = fs::metadata(p)?;
    let mut buffer = vec![0; metadata.len() as usize];
    let _read_bytes = f.read(&mut buffer)?;
    Ok(buffer)
}
