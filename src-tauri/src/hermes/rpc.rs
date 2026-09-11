/*!
Hermes JSON-RPC kanalı — Sprint 5'te GERÇEK sözleşmeye göre yeniden yazıldı.

CANLI DOĞRULAMA (Hermes 0.21.0, `hermes serve`, 127.0.0.1:9119):
  - adres `ws://127.0.0.1:9119/api/ws?token=…`, JSON-RPC 2.0
  - yanlış veya eksik token → el sıkışma HTTP 403 ile REDDEDİLİR
  - bağlantı sonradan kapatılırsa kapanış kodu: 4401 kimlik, 4403 yasak,
    4408 istemci zaman aşımı, 1011 iç hata
  - olay çerçevesi: {"jsonrpc":"2.0","method":"event",
                     "params":{"type","session_id","payload"}}
  - Hermes oturumu BAĞLANTIYA bağlıdır: soket düşerse oturum kısa sürede
    sahipsiz sayılıp toplanır. Bu yüzden bir tur boyunca kanal AÇIK kalır
    (Sprint 4'teki "her çağrı kendi soketi" modeli gerçek Hermes'te çalışmaz).

ANAYASA UYUMU:
  19.1  Yalnız 127.0.0.1 — adres sabit kodlu.
  19.3  Token yalnız burada; webview'e HİÇ inmez. Origin başlığı GÖNDERİLMEZ.
  Madde 19: hata mesajı kısa ve tek satır; `data` alanı (iz) ATILIR.

HATA SINIFLARI (Sprint 5 madde 4):
  hermes_unreachable         bağlantı kurulamadı / koptu
  hermes_unauthorized        token yok, el sıkışma 401/403, kapanış 4401/4403
  hermes_timeout             süre doldu
  hermes_rejected            Hermes JSON-RPC hatası döndü
  hermes_malformed_response  çerçeve JSON değil / beklenen alan yok
*/

use std::collections::VecDeque;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpStream};
use std::time::{Duration, Instant};

use serde_json::{json, Value};
use tungstenite::handshake::{HandshakeError, HandshakeRole};
use tungstenite::protocol::CloseFrame;
use tungstenite::{client::IntoClientRequest, Message, WebSocket};

use crate::error::{CoreError, CoreResult};
use crate::hermes::contract::{self, Endpoint};

/// Tek bir çağrının cevabı için süre. Tur beklemesi ayrıdır (`turn.rs`).
pub const CALL_TIMEOUT: Duration = Duration::from_secs(5);

/// Cevap beklerken tamponlanan olay sınırı — kötü davranan sunucu belleği
/// şişiremesin.
const MAX_BUFFERED_EVENTS: usize = 256;

/// Hermes'e açık bir kanal. Düşürülünce soket kapanır.
pub struct Channel {
    socket: WebSocket<TcpStream>,
    /// Bir çağrının cevabı beklenirken gelen olaylar. Tur tamamlanma olayı
    /// cevaptan ÖNCE gelebilir; kaybolmaması için saklanır.
    events: VecDeque<Value>,
    call_timeout: Duration,
}

impl Channel {
    pub fn open() -> CoreResult<Self> {
        let token = contract::read_token().ok_or(CoreError::HermesUnauthorized)?;
        Self::open_at(Endpoint::resolve().port, &token, CALL_TIMEOUT)
    }

    pub(crate) fn open_at(port: u16, token: &str, call_timeout: Duration) -> CoreResult<Self> {
        let address = SocketAddr::from(([127, 0, 0, 1], port));
        let stream = TcpStream::connect_timeout(&address, call_timeout).map_err(|err| {
            match err.kind() {
                ErrorKind::TimedOut | ErrorKind::WouldBlock => CoreError::HermesTimeout,
                _ => CoreError::HermesUnreachable,
            }
        })?;
        // Madde 34.1: hiçbir okuma sonsuza kadar beklemez.
        let _ = stream.set_read_timeout(Some(call_timeout));
        let _ = stream.set_write_timeout(Some(call_timeout));
        let _ = stream.set_nodelay(true);

        // Token sorgu parametresinde: Hermes'in kendi istemcisi de böyle yapıyor.
        let url = format!("ws://127.0.0.1:{port}/api/ws?token={}", urlencode(token));
        let request = url
            .into_client_request()
            .map_err(|_| CoreError::HermesUnreachable)?;

        let (socket, _response) =
            tungstenite::client::client(request, stream).map_err(classify_handshake)?;

        Ok(Self { socket, events: VecDeque::new(), call_timeout })
    }

