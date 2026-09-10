/*!
Hermes IPC komutları — Anayasa madde 12, 18, 19.3.

Webview yalnız komut ADI gönderir; adres, token ve ağ işi Rust'ta kalır.
*/

use crate::types::HermesHealth;

/// Anayasa madde 18.1: sürüm + yetenek listesi.
///
/// HATA DÖNDÜRMEZ. Madde 18.4: Hermes'e ulaşılamaması bir hata durumu
/// değildir; `reachable: false` geçerli bir cevaptır. Hata döndürmek
/// arayüzü kırmızı uyarıya zorlardı — bu yasak.
///
/// `async` çünkü ağ işi 500 ms'e kadar sürebilir; IPC iş parçacığını
/// bloke etmemek için Tauri'nin async çalıştırıcısında yürür.
#[tauri::command]
pub async fn hermes_health() -> HermesHealth {
    tauri::async_runtime::spawn_blocking(crate::hermes::health)
        .await
        .unwrap_or_else(|_| HermesHealth::unreachable())
}
