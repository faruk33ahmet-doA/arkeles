/*!
Hermes IPC komutları — Anayasa madde 12, 18, 19.3.

Webview yalnız komut ADI gönderir; adres, token ve ağ işi Rust'ta kalır.
*/

use crate::hermes::{self, HermesHealth};

/// Anayasa madde 18.1: sürüm + yetenek listesi.
///
/// Bu komut HATA DÖNDÜRMEZ. Anayasa madde 18.4: Hermes'e ulaşılamaması
/// bir hata durumu değildir; `reachable: false` geçerli bir cevaptır.
/// Hata döndürmek arayüzü kırmızı uyarıya zorlardı — bu yasak.
#[tauri::command]
pub fn hermes_health() -> HermesHealth {
    hermes::health()
}
