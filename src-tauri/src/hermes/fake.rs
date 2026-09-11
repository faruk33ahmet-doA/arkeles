/*!
Test için sahte Hermes — YALNIZ `cargo test`.

Gerçek Hermes 0.21.0'da CANLI GÖZLENEN davranışı taklit eder: el sıkışmada
403, olay çerçevesi biçimi, kapanış kodları. Sözleşme UYDURMAZ — her
davranışın kaynağı `rpc.rs` başındaki canlı doğrulama notudur.

Her sunucu tek bir bağlantıyı kabul eder ve verilen senaryoyu oynatır.
Senaryodaki iddialar `finish()` ile teste taşınır.
*/

use std::net::{TcpListener, TcpStream};
use std::thread::JoinHandle;

use serde_json::{json, Value};
use tungstenite::handshake::server::{ErrorResponse, Request, Response};
use tungstenite::http::StatusCode;
use tungstenite::protocol::frame::coding::CloseCode;
use tungstenite::protocol::CloseFrame;
use tungstenite::{Message, WebSocket};

pub type Socket = WebSocket<TcpStream>;

pub struct FakeHermes {
    pub port: u16,
    handle: Option<JoinHandle<()>>,
}

impl FakeHermes {
    /// Senaryo iş parçacığını bekler; içindeki panik testi düşürür.
    pub fn finish(mut self) {
        if let Some(handle) = self.handle.take() {
            handle.join().expect("sahte Hermes senaryosu başarısız");
        }
    }
}

pub fn serve<F>(token: &'static str, script: F) -> FakeHermes
where
    F: FnOnce(&mut Socket) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("boş port");
    let port = listener.local_addr().expect("adres").port();

    let handle = std::thread::spawn(move || {
        let Ok((stream, _)) = listener.accept() else {
            return;
        };
        let expected = format!("token={token}");
        let check = move |request: &Request, response: Response| {
            assert_eq!(
                request.headers().get("user-agent").and_then(|value| value.to_str().ok()),
                Some(crate::hermes::contract::USER_AGENT),
                "ARKELÉS WebSocket istemci kimliği eksik",
            );
            if request.uri().query() == Some(expected.as_str()) {
                Ok(response)
            } else {
                let mut refusal = ErrorResponse::new(None);
                *refusal.status_mut() = StatusCode::FORBIDDEN;
                Err(refusal)
            }
        };
        if let Ok(mut socket) = tungstenite::accept_hdr(stream, check) {
            script(&mut socket);
        }
    });

    FakeHermes { port, handle: Some(handle) }
}

/// İstemcinin gönderdiği bir sonraki JSON çerçeve.
pub fn recv(socket: &mut Socket) -> Value {
    loop {
        if let Message::Text(text) = socket.read().expect("istemci bağlantısı") {
            return serde_json::from_str(&text).expect("istemci JSON göndermeli");
        }
    }
}

pub fn send(socket: &mut Socket, value: Value) {
    send_raw(socket, &value.to_string());
}

pub fn send_raw(socket: &mut Socket, text: &str) {
    socket.send(Message::Text(text.to_string())).expect("gönderim");
}

pub fn reply(socket: &mut Socket, request: &Value, result: Value) {
    send(socket, json!({ "jsonrpc": "2.0", "id": request["id"], "result": result }));
}

pub fn event(socket: &mut Socket, session_id: &str, kind: &str, payload: Value) {
    send(
        socket,
        json!({ "jsonrpc": "2.0", "method": "event",
                "params": { "type": kind, "session_id": session_id, "payload": payload } }),
    );
}

pub fn close_with(socket: &mut Socket, code: u16) {
    let _ = socket.close(Some(CloseFrame { code: CloseCode::from(code), reason: "".into() }));
    let _ = socket.flush();
}
