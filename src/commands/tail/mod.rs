use std::net::{SocketAddr, TcpListener};

use crate::settings::global_user::GlobalUser;
use crate::settings::toml::Target;
use crate::tail::Tail;

const DEFAULT_TUNNEL_PORT: u16 = 8080;
const DEFAULT_METRICS_PORT: u16 = 8081;

pub fn start(
    target: &Target,
    user: &GlobalUser,
    tunnel_port: Option<u16>,
    metrics_port: Option<u16>,
) -> Result<(), failure::Error> {
    let tunnel_port = find_open_port(tunnel_port, DEFAULT_TUNNEL_PORT)?;
    let metrics_port = find_open_port(metrics_port, DEFAULT_METRICS_PORT)?;

    // Note that we use eprintln!() throughout this module; this is because we want any
    // helpful output to not be mixed with actual log JSON output, so we use this macro
    // to print messages to stderr instead of stdout (where log output is printed).
    eprintln!(
        "Setting up log streaming from Worker script \"{}\". Using ports {} and {}. This may take a few seconds...",
        target.name,
        tunnel_port,
        metrics_port,
    );

    Tail::run(target.clone(), user.clone(), tunnel_port, metrics_port)
}

/// Find open port takes two arguments: an Optional requested port, and a default port.
fn find_open_port(requested: Option<u16>, default: u16) -> Result<u16, failure::Error> {
    if let Some(port) = requested {
        let addr = format!("127.0.0.1:{}", port);
        match TcpListener::bind(addr) {
            Ok(socket) => Ok(socket.local_addr()?.port()),
            Err(_) => failure::bail!("the specified port {} is unavailable", port),
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
