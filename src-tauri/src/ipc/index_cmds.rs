/*!
Index IPC komutları — Anayasa madde 9, 15.

Arayüzün index'i görmesi gerekir çünkü madde 9.3 uyarınca index her an
silinebilir ve yeniden kurulabilir; kullanıcı bu durumu (sakin biçimde)
görmelidir.
*/

use serde::Serialize;
use tauri::State;

use crate::index::SCHEMA_VERSION;
use crate::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    pub ready: bool,
    /// Anayasa madde 15.5: sürüm değişince index otomatik yeniden kurulur.
    pub schema_version: i64,
    pub rebuilding: bool,
}

#[tauri::command]
pub fn index_status(state: State<'_, AppState>) -> IndexStatus {
    let version = state.index.schema_version().unwrap_or(0);

    IndexStatus {
        ready: version == SCHEMA_VERSION,
        schema_version: version,
        // Sprint 1: oluşturucu çalışırken true olacak.
        rebuilding: false,
    }
}
