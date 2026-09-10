/*!
Index IPC komutları — Anayasa madde 9, 15.

Arayüzün index'i görmesi gerekir çünkü madde 9.3 uyarınca index her an
silinebilir ve yeniden kurulabilir; kullanıcı bu durumu (sakin biçimde)
görmelidir.
*/

use tauri::State;

use crate::error::{CoreError, CoreResult};
use crate::index::ScanReport;
use crate::types::IndexStatus;
use crate::AppState;

#[tauri::command]
pub fn index_status(state: State<'_, AppState>) -> IndexStatus {
    state.index.status(state.is_rebuilding())
}

/// Elle tam tarama — Sistem katmanından tetiklenir.
///
/// Madde 9.3'ün kullanıcıya açılan yüzü: index her an yeniden kurulabilir
/// ve bu kayıpsızdır.
#[tauri::command]
pub async fn rebuild_index(state: State<'_, AppState>) -> CoreResult<ScanReportDto> {
    let root = state.config.vault_path().ok_or(CoreError::VaultNotConfigured)?;

    state.set_rebuilding(true);
    let result = state.index.full_scan(&root);
    state.set_rebuilding(false);

    let report = result?;
    state.index.record_scan_ms(report.duration_ms);
    Ok(ScanReportDto::from(report))
}

/// Tarama raporu — madde 34.2: bütçe ölçülür ve RAPORLANIR.
#[derive(Debug, serde::Serialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct ScanReportDto {
    pub scanned: u32,
    pub reindexed: u32,
    /// Madde 15.6 kazancı: hash aynı olduğu için atlanan not sayısı.
    pub unchanged: u32,
    pub removed: u32,
    pub duration_ms: u32,
}

impl From<ScanReport> for ScanReportDto {
    fn from(r: ScanReport) -> Self {
        Self {
            scanned: r.scanned as u32,
            reindexed: r.reindexed as u32,
            unchanged: r.unchanged as u32,
            removed: r.removed as u32,
            duration_ms: r.duration_ms as u32,
        }
    }
}
