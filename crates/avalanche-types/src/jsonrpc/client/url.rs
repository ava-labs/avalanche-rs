//! Url is a helper module for creating avalanche node URLs.
use hyper::{http::uri::Builder, http::uri::Scheme, Uri};
use strum::{Display, IntoStaticStr};

use crate::errors::Error;

/// Path represents the various http client paths that can be called.
/// Each Path has a distinct url.
#[non_exhaustive]
#[derive(Debug, Display, IntoStaticStr)]
pub enum Path {
    /// The admin url path /ext/admin
    #[strum(to_string = "/ext/admin")]
    Admin,
    /// The info url path /ext/info
    #[strum(to_string = "/ext/info")]
    Info,
    /// The health url path /ext/health
    #[strum(to_string = "/ext/health")]
    Health,
    /// The liveness url path /ext/health/liveness
    #[strum(to_string = "/ext/health/liveness")]
    Liveness,
    /// The P-chain url path /ext/P
    #[strum(to_string = "/ext/P")]
    P,
    /// The X-chain url path /ext/bc/X
    #[strum(to_string = "/ext/bc/X")]
    X,
    /// The C-chain url path /ext/bc/C/rpc
    #[strum(to_string = "/ext/bc/C/rpc")]
    C,
    /// A custom path for a subnet rpc url for example.
    #[strum(to_string = "{0}")]
    Custom(String),
}

/// new returns a Url from path-based components.
/// By default the scheme is http.
/// In case of an error marshaling to a Url, a non-retryable error is returned.
pub fn try_create_url(
    path: Path,
    scheme: Option<&str>,
    host: &str,
    port: Option<u16>,
) -> Result<Uri, Error> {
    Builder::new()
        .authority(if let Some(port) = port {
            format!("{host}:{port}")
        } else {
            host.to_string()
        })
        .scheme(scheme.unwrap_or(Scheme::HTTP.as_str()))
        .path_and_query(path.to_string())
        .build()
        .map_err(|e| Error::Other {
            message: format!("http://{host}{path} failed url::try_create_url '{}'", e),
            retryable: false,
        })
}

#[cfg(test)]
mod tests {
    use super::Path;

    #[test]
    fn test_try_create_url() {
        let test_table: Vec<(Option<&str>, &str, Option<u16>)> =
            vec![(None, "127.0.0.1", Some(9650))];

        assert_eq!(
            super::try_create_url(
                Path::Admin,
                test_table[0].0,
                test_table[0].1,
                test_table[0].2
            )
            .unwrap()
            .to_string(),
            "http://127.0.0.1:9650/ext/admin".to_string()
        );
        assert_eq!(
            super::try_create_url(
                Path::Liveness,
                test_table[0].0,
                test_table[0].1,
                test_table[0].2
            )
            .unwrap()
            .to_string(),
            "http://127.0.0.1:9650/ext/health/liveness".to_string()
        );
        assert_eq!(
            super::try_create_url(Path::C, test_table[0].0, test_table[0].1, test_table[0].2)
                .unwrap()
                .to_string(),
            "http://127.0.0.1:9650/ext/bc/C/rpc".to_string()
        );
    }
}
