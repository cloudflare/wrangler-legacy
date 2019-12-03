mod client;
mod messenger;

use client::WsClient;

pub fn listen(session_id: String) -> Result<(), failure::Error> {
    let socket_url = format!("wss://rawhttp.cloudflareworkers.com/inspect/{}", session_id);
    ws::connect(socket_url, |out| WsClient { out }).unwrap();
    Ok(())
}
