use chrome_devtools::events::DevtoolsEvent;

use console::style;

use tungstenite::{connect, Message};

use url::Url;

pub fn listen(session_id: String) -> Result<(), failure::Error> {
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    let socket_url = Url::parse(&socket_url)?;
    let (mut socket, _) = connect(socket_url)?;

    let enable_runtime = r#"{
      "id": 2,
      "method": "Runtime.enable"
    }"#;

    socket
        .write_message(Message::Text(enable_runtime.into()))
        .unwrap();

    loop {
        let msg = socket
            .read_message()
            .expect("Error reading message from devtools")
            .into_text()?;
        log::info!("{}", msg);
        let msg: Result<DevtoolsEvent, serde_json::Error> = serde_json::from_str(&msg);
        match msg {
            Ok(msg) => match msg {
                DevtoolsEvent::ConsoleAPICalled(event) => match event.log_type.as_str() {
                    "log" => println!("{}", style(event).blue()),
                    "error" => eprintln!("{}", style(event).red()),
                    _ => println!("unknown console event: {}", event),
                },
                DevtoolsEvent::ExceptionThrown(event) => eprintln!("{}", style(event).bold().red()),
            },
            Err(e) => {
                // this event was not parsed as a DevtoolsEvent
                // TODO: change this to a warn after chrome-devtools-rs is parsing all messages
                log::info!("this event was not parsed as a DevtoolsEvent:\n{}", e);
            }
        }
    }
}
