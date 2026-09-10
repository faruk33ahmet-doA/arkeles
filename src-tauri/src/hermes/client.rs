/*!
Hermes istemcisi — Anayasa madde 18, 19. GÜVEN SINIRI BURADADIR.

19.1  Hermes yalnız 127.0.0.1'e bind edilir → yalnız oraya bağlanırız.
19.2  Paylaşılan token dosyada 0600 izinle tutulur.
19.3  Webview Hermes'e ASLA doğrudan istek atmaz. TOKEN JAVASCRIPT'E HİÇ İNMEZ.
18.1  Hermes bir sürüm ve YETENEK LİSTESİ yayınlar.
18.4  Hermes kapalı olması bir HATA DURUMU DEĞİLDİR.

NEDEN ELDE YAZILMIŞ HTTP: madde 14.1 gereksiz bağımlılığı yasaklıyor.
İhtiyacımız tek bir şey: 127.0.0.1'e tek satırlık bir GET. TLS yok
(loopback), yönlendirme yok, chunked yok. `reqwest` + rustls bunun için
megabaytlarca bağımlılık getirirdi. TcpStream yeterli ve denetlenebilir.
*/

use std::io::{Read, Write};
use std::net::{Shutdown, SocketAddr, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use crate::types::HermesHealth;

/// Anayasa madde 18.4: Hermes kapalıysa uygulama beklemez.
/// 500 ms, madde 34.1'in "etkileşim < 100 ms" bütçesini bozmayacak kadar
/// kısa; yoklama arka planda yapıldığı için kullanıcıyı bekletmez.
const TIMEOUT: Duration = Duration::from_millis(500);

/// Hermes'in dinlediği varsayılan loopback portu.
const DEFAULT_PORT: u16 = 7717;

impl HermesHealth {
    /// Madde 18.3 tablosundaki varsayılan durum.
    pub fn unreachable() -> Self {
        Self {
            reachable: false,
            version: None,
            capabilities: Vec::new(),
        }
    }
}

/// Token dosyasının yolu. Madde 19.2.
fn token_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| {
        PathBuf::from(home).join(".arkeles").join("hermes.token")
    })
}

