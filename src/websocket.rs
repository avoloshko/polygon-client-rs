//! WebSocket client for [polygon.io](https://polygon.io).
//!
//! # Authentication
//!
//! Use an [API key](https://polygon.io/dashboard/api-keys) to authenticate.
//! This can be provided through the `auth_key` parameter to
//! [`WebSocketClient::new()`] or through the `POLYGON_AUTH_KEY` environment variable.
//!
//! # Example
//!
//! ```
//! use polygon_client::websocket::{STOCKS_CLUSTER, WebSocketClient};
//!
//! #[tokio::main]
//! async fn main() {
//!     let mut client = WebSocketClient::new(STOCKS_CLUSTER, None);
//!     let res = client.receive();
//!     let msg_text = res.unwrap().into_text().unwrap();
//!     println!("msg: {}", msg_text);
//! }
//! ```
use std::env;
use url::Url;

use serde;
use serde::Deserialize;

use tungstenite::{Message, WebSocket};
use tungstenite::client::connect;

pub const STOCKS_CLUSTER: &str = "stocks";
pub const FOREX_CLUSTER: &str = "forex";
pub const CRYPTO_CLUSTER: &str = "crypto";

#[derive(Clone, Deserialize, Debug)]
struct ConnectedMessage {
    pub ev: String,
    pub status: String,
    pub message: String
}

pub struct WebSocketClient {
    pub auth_key: String,
    websocket: WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>
}

static DEFAULT_WS_HOST: &str = "wss://socket.polygon.io";

impl WebSocketClient {
    pub fn new(cluster: &str, auth_key: Option<&str>) -> Self {
        let auth_key_actual = match auth_key {
            Some(v) => String::from(v),
            _ => match env::var("POLYGON_AUTH_KEY") {
                Ok(v) => String::from(v),
                _ => panic!("POLYGON_AUTH_KEY not set"),
            },
        };

        let url_str = format!("{}/{}", DEFAULT_WS_HOST, cluster);
        let url = Url::parse(&url_str).unwrap();
        let sock = connect(url)
            .expect("failed to connect")
            .0;

        let mut wsc = WebSocketClient { 
            auth_key: auth_key_actual,
            websocket: sock
        };

        wsc._authenticate();

        wsc
    }

    fn _authenticate(&mut self) {
        let str = format!("{{\"action\":\"auth\",\"params\":\"{}\"}}", self.auth_key);
        self.websocket.write_message(Message::Text(str.into())).expect("failed to authenticate");
    }

    pub fn subscribe(&mut self, params: &Vec<&str>) {
        let str = params.join(",");
        let msg = format!("{{\"action\":\"subscribe\",\"params\":\"{}\"}}", &str);
        self.websocket.write_message(Message::Text(msg.into())).expect("failed to subscribe");
    }
    
    pub fn unsubscribe(&mut self, params: &Vec<&str>) {
        let str = params.join(",");
        let msg = format!("{{\"action\":\"unsubscribe\",\"params\":\"{}\"}}", &str);
        self.websocket.write_message(Message::Text(msg.into())).expect("failed to unsubscribe");
    }

    pub fn receive(&mut self) -> tungstenite::error::Result<Message> {
        self.websocket.read_message()
    }
}

#[cfg(test)]
mod tests {
    use crate::websocket::ConnectedMessage;
    use crate::websocket::WebSocketClient;
    use crate::websocket::STOCKS_CLUSTER;

    #[test]
    fn test_subscribe() {
        let mut socket = WebSocketClient::new(STOCKS_CLUSTER, None);
        let params = vec! ["T.MSFT"];
        socket.subscribe(&params);
    }

    #[test]
    fn test_receive() {
        let mut socket = WebSocketClient::new(STOCKS_CLUSTER, None);
        let res = socket.receive();
        assert_eq!(res.is_ok(), true);
        let msg = res.unwrap();
        assert_eq!(msg.is_text(), true);
        let msg_str = msg.into_text().unwrap();
        let messages: Vec<ConnectedMessage> = serde_json::from_str(&msg_str).unwrap();
        let connected = messages.first().unwrap();
        assert_eq!(connected.ev, "status");
        assert_eq!(connected.status, "connected");
    }
}