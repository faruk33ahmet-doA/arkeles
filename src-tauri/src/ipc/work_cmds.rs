/*!
İş modülü IPC komutları — Sprint 3.

KURUMA ÖZEL KOMUT YOKTUR. Her komut `workspace_id` alır; WIF ve GEN aynı
yoldan geçer. Yeni kurum aktifleştirmek `config::workspaces` içinde bir
bayrak değiştirmektir.

Bu komutların hiçbiri YAZMAZ — mekanik mutasyonlar `mutation_cmds`'te ve
oradaki yetki kontrollerinden geçer (madde 8.1, 16.3).
*/

use tauri::State;

use crate::error::CoreResult;
use crate::types::{
    DocumentRef, MeetingSummary, NoteDetail, NoteSummary, Task, Workspace, WorkspaceOverview,
};
use crate::AppState;

/// 7 kurumun tamamı — Anayasa madde 36.1.
#[tauri::command]
pub fn list_workspaces(state: State<'_, AppState>) -> CoreResult<Vec<Workspace>> {
    state.index.workspaces()
}

#[tauri::command]
pub fn workspace_overview(
    state: State<'_, AppState>,
    workspace_id: String,
) -> CoreResult<WorkspaceOverview> {
    state.index.workspace_overview(&workspace_id)
}

#[tauri::command]
pub fn workspace_tasks(
    state: State<'_, AppState>,
    workspace_id: String,
) -> CoreResult<Vec<Task>> {
    state.index.workspace_tasks(&workspace_id)
}

/// `kind`: "note" | "project" — Sprint 3 madde 7, 10.
#[tauri::command]
pub fn workspace_notes(
    state: State<'_, AppState>,
    workspace_id: String,
    kind: String,
) -> CoreResult<Vec<NoteSummary>> {
    state.index.workspace_notes(&workspace_id, &kind)
}

#[tauri::command]
pub fn workspace_meetings(
    state: State<'_, AppState>,
    workspace_id: String,
) -> CoreResult<Vec<MeetingSummary>> {
    state.index.workspace_meetings(&workspace_id)
}

#[tauri::command]
pub fn workspace_documents(
    state: State<'_, AppState>,
    workspace_id: String,
) -> CoreResult<Vec<DocumentRef>> {
    state.index.workspace_documents(&workspace_id)
}

#[tauri::command]
pub fn note_detail(state: State<'_, AppState>, note_id: String) -> CoreResult<NoteDetail> {
    state.index.note_detail(&note_id)
}

/*
 * Belgeyi işletim sistemine açtırır — Sprint 3 madde 8.
 *
 * ARKELÉS belgeyi AÇMAZ, RENDER ETMEZ, KOPYALAMAZ (madde 7.3). Dosyayı
 * sahibi olan uygulamaya devreder.
 *
 * GÜVENLİK: yol çekirdekte INDEX'ten doğrulanır — arayüzden gelen ham yol
 * kullanılmaz. Index'te olmayan bir dosya açılmaz (madde 19).
 */
#[tauri::command]
pub async fn open_document(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    document_id: String,
) -> CoreResult<()> {
    use crate::error::CoreError;

    let root = state
        .config
        .vault_path()
        .ok_or(CoreError::VaultNotConfigured)?;

    let relative = state
        .index
        .document_path(&document_id)
        .ok_or(CoreError::TargetMismatch)?;

    let full = root.join(&relative);
    if !full.is_file() {
        return Err(CoreError::TargetMismatch);
    }

    tauri_plugin_opener::OpenerExt::opener(&app)
        .open_path(full.to_string_lossy(), None::<&str>)
        .map_err(|_| CoreError::TargetMismatch)?;

    Ok(())
}
