/*!
Mekanik mutasyon komutları — Anayasa madde 8.1, 20.

YETKİ SINIRI: Bu dosyadaki her komut MEKANİK bir mutasyondur. Semantik
mutasyon (yeni not, belge, PDF, analiz, AI) buraya EKLENEMEZ — madde 8.2.

AKIŞ (madde 20 + madde 9 senkronizasyonu):
  1. Hedefi INDEX'ten oku (arayüzün gönderdiği bilgiye güvenilmez)
  2. Yönetilen mi? Değilse reddet (madde 16.3)
  3. WriteGuard ile atomic write (madde 20.1, 20.2)
  4. Index'i HEMEN senkronize et — yeni hash index'e yazılır
  5. Böylece dosya izleyici olayı geldiğinde hash EŞLEŞİR ve gereksiz
     yeniden indeksleme + gereksiz render OLUŞMAZ (madde 9, 35.1)
*/

use std::path::PathBuf;

use serde::Serialize;
use tauri::State;

use crate::error::{CoreError, CoreResult};
use crate::vault::note::TaskStatus;
use crate::vault::writer::{self, FieldValue, WriteGuard};
use crate::AppState;

/// Mutasyon sonucu — arayüz yeni sürümü bilmeli ki bir sonraki
/// mutasyon doğru guard'ı taşısın (madde 20.2).
#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct MutationResult {
    pub new_hash: String,
    /// Değişikliğin uygulandığı notun kimliği.
    pub note_id: String,
}

fn vault_root(state: &AppState) -> CoreResult<PathBuf> {
    state.config.vault_path().ok_or(CoreError::VaultNotConfigured)
}

/*
 * Yazma sonrası index senkronizasyonu.
 *
 * NEDEN HEMEN: dosya izleyici 200 ms sonra aynı dosyayı görecek. Index'i
 * şimdi güncellemezsek izleyici hash farkı görüp yeniden indeksler ve
 * `vault:changed` yayınlar → arayüz ikinci kez render olur. Optimistic UI
 * zaten doğru durumu gösterdiği için bu render GEREKSİZDİR (madde 35.1).
 *
 * Şimdi güncellersek izleyici hash'i EŞİT bulur, hiçbir şey yapmaz,
 * hiçbir olay yayınlanmaz. Döngü de oluşmaz.
 */
fn resync(state: &AppState, root: &std::path::Path, source_path: &str) {
    let full = root.join(source_path);
    let _ = state.index.sync_one(root, &full);
}

/// Bir notun yazılabilir olduğunu doğrular — Anayasa madde 16.3.
fn require_managed(target: &crate::index::WriteTarget) -> CoreResult<()> {
    if !target.managed {
        return Err(CoreError::NoteUnmanaged);
    }
    Ok(())
}

// ---------------------------------------------------------------------------

/// Görev durumunu değiştirir — Anayasa madde 8.1, 8.3.
#[tauri::command]
pub async fn set_task_status(
    state: State<'_, AppState>,
    task_id: String,
    done: bool,
) -> CoreResult<MutationResult> {
    let root = vault_root(&state)?;
    let target = state.index.task_write_target(&task_id)?;
    require_managed(&target.note)?;

    let guard = WriteGuard {
        expected_hash: target.note.content_hash.clone(),
        expected_mtime_ms: None,
    };

    let status = if done { TaskStatus::Done } else { TaskStatus::Open };

    let result = writer::set_task_status(
        &root,
        &target.note.source_path,
        target.line_number,
        &target.title,
        status,
        &guard,
    )?;

    resync(&state, &root, &target.note.source_path);

    Ok(MutationResult {
        new_hash: result.new_hash,
        note_id: note_id_of(&state, &target.note.source_path),
    })
}

/*
 * Görev sırası — Anayasa madde 8.1 (satır sırası mekanik mutasyondur).
 * Sprint 2'de yazılan `move_line` altyapısının UI bağlantısı (Sprint 5).
 *
 * Arayüz yalnız İKİ GÖREV KİMLİĞİ gönderir; dosya yolu, satır numaraları ve
 * hash index'ten okunur. Görev, hedef görevin yerine taşınır (dizi taşıma
 * anlamı): aşağı taşırken hedefin ALTINA, yukarı taşırken ÜSTÜNE düşer.
 */
#[tauri::command]
pub async fn move_task(
    state: State<'_, AppState>,
    task_id: String,
    target_task_id: String,
) -> CoreResult<MutationResult> {
    let root = vault_root(&state)?;
    let moving = state.index.task_write_target(&task_id)?;
    let target = state.index.task_write_target(&target_task_id)?;
    let (from, to) = plan_task_move(&moving, &target)?;
    require_managed(&moving.note)?;

    let guard = WriteGuard {
        expected_hash: moving.note.content_hash.clone(),
        expected_mtime_ms: None,
    };
    let result = writer::move_line(&root, &moving.note.source_path, from, to, &guard)?;

    resync(&state, &root, &moving.note.source_path);

    Ok(MutationResult {
        new_hash: result.new_hash,
        note_id: note_id_of(&state, &moving.note.source_path),
    })
}

/// İki görev AYNI notta olmalı; notlar arası taşıma yapı kararıdır (madde 8.2).
fn plan_task_move(
    moving: &crate::index::TaskTarget,
    target: &crate::index::TaskTarget,
) -> CoreResult<(u32, u32)> {
    if moving.note.source_path != target.note.source_path {
        return Err(CoreError::TargetMismatch);
    }
    Ok((moving.line_number, target.line_number))
}

