/*!
ARKELÉS çekirdeği.

Anayasa madde 7.3 & 12: Çekirdek "beyin" değildir — beyin Hermes'tir.
Çekirdeğin işi kas işidir: dosya okumak, index tutmak, Hermes'e proxy olmak.
Burada hiçbir yargı, analiz veya üretim yapılmaz (madde 8.2).

Modül haritası:
  config   → vault yolu ve uygulama yapılandırması (madde 17)
  vault    → markdown okuma ve mekanik yazma (madde 8.1, 20)
  index    → SQLite türetilmiş veri (madde 9, 15)
  watcher  → dosya izleyici, 200 ms debounce (madde 20.4)
  hermes   → Hermes istemcisi; güven sınırı burada (madde 18, 19)
  types    → webview'e çıkan tipler, ts-rs ile üretilir (madde 14.2)
  ipc      → webview'e açılan komut yüzeyi (madde 12)
*/

mod config;
mod error;
mod hermes;
mod index;
mod ipc;
mod types;
mod vault;
mod watcher;

pub use error::{CoreError, CoreResult};

use std::path::Path;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use tauri::Manager;

/// Uygulama durumu — Tauri tarafından yönetilir, komutlara enjekte edilir.
///
/// Anayasa madde 17.2: uygulama yapılandırması vault'a yazılmaz.
pub struct AppState {
    pub config: config::Config,
    pub index: index::IndexHandle,
    /// Aktif izleyici. Vault değişince düşürülüp yenisi kurulur (madde 20.4).
    watcher: Mutex<Option<watcher::WatcherHandle>>,
    /// Tam tarama sürüyor mu? Arayüz bunu sakin bir durum olarak gösterir.
    rebuilding: AtomicBool,
}

impl AppState {
    pub fn is_rebuilding(&self) -> bool {
        self.rebuilding.load(Ordering::Relaxed)
    }

    pub fn set_rebuilding(&self, value: bool) {
        self.rebuilding.store(value, Ordering::Relaxed);
    }

    /// İzleyiciyi yeniden kurar. Eski handle düşürülünce izleme durur.
    pub fn restart_watcher(&self, app: &tauri::AppHandle, vault_root: &Path) {
        let mut slot = self.watcher.lock().unwrap_or_else(|p| p.into_inner());
        *slot = None; // önce eskisini düşür
        *slot = watcher::start(app.clone(), vault_root.to_path_buf());
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Anayasa madde 17.3: vault yolu SABİT KODLANMAZ.
            let config = config::Config::load(app.handle())?;

            // Anayasa madde 15.3: index app-data dizininde, vault'un İÇİNDE DEĞİL.
            let index = index::IndexHandle::open(app.handle())?;

            let vault_path = config.vault_path();

            app.manage(AppState {
                config,
                index,
                watcher: Mutex::new(None),
                rebuilding: AtomicBool::new(false),
            });

            /*
             * Vault yapılandırılmışsa açılışta tarama + izleme başlar.
             *
             * Tarama ARKA PLANDA yapılır: madde 21.2 splash ekranı yasaklıyor,
             * madde 21.3 "ilk görünen şey son bilinen veridir" diyor. Index
             * zaten diskte olduğu için pencere anında açılır, tazeleme arkada
             * yürür ve bitince `vault:changed` ile arayüz kendini yeniler.
             */
            if let Some(root) = vault_path {
                if vault::reader::is_readable(&root) {
                    let handle = app.handle().clone();
                    let scan_root = root.clone();

                    std::thread::spawn(move || {
                        let Some(state) = handle.try_state::<AppState>() else {
                            return;
                        };
                        state.set_rebuilding(true);
                        if let Ok(report) = state.index.full_scan(&scan_root) {
                            state.index.record_scan_ms(report.duration_ms);
                        }
                        state.set_rebuilding(false);
                        use tauri::Emitter;
                        let _ = handle.emit(watcher::CHANGED_EVENT, ());
                    });

                    let state = app.state::<AppState>();
                    state.restart_watcher(app.handle(), &root);
                }
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::vault_cmds::vault_status,
            ipc::vault_cmds::select_vault,
            ipc::vault_cmds::list_today,
            ipc::vault_cmds::dashboard_view,
            ipc::vault_cmds::search_notes,
            ipc::index_cmds::index_status,
            ipc::index_cmds::rebuild_index,
            ipc::hermes_cmds::hermes_health,
        ])
        .run(tauri::generate_context!())
        .expect("ARKELÉS başlatılamadı");
}
