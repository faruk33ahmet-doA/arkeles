/*!
ARKELÉS çekirdeği.

Anayasa madde 7.3 & 12: Çekirdek "beyin" değildir — beyin Hermes'tir.
Çekirdeğin işi kas işidir: dosya okumak, index tutmak, Hermes'e proxy olmak.
Burada hiçbir yargı, analiz veya üretim yapılmaz (madde 8.2).

Modül haritası:
  config  → vault yolu ve uygulama yapılandırması (madde 17)
  vault   → markdown okuma ve mekanik yazma (madde 8.1, 20)
  index   → SQLite türetilmiş veri (madde 9, 15)
  hermes  → Hermes istemcisi; güven sınırı burada (madde 18, 19)
  ipc     → webview'e açılan komut yüzeyi (madde 12, 14.2)
*/

mod config;
mod error;
mod hermes;
mod index;
mod ipc;
mod vault;

pub use error::{CoreError, CoreResult};

use tauri::Manager;

/// Uygulama durumu — Tauri tarafından yönetilir, komutlara enjekte edilir.
///
/// Anayasa madde 17.2: uygulama yapılandırması vault'a yazılmaz.
pub struct AppState {
    pub config: config::Config,
    pub index: index::IndexHandle,
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            // Anayasa madde 17.3: vault yolu SABİT KODLANMAZ.
            // Sprint 0'da yapılandırma boş başlar; ilk açılış akışı Sprint 1.
            let config = config::Config::load(app.handle())?;

            // Anayasa madde 15.3: index app-data dizininde, vault'un İÇİNDE DEĞİL.
            let index = index::IndexHandle::open(app.handle())?;

            app.manage(AppState { config, index });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::vault_cmds::vault_status,
            ipc::index_cmds::index_status,
            ipc::hermes_cmds::hermes_health,
        ])
        .run(tauri::generate_context!())
        .expect("ARKELÉS başlatılamadı");
}
