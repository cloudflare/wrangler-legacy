use chrome_devtools::events::DevtoolsEvent;
use console::style;

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
        let msg = msg.as_text()?;
        log::info!("{}", msg);
        let msg: Result<DevtoolsEvent, serde_json::Error> = serde_json::from_str(msg);
        match msg {
            Ok(msg) => {
                match msg {
                    DevtoolsEvent::ConsoleAPICalled(event) => match event.log_type.as_str() {
                        "log" => println!("{}", style(event).blue()),
                        "error" => eprintln!("{}", style(event).red()),
                        _ => println!("unknown console event: {}", event),
                    },
                    DevtoolsEvent::ExceptionThrown(event) => {
                        eprintln!("{}", style(event).bold().red())
                    }
                }
                Ok(())
            }
            Err(e) => {
                // this event was not parsed as a DevtoolsEvent
                // TODO: change this to a warn after chrome-devtools-rs is parsing all messages
                log::info!("this event was not parsed as a DevtoolsEvent:\n{}", e);
                Ok(())
            }
        }
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
