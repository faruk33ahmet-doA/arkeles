/*!
Hermes istemcisi — Anayasa madde 18, 19. GÜVEN SINIRI BURADADIR.

19.1  Hermes yalnız 127.0.0.1'e bind edilir.
19.2  Paylaşılan token dosyada 0600 izinle tutulur.
19.3  ARKELÉS'in webview'i Hermes'e ASLA doğrudan istek atmaz.
      Tüm çağrılar bu modülden geçer. TOKEN JAVASCRIPT'E HİÇ İNMEZ.
19.4  Hermes, Origin header'ı taşıyan istekleri reddeder.

18.1  Hermes bir sürüm ve YETENEK LİSTESİ yayınlar.
18.2  ARKELÉS, bildirilmeyen yeteneği arayüzde HİÇ göstermez.
18.4  Hermes kapalı olması bir HATA DURUMU DEĞİLDİR.

SPRINT 0 KAPSAMI: `hermes_health` mock döner (reachable: false).
Sprint 0 teslim kriteri: "Henüz Hermes bağlanmayacak."

Sprint 1'de buraya eklenecek:
  - token dosyasını okuma (~/.arkeles/hermes.token, izin kontrolü)
  - 127.0.0.1'e HTTP GET /health, kısa timeout (500 ms)
  - yetenek listesini parse etme
*/

use serde::Serialize;

/// Anayasa madde 18.1 sözleşmesi.
///
/// `ts-rs` ile TypeScript'e üretilmesi Sprint 1'de yapılacak (madde 14.2).
/// Sprint 0'da alan adları frontend'deki `HermesHealth` ile ELLE hizalı
/// tutuldu; hizanın bozulmaması için ikisi de aynı anda değişir.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HermesHealth {
    /// Anayasa madde 18.4: `false` bir hata değil, sakin bir durumdur.
    pub reachable: bool,
    pub version: Option<String>,
    /// Anayasa madde 18.1 — yetenek anahtarları. Ulaşılamıyorsa boş.
    pub capabilities: Vec<String>,
}

impl HermesHealth {
    /// Ulaşılamayan Hermes. Anayasa madde 18.3 tablosundaki varsayılan durum.
    pub fn unreachable() -> Self {
        Self {
            reachable: false,
            version: None,
            capabilities: Vec::new(),
        }
    }
}

/// Hermes sağlık yoklaması.
///
/// SPRINT 0: her zaman `unreachable()` döner. Bu bir hata simülasyonu
/// değil, DÜRÜST bir cevaptır — Hermes gerçekten bağlanmadı.
pub fn health() -> HermesHealth {
    HermesHealth::unreachable()
}
