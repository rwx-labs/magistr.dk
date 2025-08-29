use std::{
    net::{Ipv6Addr, SocketAddr},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use url::Url;

const DEFAULT_MAX_CONNECTIONS: u32 = 5;
const DEFAULT_IDLE_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct Config {
    /// Tracing configuration
    pub tracing: Tracing,
    /// Database configuration
    pub database: Database,
    /// HTTP configuration
    pub http: Http,
}

/// Tracing configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Tracing {
    /// Enable tracing
    pub enabled: bool,
}

/// Database configuration
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Database {
    /// Connection URL to database
    pub url: Url,
    /// Maximum number of connections for the connection pool.
    pub max_connections: u32,
    /// Idle timeout, in seconds.
    pub idle_timeout: Duration,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Http {
    /// The address to listen to connections on.
    pub address: SocketAddr,
    /// Enable support for compressing responses if the client asks for it.
    pub compression: bool,
}

impl Default for Http {
    fn default() -> Self {
        Http {
            address: SocketAddr::from((Ipv6Addr::UNSPECIFIED, 3000)),
            compression: true,
        }
    }
}

impl Default for Tracing {
    fn default() -> Self {
        Tracing { enabled: true }
    }
}

impl Default for Database {
    fn default() -> Self {
        Database {
            url: Url::parse("postgresql://magistr:password@localhost/magistr_development").unwrap(),
            max_connections: DEFAULT_MAX_CONNECTIONS,
            idle_timeout: DEFAULT_IDLE_TIMEOUT,
        }
    }
}
