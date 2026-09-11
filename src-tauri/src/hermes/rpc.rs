/*!
Hermes JSON-RPC istemcisi — Sprint 4 madde 2.

Hermes'in iş gönderme yüzeyi `ws://127.0.0.1:9119/api/ws?token=…` üzerinde
JSON-RPC 2.0'dır (yerel kurulumdan doğrulandı: `web/src/lib/gatewayClient.ts`).

ANAYASA UYUMU:
  19.1  Yalnız 127.0.0.1 — adres sabit kodlu.
  19.3  Token yalnız burada; webview'e HİÇ inmez.
  Sprint 4 madde 2: "genel-purpose keyfi HTTP proxy yapmak değildir."
  Bu yüzden metod adı ÇAĞIRANDAN GELMEZ — allowlist'ten seçilir.

Bağlantı KALICI DEĞİL: her çağrı kendi soketini açar ve kapatır. Kalıcı
bağlantı yeniden bağlanma, kalp atışı ve durum yönetimi getirirdi; Sprint 4
için gereken tek şey "istek gönder, cevabı al". Basit olan doğru olandır.
*/

use std::time::Duration;

use serde_json::{json, Value};
use tungstenite::{client::IntoClientRequest, Message};

use crate::error::{CoreError, CoreResult};
use crate::hermes::contract::{self, Endpoint};

/// İş gönderme bağlantısı için zaman aşımı. Health yoklamasından uzun,
/// çünkü Hermes isteği kaydetmek için bir tur atabilir.
const TIMEOUT: Duration = Duration::from_secs(5);

/// Tek bir JSON-RPC çağrısı yapar ve `result` alanını döner.
///
/// `method` ÇAĞIRANDAN gelir ama çağıran da çekirdektir; arayüzden gelen
/// dizgeler buraya ASLA doğrudan ulaşmaz (bkz. `jobs::submit`).
pub fn call(method: &str, params: Value) -> CoreResult<Value> {
    let token = contract::read_token().ok_or(CoreError::HermesUnauthorized)?;
    let endpoint = Endpoint::resolve();

    // Token sorgu parametresinde: Hermes'in kendi istemcisi de böyle yapıyor.
    // Adres SABİT KODLU — arayüzden gelen hiçbir şey buraya girmiyor.
    let url = format!(
        "ws://127.0.0.1:{}/api/ws?token={}",
        endpoint.port,
        urlencode(&token)
    );

    let request = url
        .into_client_request()
        .map_err(|_| CoreError::HermesUnreachable)?;

    let (mut socket, _response) =
        tungstenite::connect(request).map_err(|_| CoreError::HermesUnreachable)?;

    // Soketi bloke etmeyen hale getirmek yerine okuma zaman aşımı koyuyoruz:
    // cevap gelmezse sonsuza kadar beklemeyiz (madde 34.1).
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_ref() {
        let _ = stream.set_read_timeout(Some(TIMEOUT));
        let _ = stream.set_write_timeout(Some(TIMEOUT));
    }

    let request_id = next_id();
    let payload = json!({
        "jsonrpc": "2.0",
        "id": request_id,
        "method": method,
        "params": params,
    });

    socket
        .send(Message::Text(payload.to_string()))
        .map_err(|_| CoreError::HermesUnreachable)?;

    // Cevabımızı bekle. Hermes aynı sokete olay (event) çerçeveleri de
    // yazabilir; kendi `id`'mizi taşımayan her şeyi atlarız.
    let result = read_response(&mut socket, &request_id);

    let _ = socket.close(None);
    result
}

fn read_response(
    socket: &mut tungstenite::WebSocket<tungstenite::stream::MaybeTlsStream<std::net::TcpStream>>,
    request_id: &str,
) -> CoreResult<Value> {
    // Sınır: kötü davranan bir sunucu bizi sonsuz döngüde tutmasın.
    const MAX_FRAMES: usize = 64;

    for _ in 0..MAX_FRAMES {
        let message = socket.read().map_err(|_| CoreError::HermesTimeout)?;

        let text = match message {
            Message::Text(text) => text,
            Message::Close(_) => return Err(CoreError::HermesUnreachable),
            // Ping/Pong/Binary: bizi ilgilendirmez.
            _ => continue,
        };

        let Ok(value) = serde_json::from_str::<Value>(&text) else {
            // Bozuk çerçeve tüm çağrıyı düşürmez (madde 18.1: Hermes bağımsız gelişir).
            continue;
        };

        if value.get("id").and_then(Value::as_str) != Some(request_id) {
            continue; // bize ait değil (olay çerçevesi veya başka çağrı)
        }

        if let Some(error) = value.get("error") {
            /*
             * Sprint 4 madde 19: "Hermes hata mesajı kullanıcıya raw stack
             * trace olarak gösterilmiyor." Yalnız kısa mesaj alınır; `data`
             * alanı (stack trace taşıyabilir) ATILIR.
             */
            let message = error
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Hermes isteği reddetti")
                .chars()
                .take(200)
                .collect::<String>();
            return Err(CoreError::HermesRejected(message));
        }

        return Ok(value.get("result").cloned().unwrap_or(Value::Null));
    }

    Err(CoreError::HermesTimeout)
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
}
