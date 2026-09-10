/*!
Vault IPC komutları — Anayasa madde 12, 15.1, 17.3.

Madde 15.1: arayüz markdown taramaz. Burada "dosya oku" komutu YOKTUR —
yalnız index'ten türetilmiş görünümler döner.
*/

use std::path::PathBuf;

use tauri::State;

use crate::error::CoreResult;
use crate::types::{DashboardView, SearchHit, TodayView, VaultStatus};
use crate::AppState;

/// Vault durumu. Yapılandırılmamış vault HATA DEĞİLDİR (madde 18.3 mantığı).
#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    let path = state
        .config
        .vault_path()
        .filter(|p| crate::vault::reader::is_readable(p))
        .map(|p| p.to_string_lossy().into_owned());

    state.index.vault_status(path)
}

/// Vault'u seçer: yapılandırmaya yazar, index'i sıfırlar, tam tarama yapar,
/// izleyiciyi yeniden kurar. Anayasa madde 17.3.
///
/// Yol geçersizse hata döner — bu, kullanıcının BİLİNÇLİ bir eyleminin
/// başarısız olmasıdır, arka plan arızası değil; sessiz kalmak yanlış olur.
#[tauri::command]
pub async fn select_vault(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    path: String,
) -> CoreResult<VaultStatus> {
    let root = PathBuf::from(&path);

    state.config.set_vault_path(&root)?;

    // Madde 9.3: index türetilmiş veridir, sıfırlamak kayıpsızdır.
    state.index.clear()?;

    let report = state.index.full_scan(&root)?;
    state.index.record_scan_ms(report.duration_ms);

    // Eski izleyiciyi düşür, yenisini kur (madde 20.4).
    state.restart_watcher(&app, &root);

    Ok(state.index.vault_status(Some(path)))
}

/// Bugün görünümü — Anayasa madde 36.2. Bütçe: < 10 ms (madde 34.1).
#[tauri::command]
pub fn list_today(state: State<'_, AppState>) -> CoreResult<TodayView> {
    state.index.today()
}

/// Panel görünümü — Anayasa madde 24.2.
#[tauri::command]
pub fn dashboard_view(state: State<'_, AppState>) -> CoreResult<DashboardView> {
    state.index.dashboard()
}

/// Not arama — Anayasa madde 27.4 (Cmd+K içinden).
#[tauri::command]
pub fn search_notes(state: State<'_, AppState>, query: String) -> CoreResult<Vec<SearchHit>> {
    state.index.search(&query, 20)
}