    /// Tek bir JSON-RPC çağrısı; `result` alanını döner.
    ///
    /// `method` çekirdekten gelir (turn.rs'teki sabitler); arayüzden gelen
    /// hiçbir dizge buraya ulaşmaz.
    pub fn call(&mut self, method: &str, params: Value) -> CoreResult<Value> {
        let id = next_id();
        let frame = json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params });
        self.socket
            .send(Message::Text(frame.to_string()))
            .map_err(classify)?;

        let deadline = Instant::now() + self.call_timeout;
        loop {
            let value = self.next_json(deadline)?;
            if is_event(&value) {
                if self.events.len() >= MAX_BUFFERED_EVENTS {
                    self.events.pop_front();
                }
                self.events.push_back(value);
                continue;
            }
            if value.get("id").and_then(Value::as_str) != Some(id.as_str()) {
                continue; // başka bir çağrının cevabı
            }
            return response_of(&value);
        }
    }

    /// Belirli oturumun belirli olayını bekler ve `payload`'ını döner.
    pub fn wait_event(&mut self, session_id: &str, kind: &str, timeout: Duration) -> CoreResult<Value> {
        if let Some(position) = self
            .events
            .iter()
            .position(|event| event_matches(event, session_id, kind))
        {
            let event = self.events.remove(position).unwrap_or(Value::Null);
            return Ok(payload_of(&event));
        }
        self.events.clear();

        let deadline = Instant::now() + timeout;
        loop {
            let value = self.next_json(deadline)?;
            if event_matches(&value, session_id, kind) {
                return Ok(payload_of(&value));
            }
        }
    }

    fn next_json(&mut self, deadline: Instant) -> CoreResult<Value> {
        loop {
            if Instant::now() >= deadline {
                return Err(CoreError::HermesTimeout);
            }
            match self.socket.read() {
                Ok(Message::Text(text)) => {
                    return serde_json::from_str(&text).map_err(|_| CoreError::HermesMalformed)
                }
                Ok(Message::Close(frame)) => return Err(classify_close(frame.as_ref())),
                // Ping/Pong/Binary: bizi ilgilendirmez (pong'u kütüphane yollar).
                Ok(_) => continue,
                // Okuma süresi doldu: genel süre dolmadıysa beklemeye devam.
                Err(tungstenite::Error::Io(err))
                    if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) =>
                {
                    continue
                }
                Err(err) => return Err(classify(err)),
            }
        }
    }
}

impl Drop for Channel {
    fn drop(&mut self) {
        let _ = self.socket.close(None);
        let _ = self.socket.flush();
    }
}

/// Kısa ömürlü tek çağrı — tur dışı istekler için (ör. iptal).
pub fn call(method: &str, params: Value) -> CoreResult<Value> {
    Channel::open()?.call(method, params)
}

/// Kullanıcıya gösterilebilir kısa metin: TEK SATIR, en çok 200 karakter.
/// Çok satırlı metin iz (stack trace) taşıyabilir — madde 19.
pub fn short_text(value: Option<&Value>, fallback: &str) -> String {
    value
        .and_then(Value::as_str)
        .and_then(|text| text.lines().map(str::trim).find(|line| !line.is_empty()))
        .unwrap_or(fallback)
        .chars()
        .take(200)
        .collect()
}

fn response_of(value: &Value) -> CoreResult<Value> {
    if let Some(error) = value.get("error") {
        return Err(CoreError::HermesRejected(short_text(
            error.get("message"),
            "Hermes isteği reddetti.",
        )));
    }
    value.get("result").cloned().ok_or(CoreError::HermesMalformed)
}

