/*!
Hermes REST istemcisi — Anayasa madde 19.

19.1  YALNIZ 127.0.0.1. Adres sabit kodlu, arayüzden gelmez.
19.3  Token yalnız burada okunur, webview'e HİÇ inmez.
18.4  Ulaşılamamak HATA DEĞİLDİR.

El yazımı HTTP: loopback'e birkaç GET için `reqwest` + TLS yığını gereksiz
ağırlık olurdu (madde 14.1). Sprint 2'de aynı gerekçeyle yazıldı, burada
gerçek sözleşmeye uyarlandı.
*/

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::time::Duration;

use crate::hermes::contract::{self, Endpoint};

/// Anayasa madde 18.4: Hermes kapalıysa uygulama BEKLEMEZ.
/// Yoklama arka planda yürüdüğü için kullanıcıyı bloke etmez.
const TIMEOUT: Duration = Duration::from_millis(700);

/// Yanıt sınırı: kötü davranan bir sunucu belleği doldurmasın.
const MAX_RESPONSE: u64 = 512 * 1024;

/// `GET <path>` — başarısızlık `None`, hata DEĞİL (madde 18.4).
pub fn get_json(path: &str) -> Option<serde_json::Value> {
    let endpoint = Endpoint::resolve();
    let addr: SocketAddr = ([127, 0, 0, 1], endpoint.port).into();

    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT).ok()?;
    stream.set_read_timeout(Some(TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(TIMEOUT)).ok()?;

    /*
     * Origin GÖNDERİLMEZ (madde 19.4): Origin taşıyan istek tarayıcıdan
     * gelmiş demektir ve Hermes onu reddetmelidir. Bizim isteğimiz
     * çekirdekten geliyor.
     */
    let mut request = format!(
        "GET {path} HTTP/1.1\r\nHost: {}\r\nAccept: application/json\r\nConnection: close\r\n",
        endpoint.http_authority()
    );
    if let Some(token) = contract::read_token() {
        request.push_str(&format!("{}: {token}\r\n", contract::SESSION_HEADER));
    }
    request.push_str("\r\n");

    stream.write_all(request.as_bytes()).ok()?;
    stream.flush().ok()?;

    let mut buf = Vec::with_capacity(8192);
    Read::by_ref(&mut stream)
        .take(MAX_RESPONSE)
        .read_to_end(&mut buf)
        .ok()?;
    let _ = stream.shutdown(Shutdown::Both);

    let text = String::from_utf8_lossy(&buf);
    let (head, body) = split_http(&text)?;

    // 401/403: token yanlış ya da yok. Bu bir ARIZA DEĞİL, yapılandırma
    // durumudur — çağıran onu `reachable: false` olarak gösterir.
    if !head.starts_with("HTTP/1.1 200") && !head.starts_with("HTTP/1.0 200") {
        return None;
    }

    parse_json_body(body)
}

fn split_http(text: &str) -> Option<(&str, &str)> {
    let idx = text.find("\r\n\r\n")?;
    Some((&text[..idx], &text[idx + 4..]))
}

/// Gövdeyi ayrıştırır. Chunked yanıtta boyut satırlarını atlar.
///
/// Katı ayrıştırma YAPILMAZ: Hermes bağımsız gelişiyor, beklenmeyen alanlar
/// yok sayılır (madde 18.1 sözleşme kırılganlığı).
pub fn parse_json_body(body: &str) -> Option<serde_json::Value> {
    let candidate = body.trim_start();

    // Nesne veya dizi — ilk açılış ve son kapanış arası.
    let start = candidate.find(['{', '['])?;
    let open = candidate.as_bytes()[start];
    let close = if open == b'{' { '}' } else { ']' };
    let end = candidate.rfind(close)?;
    if end < start {
        return None;
    }

    serde_json::from_str(&candidate[start..=end]).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn http_basligi_ve_govdesi_ayrilir() {
        let (head, body) = split_http("HTTP/1.1 200 OK\r\nX: 1\r\n\r\n{\"a\":1}").unwrap();
        assert!(head.starts_with("HTTP/1.1 200"));
        assert_eq!(body, "{\"a\":1}");
    }

    #[test]
    fn nesne_govdesi_ayristirilir() {
        let v = parse_json_body(r#"{"ok":true}"#).unwrap();
        assert_eq!(v["ok"], serde_json::json!(true));
    }

    #[test]
    fn dizi_govdesi_ayristirilir() {
        let v = parse_json_body(r#"[{"id":"a"}]"#).unwrap();
        assert_eq!(v[0]["id"], serde_json::json!("a"));
    }

    #[test]
    fn chunked_govde_temizlenir() {
        let v = parse_json_body("1a\r\n{\"ok\":true}\r\n0\r\n\r\n").unwrap();
        assert_eq!(v["ok"], serde_json::json!(true));
    }

    #[test]
    fn bozuk_govde_none_doner() {
        // Sprint 4 madde 20: malformed response.
        assert!(parse_json_body("").is_none());
        assert!(parse_json_body("{bozuk").is_none());
        assert!(parse_json_body("düz metin").is_none());
        assert!(parse_json_body("}{" ).is_none());
    }

    #[test]
    fn cok_buyuk_govde_siniri_sabit() {
        assert_eq!(MAX_RESPONSE, 512 * 1024);
    }
}
