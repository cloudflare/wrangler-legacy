use super::events::DevtoolsEvent;

use openssl::ssl::{SslConnector, SslMethod, SslStream, SslVerifyMode};

use ws::util::TcpStream;
use ws::{Handler, Handshake, Message as WsMessage, Sender};

use url::Url;

pub struct WsClient {
    pub out: Sender,
}

impl Handler for WsClient {
    fn on_open(&mut self, _: Handshake) -> ws::Result<()> {
        let enable_profiler = WsMessage::text(
            r#"{
                "id": 1,
                "method": "Profiler.enable"
            }"#,
        );
        let enable_runtime = WsMessage::text(
            r#"{
                "id": 2,
                "method": "Runtime.enable"
            }"#,
        );
        let enable_debugger = WsMessage::text(
            r#"{
                "id": 3,
                "method": "Debugger.enable"
            }"#,
        );
        self.out.send(enable_profiler)?;
        self.out.send(enable_runtime)?;
        self.out.send(enable_debugger)
    }

    fn on_message(&mut self, msg: WsMessage) -> ws::Result<()> {
        let msg_text = msg.as_text()?;
        // println!("{}", msg_text);
        let msg = serde_json::from_str(msg_text);
        log::info!("---\n{}", msg_text);
        match msg {
            Ok(msg) => match msg {
                DevtoolsEvent::ConsoleAPICalled(event) => {
                    for message in &event.messages {
                        // TODO: format log messages differently
                        println!("{}", message);
                    }
                }
                DevtoolsEvent::ExceptionThrown(event) => {
                    println!("{}", event);
                }
            },
            Err(err) => {
                log::info!("\n{}", err);
            }
        };
        log::info!("\n---");
        Ok(())
    }

    fn upgrade_ssl_client(&mut self, sock: TcpStream, _: &Url) -> ws::Result<SslStream<TcpStream>> {
        let mut builder = SslConnector::builder(SslMethod::tls()).map_err(|e| {
            ws::Error::new(
                ws::ErrorKind::Internal,
                format!("Failed to upgrade client to SSL: {}", e),
            )
        })?;
        builder.set_verify(SslVerifyMode::empty());

        let connector = builder.build();
        connector
            .configure()
            .unwrap()
            .use_server_name_indication(false)
            .verify_hostname(false)
            .connect("", sock)
            .map_err(From::from)
    }
}