fn is_event(value: &Value) -> bool {
    value.get("method").and_then(Value::as_str) == Some("event")
}

fn event_matches(value: &Value, session_id: &str, kind: &str) -> bool {
    let Some(params) = value.get("params").filter(|_| is_event(value)) else {
        return false;
    };
    params.get("type").and_then(Value::as_str) == Some(kind)
        && params.get("session_id").and_then(Value::as_str) == Some(session_id)
}

fn payload_of(event: &Value) -> Value {
    event
        .get("params")
        .and_then(|params| params.get("payload"))
        .cloned()
        .unwrap_or(Value::Null)
}

fn classify_handshake<R: HandshakeRole>(err: HandshakeError<R>) -> CoreError {
    match err {
        HandshakeError::Interrupted(_) => CoreError::HermesTimeout,
        HandshakeError::Failure(err) => classify(err),
    }
}

fn classify(err: tungstenite::Error) -> CoreError {
    use tungstenite::Error as E;
    match err {
        // Canlı: yanlış/eksik token → 403. 401 de aynı anlamda ele alınır.
        E::Http(response) => match response.status().as_u16() {
            401 | 403 => CoreError::HermesUnauthorized,
            _ => CoreError::HermesRejected("Hermes bağlantıyı reddetti.".into()),
        },
        E::Io(err) if matches!(err.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
            CoreError::HermesTimeout
        }
        E::Io(_) | E::ConnectionClosed | E::AlreadyClosed => CoreError::HermesUnreachable,
        // Protokol, UTF-8, kapasite, HTTP biçimi: karşı taraf beklenmeyen şey yolladı.
        _ => CoreError::HermesMalformed,
    }
}

fn classify_close(frame: Option<&CloseFrame<'_>>) -> CoreError {
    match frame.map(|frame| u16::from(frame.code)) {
        Some(4401 | 4403) => CoreError::HermesUnauthorized,
        Some(4408) => CoreError::HermesTimeout,
        Some(4400) => CoreError::HermesRejected("Hermes isteği reddetti.".into()),
        _ => CoreError::HermesUnreachable,
    }
}

/// Sorgu parametresi için minimal yüzde kodlama.
///
/// Token base64url alfabesinde olsa da varsayım yapmıyoruz: alfanümerik ve
/// `-._~` dışındaki her şey kodlanır.
fn urlencode(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                out.push(byte as char)
            }
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

