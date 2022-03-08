#![allow(unused_imports)]

use std::env;

use futures_util::{
    future, pin_mut,
    stream::{SplitSink, SplitStream},
    StreamExt,
};
use tauri::{AppHandle, Manager, Wry};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
};
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};
use url::Url;
use uuid::Uuid;

// Project Imports
use super::accessibility_messages;
use super::websocket_message::WebsocketMessage;

pub struct WebsocketClient {
    pub url: Url,
    pub client_id: Uuid,
    pub tauri_app_handle: AppHandle<Wry>,
}

impl WebsocketClient {
    pub async fn new(url_string: &str, app_handle: AppHandle<Wry>) -> Self {
        let url = url::Url::parse(&url_string).expect("No valid URL path provided.");
        let client_id = Uuid::new_v4();
        let tauri_app_handle = app_handle.clone();

        let ws_stream = Self::connect(&url).await;
        let (stream_write, stream_read) = ws_stream.split();
        let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();

        // Attempt connection to server
        let payload: accessibility_messages::models::Connect =
            accessibility_messages::models::Connect { connect: true };
        let ws_message = WebsocketMessage::from_request(
            accessibility_messages::types::Request::Connect(payload),
            client_id,
        );
        stdin_tx
            .unbounded_send(tungstenite::Message::binary(
                serde_json::to_vec(&ws_message).unwrap(),
            ))
            .unwrap();

        // Setup stdin stream to send messages to server
        // The following code is commented out because it blocks prints to stdout 🤔
        // tokio::spawn(Self::read_stdin(stdin_tx));

        // Setup stdout stream to receive messages from server
        let stdin_to_ws = stdin_rx.map(Ok).forward(stream_write);
        let ws_to_stdout = {
            stream_read.for_each(|message| async {
                let data = message.unwrap().into_text().unwrap();
                let parsed_msg: WebsocketMessage<accessibility_messages::Message> =
                    serde_json::from_str(&data.to_string()).unwrap();

                // DEBUG
                let print_str = serde_json::to_string(&parsed_msg).unwrap();
                tokio::io::stdout()
                    .write_all(&print_str.as_bytes())
                    .await
                    .unwrap();

                app_handle.emit_all("ax-messages", parsed_msg).unwrap();
            })
        };

        pin_mut!(stdin_to_ws, ws_to_stdout);
        future::select(stdin_to_ws, ws_to_stdout).await;

        Self {
            url,
            client_id,
            tauri_app_handle: app_handle.clone(),
        }
    }

    async fn connect(url: &Url) -> WebSocketStream<MaybeTlsStream<TcpStream>> {
        let (ws_stream, _) = tokio_tungstenite::connect_async(url)
            .await
            .expect("Failed to connect");
        println!("WebSocket handshake has been successfully completed");

        return ws_stream;
    }

    // Our helper method which will read data from stdin and send it along the
    // sender provided.
    #[allow(dead_code)]
    async fn read_stdin(tx: futures_channel::mpsc::UnboundedSender<tungstenite::Message>) {
        let mut stdin = tokio::io::stdin();
        loop {
            let mut buf = vec![0; 1024];
            let n = match stdin.read(&mut buf).await {
                Err(_) | Ok(0) => break,
                Ok(n) => n,
            };
            buf.truncate(n);
            tx.unbounded_send(tungstenite::Message::binary(buf))
                .unwrap();
        }
    }
}
