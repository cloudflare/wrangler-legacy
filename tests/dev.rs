pub mod fixture;

use fixture::{Fixture, WranglerToml};

use std::net::TcpListener;
use std::process::{Command, Output};

use assert_cmd::prelude::*;

#[test]
fn it_can_handle_get_requests() {
    let fixture = Fixture::new();
    let expected_body = "Page Not Found!".to_string();
    let expected_status = 404;
    fixture.create_file(
        "index.js",
        &format!(
            r#"
        addEventListener('fetch', event => {{
            event.respondWith(handleRequest(event.request))
        }})
        
        async function handleRequest(request) {{
            return new Response('{}', {{ status: {} }})
        }}
    "#,
            &expected_body, expected_status
        ),
    );

    fixture.create_default_package_json();

    let wrangler_toml = WranglerToml::javascript("test-dev-get-requests");
    fixture.create_wrangler_toml(wrangler_toml);
    let (host, output) = wrangler_dev(&fixture, Vec::new());
    let response = reqwest::blocking::get(&host).expect("could not get response");
    assert_eq!(response.status(), expected_status);
    let body = response.text().expect("could not get response body");
    println!("\n\n---\nBODY\n\n{}\n\n---\n", body);
    assert_eq!(body, expected_body);
    let output = String::from_utf8_lossy(&output.stdout);
    println!("\n\n---\nOUTPUT\n\n{}\n\n---\n", output);
    assert!(output.contains(&expected_status.to_string()));
}

fn wrangler_dev(fixture: &Fixture, args: Vec<String>) -> (String, Output) {
    let _lock = fixture.lock();
    let mut dev = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    dev.current_dir(fixture.get_path());
    dev.arg("dev");
    for arg in args {
        dev.arg(&arg);
    }
    let port = find_available_port();
    dev.arg("--port").arg(port.to_string());
    let output = dev
        .output()
        .unwrap_or_else(|_| panic!("could not start command {:?}", dev));
    let host = format!("http://localhost:{}", port);
    (host, output)
}

fn find_available_port() -> u16 {
    for port in 1025..65535 {
        if let Ok(_) = TcpListener::bind(("127.0.0.1", port)) {
            return port;
        }
    }

    panic!("Could not find a usable port");
}