/// Frontmatter alanı yazar — Anayasa madde 8.1.
///
/// `value` JSON olarak gelir; desteklenen tipler madde 7 kapsamı:
/// string, boolean, number, string list. `null` → anahtarı kaldır.
#[tauri::command]
pub async fn set_frontmatter_field(
    state: State<'_, AppState>,
    note_id: String,
    key: String,
    value: Option<serde_json::Value>,
) -> CoreResult<MutationResult> {
    let root = vault_root(&state)?;
    let target = state.index.note_write_target(&note_id)?;
    require_managed(&target)?;

    let field = match value {
        None | Some(serde_json::Value::Null) => None,
        Some(v) => Some(to_field_value(v)?),
    };

    let guard = WriteGuard {
        expected_hash: target.content_hash.clone(),
        expected_mtime_ms: None,
    };

    let result =
        writer::set_frontmatter_field(&root, &target.source_path, &key, field, &guard)?;

    resync(&state, &root, &target.source_path);

    Ok(MutationResult {
        new_hash: result.new_hash,
        note_id,
    })
}

/// Etiket ekler/çıkarır — Anayasa madde 8.1.
#[tauri::command]
pub async fn toggle_tag(
    state: State<'_, AppState>,
    note_id: String,
    tag: String,
    add: bool,
) -> CoreResult<MutationResult> {
    let root = vault_root(&state)?;
    let target = state.index.note_write_target(&note_id)?;
    require_managed(&target)?;

    let guard = WriteGuard {
        expected_hash: target.content_hash.clone(),
        expected_mtime_ms: None,
    };

    let result = writer::toggle_tag(&root, &target.source_path, &tag, add, &guard)?;

    resync(&state, &root, &target.source_path);

    Ok(MutationResult {
        new_hash: result.new_hash,
        note_id,
    })
}

/// Hızlı Yakalama — Anayasa madde 10.
///
/// ARKELÉS ham satırı ekler, ANLAM VERMEZ (madde 10.3). Gelen kutusu
/// dosyası yoksa OLUŞTURULMAZ (madde 10.2) — `inbox_missing` döner.
#[tauri::command]
pub async fn quick_capture(state: State<'_, AppState>, text: String) -> CoreResult<()> {
    let root = vault_root(&state)?;
    let inbox = state.config.inbox_path();

    writer::append_to_inbox(&root, &inbox, &text)?;
    resync(&state, &root, &inbox);

    Ok(())
}

/// Gelen kutusunun durumu — madde 18.2: olmayan yetenek gösterilmez.
#[derive(Debug, Serialize, ts_rs::TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct InboxStatus {
    /// Vault köküne göre yol.
    pub path: String,
    /// Dosya var mı? Yoksa Hızlı Yakalama arayüzde GÖRÜNMEZ.
    pub exists: bool,
}

#[tauri::command]
pub fn inbox_status(state: State<'_, AppState>) -> InboxStatus {
    let inbox = state.config.inbox_path();
    let exists = state
        .config
        .vault_path()
        .map(|root| root.join(&inbox).is_file())
        .unwrap_or(false);

    InboxStatus { path: inbox, exists }
}

// ---------------------------------------------------------------------------

/// JSON değerini frontmatter alanına çevirir. Desteklenmeyen tip reddedilir.
fn to_field_value(value: serde_json::Value) -> CoreResult<FieldValue> {
    match value {
        serde_json::Value::String(s) => Ok(FieldValue::Text(s)),
        serde_json::Value::Bool(b) => Ok(FieldValue::Bool(b)),
        serde_json::Value::Number(n) => n
            .as_f64()
            .map(FieldValue::Number)
            .ok_or(CoreError::TargetMismatch),
        serde_json::Value::Array(items) => {
            // Yalnız dizge listesi (madde 7 ilk kapsam). Karışık liste
            // reddedilir — sessizce dönüştürmek veri anlamını değiştirirdi.
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    serde_json::Value::String(s) => out.push(s),
                    _ => return Err(CoreError::TargetMismatch),
                }
            }
            Ok(FieldValue::List(out))
        }
        _ => Err(CoreError::TargetMismatch),
    }
}

/// Yazma sonrası notun kimliğini index'ten tazeler.
///
/// Kimlik değişmez (madde 16.1: kalıcı ULID) ama yol üzerinden okumak,
/// arayüze doğru kimliği döndürmenin en güvenli yolu.
fn note_id_of(state: &AppState, source_path: &str) -> String {
    state
        .index
        .note_id_by_path(source_path)
        .unwrap_or_else(|| source_path.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::index::{TaskTarget, WriteTarget};

    fn target(path: &str, line: u32) -> TaskTarget {
        TaskTarget {
            note: WriteTarget {
                source_path: path.into(),
                managed: true,
                content_hash: "h".into(),
            },
            line_number: line,
            title: "görev".into(),
            status: "open".into(),
        }
    }

    #[test]
    fn ayni_nottaki_gorevler_satir_planina_donusur() {
        assert_eq!(plan_task_move(&target("a.md", 7), &target("a.md", 9)).unwrap(), (7, 9));
        assert_eq!(plan_task_move(&target("a.md", 9), &target("a.md", 7)).unwrap(), (9, 7));
    }

    #[test]
    fn notlar_arasi_tasima_reddedilir() {
        let err = plan_task_move(&target("a.md", 3), &target("b.md", 3)).unwrap_err();
        assert_eq!(err.code(), "target_mismatch");
    }
}
