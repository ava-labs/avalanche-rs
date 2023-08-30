use std::io::{self, Error, ErrorKind};

use url::Url;

pub fn extract_scheme_host_port_path_chain_alias(
    s: &str,
) -> io::Result<(
    Option<String>, // scheme
    String,         // host
    Option<u16>,    // port
    Option<String>, // URL path
    Option<String>, // chain alias
)> {
    if !s.starts_with("http://") && !s.starts_with("https://") {
        let (_, host, port, path, chain_alias) = parse_url(format!("http://{s}").as_str())?;
        return Ok((None, host, port, path, chain_alias));
    }
    parse_url(s)
}

fn parse_url(
    s: &str,
) -> io::Result<(
    Option<String>,
    String,
    Option<u16>,
    Option<String>,
    Option<String>,
)> {
    let url = Url::parse(&s).map_err(|e| {
        Error::new(
            ErrorKind::InvalidInput,
            format!("failed Url::parse '{}'", e),
        )
    })?;

    let host = if let Some(hs) = url.host_str() {
        hs.to_string()
    } else {
        return Err(Error::new(ErrorKind::InvalidInput, "no host found"));
    };

    let port = if let Some(port) = url.port() {
        Some(port)
    } else {
        None // e.g., DNS
    };

    let (path, chain_alias) = if url.path().is_empty() || url.path() == "/" {
        (None, None)
    } else {
        // e.g., "/ext/bc/C/rpc"
        if let Some(mut path_segments) = url.path_segments() {
            let _ext = path_segments.next();
            let _bc = path_segments.next();
            let chain_alias = path_segments.next();
            if let Some(ca) = chain_alias {
                (Some(url.path().to_string()), Some(ca.to_string()))
            } else {
                (Some(url.path().to_string()), None)
            }
        } else {
            (Some(url.path().to_string()), None)
        }
    };

    Ok((
        Some(url.scheme().to_string()),
        host,
        port,
        path,
        chain_alias,
    ))
}

/// RUST_LOG=debug cargo test --package avalanche-types --lib -- utils::urls::test_extract_scheme_host_port_path_chain_alias --exact --show-output
#[test]
fn test_extract_scheme_host_port_path_chain_alias() {
    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://localhost:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "localhost");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("localhost:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "localhost");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://abc:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "abc");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("abc:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "abc");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://127.0.0.1:9650").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("127.0.0.1:9650").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert!(path.is_none());
    assert!(chain_alias.is_none());

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("http://127.0.0.1:9650/ext/bc/C/rpc").unwrap();
    assert_eq!(scheme.unwrap(), "http");
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert_eq!(path.unwrap(), "/ext/bc/C/rpc");
    assert_eq!(chain_alias.unwrap(), "C");

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("127.0.0.1:9650/ext/bc/C/rpc").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "127.0.0.1");
    assert_eq!(port.unwrap(), 9650);
    assert_eq!(path.unwrap(), "/ext/bc/C/rpc");
    assert_eq!(chain_alias.unwrap(), "C");

    let (scheme, host, port, path, chain_alias) =
        extract_scheme_host_port_path_chain_alias("1.2.3.4:1/ext/bc/abcde/rpc").unwrap();
    assert!(scheme.is_none());
    assert_eq!(host, "1.2.3.4");
    assert_eq!(port.unwrap(), 1);
    assert_eq!(path.unwrap(), "/ext/bc/abcde/rpc");
    assert_eq!(chain_alias.unwrap(), "abcde");
}
