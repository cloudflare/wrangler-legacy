use std::net::{SocketAddr, TcpListener};

use anyhow::Result;

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::tail::Tail;

const DEFAULT_TUNNEL_PORT: u16 = 8080;
const DEFAULT_METRICS_PORT: u16 = 8081;

pub fn start(
    target: &Target,
    user: &GlobalUser,
    format: String,
    tunnel_port: Option<u16>,
    metrics_port: Option<u16>,
    verbose: bool,
) -> Result<()> {
    let tunnel_port = find_open_port(tunnel_port, DEFAULT_TUNNEL_PORT)?;
    let metrics_port = find_open_port(metrics_port, DEFAULT_METRICS_PORT)?;

    Tail::run(
        target.clone(),
        user.clone(),
        format,
        tunnel_port,
        metrics_port,
        verbose,
    )
}

/// Find open port takes two arguments: an Optional requested port, and a default port.
fn find_open_port(requested: Option<u16>, default: u16) -> Result<u16> {
    if let Some(port) = requested {
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(addr) {
            Ok(socket) => Ok(socket.local_addr()?.port()),
            Err(_) => anyhow::bail!("the specified port {} is unavailable", port),
        }
    } else {
        // try to use the default port; else get an ephemeral port from the OS
        let addrs = [
            SocketAddr::from(([127, 0, 0, 1], default)),
            // Binding to port 0 effectively asks the OS to provide the next available
            // ephemeral port: https://www.grc.com/port_0.htm
            SocketAddr::from(([127, 0, 0, 1], 0)),
        ];

        let socket = TcpListener::bind(&addrs[..])?;

        Ok(socket.local_addr()?.port())
    }
}

#[cfg(test)]
mod tests {
    // These tests are extremely stateful; since what we are testing is how this function behaves
    // when requested ports are unavailable, and since our tests run concurrently, each test uses
    // unique ports to avoid collisions. There are two possible solutions to this problem:
    // 1. require that these tests be run serially, and find a way to clean up bound ports
    // 2. use dependency injection and write a substitute for the TcpListener::bind fn.
    use super::*;

    #[test]
    fn test_find_open_port_no_requested_default_available() {
        let requested = None;
        let default = 3000;
        let port = find_open_port(requested, default).unwrap();

        // returns default
        assert_eq!(port, default);
    }

    #[test]
    fn test_find_open_port_no_requested_default_unavailable() {
        let requested = None;
        let default = 3001;
        let listener = find_open_port(requested, default);

        // returns random
        assert!(listener.is_ok());
    }

    #[test]
    fn test_find_open_port_requested_available_default_available() {
        let requested = 3002;
        let default = 3003;
        let port = find_open_port(Some(requested), default).unwrap();

        // returns requested
        assert_eq!(port, requested);
    }

    #[test]
    fn test_find_open_port_requested_available_default_unavailable() {
        let requested = 3004;
        let default = 3005;
        let _default_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], default))).unwrap();
        let port = find_open_port(Some(requested), default).unwrap();

        // returns requested
        assert_eq!(port, requested);
    }

    #[test]
    fn test_find_open_port_requested_unavailable_default_available() {
        let requested = 3006;
        let default = 3007;
        let _requested_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], requested))).unwrap();
        let listener = find_open_port(Some(requested), default);

        // throws error
        assert!(listener.is_err());
    }

    #[test]
    fn test_find_open_port_requested_unavailable_default_unavailable() {
        let requested = 3008;
        let default = 3009;
        let _requested_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], requested))).unwrap();
        let _default_listener =
            TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], default))).unwrap();
        let listener = find_open_port(Some(requested), default);

        // throws error
        assert!(listener.is_err());
    }
}