/// Token'ı okur. Madde 19.3: bu değer ASLA dışarıya (webview'e) verilmez.
///
/// Dosya izinleri 0600 değilse token KULLANILMAZ (madde 19.2). Sebep:
/// dünyaya okunabilir bir sır, sır değildir; sessizce kabul etmek
/// güvenlik kuralını kağıt üzerinde bırakırdı.
fn read_token() -> Option<String> {
    let path = token_path()?;
    let metadata = std::fs::metadata(&path).ok()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode() & 0o777;
        if mode & 0o077 != 0 {
            // Grup veya diğerleri erişebiliyor → reddet.
            return None;
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;

    let raw = std::fs::read_to_string(&path).ok()?;
    let token = raw.trim().to_string();
    if token.is_empty() {
        return None;
    }
    Some(token)
}

/// Hermes portu — ortam değişkeniyle geçersiz kılınabilir.
fn port() -> u16 {
    std::env::var("ARKELES_HERMES_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(DEFAULT_PORT)
}

/// `GET /health` — Anayasa madde 18.1.
///
/// HİÇ HATA DÖNDÜRMEZ. Ulaşılamamak geçerli bir cevaptır (madde 18.4);
/// hata döndürmek arayüzü kırmızı uyarıya zorlardı, bu yasak.
pub fn health() -> HermesHealth {
    match fetch_health() {
        Some(health) => health,
        None => HermesHealth::unreachable(),
    }
}

fn fetch_health() -> Option<HermesHealth> {
    let addr = SocketAddr::from(([127, 0, 0, 1], port()));
    let mut stream = TcpStream::connect_timeout(&addr, TIMEOUT).ok()?;
    stream.set_read_timeout(Some(TIMEOUT)).ok()?;
    stream.set_write_timeout(Some(TIMEOUT)).ok()?;

    // Madde 19.4: Hermes Origin taşıyan istekleri reddeder — biz Origin
    // GÖNDERMEYİZ. Bu, isteğin tarayıcıdan değil çekirdekten geldiğinin işareti.
    let mut request = String::from("GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\n");
    if let Some(token) = read_token() {
        request.push_str(&format!("Authorization: Bearer {token}\r\n"));
    }
    request.push_str("Accept: application/json\r\nConnection: close\r\n\r\n");

    stream.write_all(request.as_bytes()).ok()?;
    stream.flush().ok()?;

    // Yanıt sınırı: kötü davranan bir sunucu belleği doldurmasın.
    // `by_ref` ile ödünç alıyoruz; stream sonra kapatılacak.
    let mut buf = Vec::with_capacity(4096);
    Read::by_ref(&mut stream)
        .take(64 * 1024)
        .read_to_end(&mut buf)
        .ok()?;
    let _ = stream.shutdown(Shutdown::Both);

    let text = String::from_utf8_lossy(&buf);
    let (head, body) = split_http(&text)?;

    if !head.starts_with("HTTP/1.1 200") && !head.starts_with("HTTP/1.0 200") {
        return None;
    }

    parse_health_body(body)
}

fn split_http(text: &str) -> Option<(&str, &str)> {
    let idx = text.find("\r\n\r\n")?;
    Some((&text[..idx], &text[idx + 4..]))
}

/// Gövdeyi ayrıştırır. Anayasa madde 18.2'nin kaynağı `capabilities` listesidir.
///
/// Beklenmeyen alanlar YOK SAYILIR, eksik alanlar varsayılana düşer —
/// Hermes bağımsız gelişecek (madde 18.1), katı ayrıştırma kırılganlık üretir.
fn parse_health_body(body: &str) -> Option<HermesHealth> {
    // Chunked yanıt gelirse ilk parçanın boyut satırını atla.
    let candidate = body.trim_start();
    let json_start = candidate.find('{')?;
    let json_end = candidate.rfind('}')?;
    let raw = &candidate[json_start..=json_end];

    let value: serde_json::Value = serde_json::from_str(raw).ok()?;

    let capabilities = value
        .get("capabilities")
        .and_then(serde_json::Value::as_array)
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default();

    Some(HermesHealth {
        reachable: true,
        version: value
            .get("version")
            .and_then(serde_json::Value::as_str)
            .map(str::to_string),
        capabilities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saglikli_yanit_ayristirilir() {
        let body = r#"{"version":"1.2.0","capabilities":["pdf","telegram"]}"#;
        let health = parse_health_body(body).unwrap();
        assert!(health.reachable);
        assert_eq!(health.version.as_deref(), Some("1.2.0"));
        assert_eq!(health.capabilities, vec!["pdf", "telegram"]);
    }

    #[test]
    fn eksik_alanlar_varsayilana_duser() {
        let health = parse_health_body("{}").unwrap();
        assert!(health.reachable);
        assert!(health.version.is_none());
        assert!(health.capabilities.is_empty());
    }

    #[test]
    fn beklenmeyen_alanlar_yok_sayilir() {
        let body = r#"{"version":"2.0","surprise":42,"capabilities":["ai.summarize"]}"#;
        let health = parse_health_body(body).unwrap();
        assert_eq!(health.capabilities, vec!["ai.summarize"]);
    }

    #[test]
    fn yetenek_listesinde_dizge_olmayanlar_atlanir() {
        let body = r#"{"capabilities":["pdf",5,null,"telegram"]}"#;
        let health = parse_health_body(body).unwrap();
        assert_eq!(health.capabilities, vec!["pdf", "telegram"]);
    }

    #[test]
    fn bozuk_json_none_doner() {
        assert!(parse_health_body("{bozuk").is_none());
        assert!(parse_health_body("").is_none());
    }

    #[test]
    fn chunked_govde_temizlenir() {
        // Boyut satırı + JSON + kapanış
        let body = "34\r\n{\"version\":\"1.0\",\"capabilities\":[]}\r\n0\r\n\r\n";
        let health = parse_health_body(body).unwrap();
        assert_eq!(health.version.as_deref(), Some("1.0"));
    }

    #[test]
    fn http_ayirma() {
        let (head, body) = split_http("HTTP/1.1 200 OK\r\nX: 1\r\n\r\n{}").unwrap();
        assert!(head.starts_with("HTTP/1.1 200"));
        assert_eq!(body, "{}");
    }

    #[test]
    fn ulasilamayan_hermes_hata_degil() {
        // Madde 18.4: sakin bir durum, hata değil.
        let h = HermesHealth::unreachable();
        assert!(!h.reachable);
        assert!(h.capabilities.is_empty());
    }
}