/// Çağrı kimliği. Hermes'in kendi istemcisi `w` öneki kullanıyor; biz `a`.
fn next_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    format!("a{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hermes::fake;

    const TOKEN: &str = "dogru-token";

    fn open(port: u16) -> CoreResult<Channel> {
        Channel::open_at(port, TOKEN, Duration::from_millis(400))
    }

    #[test]
    fn urlencode_ozel_karakterleri_kacirir() {
        assert_eq!(urlencode("abc-123_x.y~z"), "abc-123_x.y~z");
        assert_eq!(urlencode("a b"), "a%20b");
        assert_eq!(urlencode("a&b=c"), "a%26b%3Dc");
        // Token'a enjeksiyon denemesi URL'i bozamaz.
        assert_eq!(urlencode("x?y#z"), "x%3Fy%23z");
    }

    #[test]
    fn cagri_kimlikleri_tekil_ve_onekli() {
        let a = next_id();
        let b = next_id();
        assert_ne!(a, b);
        assert!(a.starts_with('a'));
    }

    #[test]
    fn dogru_token_ile_cagri_sonuc_doner() {
        let server = fake::serve(TOKEN, |ws| {
            let request = fake::recv(ws);
            assert_eq!(request["method"], "ping");
            fake::reply(ws, &request, json!({"pong": true}));
        });
        let result = open(server.port).unwrap().call("ping", json!({})).unwrap();
        assert_eq!(result, json!({"pong": true}));
        server.finish();
    }

    #[test]
    fn yanlis_token_yetkisiz_siniflanir() {
        // Canlı gözlem: Hermes el sıkışmayı HTTP 403 ile reddediyor.
        let server = fake::serve(TOKEN, |_| unreachable!("el sıkışma kabul edilmemeli"));
        let err = Channel::open_at(server.port, "yanlis", Duration::from_millis(400)).err();
        assert_eq!(err.map(|e| e.code()), Some("hermes_unauthorized"));
        server.finish();
    }

    #[test]
    fn kapanis_4401_yetkisiz_siniflanir() {
        let server = fake::serve(TOKEN, |ws| {
            let _ = fake::recv(ws);
            fake::close_with(ws, 4401);
        });
        let err = open(server.port).unwrap().call("ping", json!({})).unwrap_err();
        assert_eq!(err.code(), "hermes_unauthorized");
        server.finish();
    }

    #[test]
    fn ulasilamayan_port_unreachable() {
        let port = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap().port()
        }; // dinleyici düştü: port kapalı
        assert_eq!(open(port).err().map(|e| e.code()), Some("hermes_unreachable"));
    }

    #[test]
    fn sessiz_sunucu_timeout() {
        let server = fake::serve(TOKEN, |ws| {
            let _ = fake::recv(ws);
            std::thread::sleep(Duration::from_millis(900)); // cevap YOK
        });
        let err = open(server.port).unwrap().call("ping", json!({})).unwrap_err();
        assert_eq!(err.code(), "hermes_timeout");
        server.finish();
    }

    #[test]
    fn json_olmayan_cerceve_malformed() {
        let server = fake::serve(TOKEN, |ws| {
            let _ = fake::recv(ws);
            fake::send_raw(ws, "<html>bu JSON değil</html>");
        });
        let err = open(server.port).unwrap().call("ping", json!({})).unwrap_err();
        assert_eq!(err.code(), "hermes_malformed_response");
        server.finish();
    }

    #[test]
    fn sonucsuz_cevap_malformed() {
        let server = fake::serve(TOKEN, |ws| {
            let request = fake::recv(ws);
            fake::send(ws, json!({"jsonrpc": "2.0", "id": request["id"]}));
        });
        let err = open(server.port).unwrap().call("ping", json!({})).unwrap_err();
        assert_eq!(err.code(), "hermes_malformed_response");
        server.finish();
    }

    #[test]
    fn hermes_hatasi_rejected_tek_satir_kisa() {
        // Madde 19: çok satırlı iz kullanıcıya SIZMAZ; `data` atılır.
        let long = format!("İstek reddedildi\n{}", "Traceback (most recent call last):\n".repeat(40));
        let server = fake::serve(TOKEN, move |ws| {
            let request = fake::recv(ws);
            fake::send(
                ws,
                json!({"jsonrpc": "2.0", "id": request["id"],
                       "error": {"code": 4090, "message": long, "data": {"trace": "gizli"}}}),
            );
        });
        let err = open(server.port).unwrap().call("ping", json!({})).unwrap_err();
        assert_eq!(err.code(), "hermes_rejected");
        assert_eq!(err.to_string(), "İstek reddedildi");
        server.finish();
    }

    #[test]
    fn cevap_beklerken_gelen_olay_kaybolmaz() {
        let server = fake::serve(TOKEN, |ws| {
            let request = fake::recv(ws);
            fake::event(ws, "s1", "message.complete", json!({"status": "complete"}));
            fake::reply(ws, &request, json!({}));
        });
        let mut channel = open(server.port).unwrap();
        channel.call("prompt.submit", json!({})).unwrap();
        let payload = channel
            .wait_event("s1", "message.complete", Duration::from_millis(400))
            .unwrap();
        assert_eq!(payload["status"], "complete");
        server.finish();
    }

    #[test]
    fn kisa_metin_ilk_bos_olmayan_satir() {
        assert_eq!(short_text(Some(&json!("\n  ilk\nikinci")), "x"), "ilk");
        assert_eq!(short_text(Some(&json!("")), "yedek"), "yedek");
        assert_eq!(short_text(Some(&json!(42)), "yedek"), "yedek");
        assert_eq!(short_text(None, "yedek"), "yedek");
        assert_eq!(short_text(Some(&json!("x".repeat(500))), "").chars().count(), 200);
    }
}
