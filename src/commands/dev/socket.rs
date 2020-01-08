use std::sync::{Arc, Mutex};
use std::thread;
use std::time::SystemTime;

use chrome_devtools::events::DevtoolsEvent;

use console::style;

use tungstenite::client::AutoStream;
use tungstenite::{connect, Message, WebSocket};

use url::Url;

pub fn listen(session_id: String) -> Result<(), failure::Error> {
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    let socket_url = Url::parse(&socket_url)?;
    let (socket, _) = connect(socket_url)?;

    let socket = Arc::new(Mutex::new(socket));

    let enable_runtime = r#"{
      "id": 1,
      "method": "Runtime.enable"
    }"#;

    {
        let socket = Arc::clone(&socket);
        let mut socket = socket.lock().unwrap();
        socket
            .write_message(Message::Text(enable_runtime.into()))
            .unwrap();
    }

    {
        let socket = Arc::clone(&socket);
        thread::spawn(move || keep_alive(socket));
    }

    loop {
        let msg = socket
            .lock()
            .unwrap()
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

fn keep_alive(socket: Arc<Mutex<WebSocket<AutoStream>>>) {
    let mut keep_alive_time = SystemTime::now();
    let mut id = 2;
    loop {
        let elapsed = keep_alive_time.elapsed().unwrap().as_secs();
        println!("elapsed: {}", elapsed);
        if elapsed >= 5 {
            let keep_alive_message = format!(
                r#"{{
                "id": {},
                "method": "Runtime.getIsolateId"
            }}"#,
                id
            );
            println!("before sending keepalive message");
            {
                let mut socket = socket.lock().unwrap();
                socket
                    .write_message(Message::Text(keep_alive_message.into()))
                    .unwrap();
            }
            println!("after sending keepalive message");
            id += 1;
            keep_alive_time = SystemTime::now();
        }
    }
}
