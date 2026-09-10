/*!
Vault IPC komutları — Anayasa madde 12, 15.1, 17.3.

Madde 15.1: arayüz markdown taramaz. Bu yüzden burada "dosya oku" gibi
bir komut YOKTUR — yalnız index'ten türetilmiş özetler döner.
*/

use serde::Serialize;
use tauri::State;

use crate::vault::reader;
use crate::AppState;

/// Anayasa madde 17.3: vault yolu yapılandırılmamış olabilir → `path: None`.
///
/// `camelCase` çünkü tüketici TypeScript. Sprint 1'de bu tip `ts-rs` ile
/// üretilecek ve elle hizalama bitecek (madde 14.2).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub path: Option<String>,
    /// Anayasa madde 9.4 & 11.5: index'ten SAYIM. Analiz değil.
    pub note_count: u64,
    pub indexed_at: Option<String>,
}

/// Vault durumu.
///
/// Yapılandırılmamış veya erişilemez vault HATA DEĞİLDİR (madde 18.3
/// mantığı): arayüz bunu "Vault yapılandırılmadı" diye sakin gösterir.
#[tauri::command]
pub fn vault_status(state: State<'_, AppState>) -> VaultStatus {
    let path = state
        .config
        .vault_path
        .as_ref()
        .filter(|p| reader::is_readable(p))
        .map(|p| p.to_string_lossy().into_owned());

    // Sayım başarısız olursa 0 döner — index henüz kurulmamış olabilir.
    let note_count = state.index.note_count().unwrap_or(0);

    VaultStatus {
        path,
        note_count,
        // Sprint 1: index oluşturucu son tarama zamanını yazacak.
        indexed_at: None,
    }
}
