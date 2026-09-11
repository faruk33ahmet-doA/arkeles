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
/// İş kuyruğu değiştiğinde arayüze yayınlanan olay — Sprint 4.
pub const JOBS_CHANGED_EVENT: &str = "hermes:jobs-changed";

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
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Anayasa madde 17.3: vault yolu SABİT KODLANMAZ.
            let config = config::Config::load(app.handle())?;

            // Anayasa madde 15.3: index app-data dizininde, vault'un İÇİNDE DEĞİL.
            let index = index::IndexHandle::open(app.handle())?;
            index.recover_orphan_jobs();

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

            /*
             * İş sürücüsü — Sprint 4 madde 6, 21; Sprint 5.
             *
             * Hermes YOKLANMAZ. Kuyruktaki her iş için ayrı bir iş parçacığı
             * turu açar ve Hermes'in kendi olay akışını (`message.complete`,
             * aynı WebSocket) bekler — sonuç geldiği anda deftere yazılır.
             * Bu döngü yalnız yerel kuyruğa bakar ve defter değiştiyse
             * arayüze TEK olay yayınlar. Boşta maliyeti: bir SQLite okuması.
             */
            {
                let handle = app.handle().clone();
                std::thread::spawn(move || loop {
                    std::thread::sleep(std::time::Duration::from_secs(2));
                    let Some(state) = handle.try_state::<AppState>() else {
                        return;
                    };

                    while let Some(job_id) = state.index.claim_job() {
                        let worker = handle.clone();
                        std::thread::spawn(move || match worker.try_state::<AppState>() {
                            Some(state) => state.index.run_job(&job_id),
                            None => hermes::jobs::release(&job_id),
                        });
                    }

                    if hermes::jobs::take_dirty() {
                        use tauri::Emitter;
                        let _ = handle.emit(JOBS_CHANGED_EVENT, ());
                    }
                });
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
            // Mekanik mutasyonlar — madde 8.1. Semantik mutasyon BURAYA EKLENEMEZ.
            ipc::mutation_cmds::set_task_status,
            ipc::mutation_cmds::set_frontmatter_field,
            ipc::mutation_cmds::toggle_tag,
            ipc::mutation_cmds::move_task,
            ipc::mutation_cmds::quick_capture,
            ipc::mutation_cmds::inbox_status,
            // İş modülü — Sprint 3. Kuruma özel komut YOK.
            ipc::work_cmds::list_workspaces,
            ipc::work_cmds::workspace_overview,
            ipc::work_cmds::workspace_tasks,
            ipc::work_cmds::workspace_notes,
            ipc::work_cmds::workspace_meetings,
            ipc::work_cmds::workspace_documents,
            ipc::work_cmds::note_detail,
            ipc::work_cmds::open_document,
            // Hermes — Sprint 4. Aksiyon adları allowlist'ten doğrulanır.
            ipc::hermes_cmds::hermes_summary,
            ipc::hermes_cmds::hermes_actions,
            ipc::hermes_cmds::submit_job,
            ipc::hermes_cmds::active_jobs,
            ipc::hermes_cmds::job_history,
            ipc::hermes_cmds::retry_job,
            ipc::hermes_cmds::cancel_job,
        ])
        .run(tauri::generate_context!())
        .expect("ARKELÉS başlatılamadı");
}
