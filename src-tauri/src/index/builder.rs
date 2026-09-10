/*!
Index oluşturucu — Anayasa madde 15.

15.4  Vault → index akışı TEK YÖNLÜDÜR. Index'ten vault'a asla veri akmaz.
15.6  Dosya izleyici değişikliği yakalar, index'i artımlı günceller.
      İçerik hash'i aynıysa yeniden indeksleme YAPILMAZ.
34.1  Tam index kurulumu (2.000 not) < 1 sn.

Bu modül `&Connection` alır; kilit yönetimi çağırana (db.rs) aittir.
*/

use std::collections::HashMap;
use std::path::Path;

use rusqlite::{params, Connection, Transaction};

use crate::error::{CoreError, CoreResult};
use crate::vault::note::ParsedNote;
use crate::vault::{reader, time};

/// Bir tarama turunun sonucu — arayüze rapor edilir, karar için değil bilgi için.
#[derive(Debug, Default, Clone, Copy)]
pub struct ScanReport {
    pub scanned: usize,
    /// Hash değiştiği için yeniden yazılan not sayısı.
    pub reindexed: usize,
    /// Hash aynı olduğu için ATLANAN not sayısı (madde 15.6 kazancı).
    pub unchanged: usize,
    /// Vault'tan silindiği için index'ten düşen not sayısı.
    pub removed: usize,
    /// Okunamayan dosya sayısı. Sprint 1 borcu #4: artık SAYILIYOR, yutulmuyor.
    pub unreadable: usize,
    /// Index satır hataları. Sprint 1 borcu #4.
    pub row_errors: usize,
    pub duration_ms: u64,
}

/// Tam tarama: vault'u baştan sona gezip index'i güncel hale getirir.
///
/// "Tam" tarama artımlıdır: her dosyanın hash'i index'teki ile karşılaştırılır,
/// değişmeyen dosya yeniden ayrıştırılmaz. İlk çalıştırmada her şey yeni
/// olduğu için tam maliyet ödenir; sonraki turlar ucuzdur.
pub fn full_scan(conn: &mut Connection, vault_root: &Path) -> CoreResult<ScanReport> {
    let started = std::time::Instant::now();
    let mut report = ScanReport::default();

    let existing = load_hashes(conn)?;
    let files = reader::collect_markdown_files(vault_root);

    let tx = conn.transaction().map_err(CoreError::IndexQuery)?;
    let mut seen_paths: Vec<String> = Vec::with_capacity(files.len());

    for path in &files {
        report.scanned += 1;

        let Some(note) = reader::read_note(vault_root, path) else {
            // Sprint 1 borcu #4: sessizce atlamıyoruz — sayıyoruz ve logluyoruz.
            // Madde 19.5: yalnız YOL loglanır, içerik asla.
            report.unreadable += 1;
            eprintln!("[index] okunamadı: {}", path.display());
            continue;
        };
        seen_paths.push(note.source_path.clone());

        // Madde 15.6: hash aynıysa hiç dokunmayız.
        if existing.get(&note.source_path).is_some_and(|h| h == &note.content_hash) {
            report.unchanged += 1;
            continue;
        }

        upsert_note(&tx, &note)?;
        report.reindexed += 1;
    }

    report.removed = remove_missing(&tx, &seen_paths)?;

    set_meta(&tx, "indexed_at", &time::now_iso())?;
    set_meta(&tx, "note_count", &seen_paths.len().to_string())?;

    tx.commit().map_err(CoreError::IndexQuery)?;

    report.duration_ms = started.elapsed().as_millis() as u64;
    Ok(report)
}

/// Tek bir dosyayı günceller — dosya izleyicinin kullandığı yol (madde 15.6).
///
/// Dosya silinmişse index'ten düşürülür. Hash aynıysa hiçbir şey yapılmaz.
pub fn sync_one(conn: &mut Connection, vault_root: &Path, path: &Path) -> CoreResult<bool> {
    let source_path = path
        .strip_prefix(vault_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");

    let tx = conn.transaction().map_err(CoreError::IndexQuery)?;

    let changed = if path.is_file() {
        match reader::read_note(vault_root, path) {
            Some(note) => {
                let current: Option<String> = tx
                    .query_row(
                        "SELECT content_hash FROM notes WHERE source_path = ?1",
                        params![note.source_path],
                        |row| row.get(0),
                    )
                    .ok();

                if current.as_deref() == Some(note.content_hash.as_str()) {
                    false // madde 15.6: değişmemiş
                } else {
                    upsert_note(&tx, &note)?;
                    true
                }
            }
            None => false,
        }
    } else {
        delete_by_path(&tx, &source_path)? > 0
    };

    if changed {
        set_meta(&tx, "indexed_at", &time::now_iso())?;
        let count: i64 = tx
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .map_err(CoreError::IndexQuery)?;
        set_meta(&tx, "note_count", &count.to_string())?;
    }

    tx.commit().map_err(CoreError::IndexQuery)?;
    Ok(changed)
}

fn load_hashes(conn: &Connection) -> CoreResult<HashMap<String, String>> {
    let mut stmt = conn
        .prepare("SELECT source_path, content_hash FROM notes")
        .map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map([], |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)))
        .map_err(CoreError::IndexQuery)?;

    let mut out = HashMap::new();
    for row in rows {
        let (path, hash) = row.map_err(CoreError::IndexQuery)?;
        out.insert(path, hash);
    }
    Ok(out)
}

fn upsert_note(tx: &Transaction<'_>, note: &ParsedNote) -> CoreResult<()> {
    // Aynı yolda eski kayıt varsa (kimliği değişmiş olabilir) önce temizle.
    delete_by_path(tx, &note.source_path)?;

    let frontmatter = serde_json::to_string(&note.frontmatter).unwrap_or_else(|_| "{}".into());
    let created_at = note
        .frontmatter
        .get("created")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    tx.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at, created_at, frontmatter)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
         ON CONFLICT(id) DO UPDATE SET
            source_path = excluded.source_path,
            title = excluded.title,
            managed = excluded.managed,
            content_hash = excluded.content_hash,
            modified_at = excluded.modified_at,
            created_at = excluded.created_at,
            frontmatter = excluded.frontmatter",
        params![
            note.id,
            note.source_path,
            note.title,
            note.managed as i64,
            note.content_hash,
            note.modified_at,
            created_at,
            frontmatter,
        ],
    )
    .map_err(CoreError::IndexQuery)?;

    // Görevler ve bağlantılar her seferinde yeniden türetilir — türetilmiş
    // veri (madde 9.2), birikmemesi için önce silinir.
    tx.execute("DELETE FROM tasks WHERE note_id = ?1", params![note.id])
        .map_err(CoreError::IndexQuery)?;
    tx.execute("DELETE FROM links WHERE source_id = ?1", params![note.id])
        .map_err(CoreError::IndexQuery)?;

    for task in &note.tasks {
        // Görev kimliği: not + satır. Satır kayarsa kimlik değişir — kabul
        // edilebilir, çünkü görevler her taramada yeniden türetilir.
        let task_id = format!("{}#{}", note.id, task.line_number);
        let due = task.due.clone().or_else(|| {
            note.frontmatter
                .get("due")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        });

        tx.execute(
            "INSERT INTO tasks (id, note_id, line_number, title, status, workspace, due)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                task_id,
                note.id,
                task.line_number as i64,
                task.title,
                task.status.as_str(),
                note.workspace,
                due,
            ],
        )
        .map_err(CoreError::IndexQuery)?;
    }

    for target in &note.links {
        // Aynı hedefe iki bağlantı bir ilişkidir — PRIMARY KEY çakışması
        // hata değil, beklenen durum.
        tx.execute(
            "INSERT OR IGNORE INTO links (source_id, target_ref) VALUES (?1, ?2)",
            params![note.id, target],
        )
        .map_err(CoreError::IndexQuery)?;
    }

    tx.execute("DELETE FROM notes_fts WHERE note_id = ?1", params![note.id])
        .map_err(CoreError::IndexQuery)?;
    tx.execute(
        "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, ?3)",
        params![note.id, note.title, note.body],
    )
    .map_err(CoreError::IndexQuery)?;

    Ok(())
}

fn delete_by_path(tx: &Transaction<'_>, source_path: &str) -> CoreResult<usize> {
    // Satır hatası burada VERİ KAYBI riski taşır: silinmesi gereken bir
    // FTS kaydını atlarsak index tutarsız kalır. Bu yüzden hata YUTULMAZ,
    // yukarı fırlatılır (Sprint 1 borcu #4).
    let ids: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT id FROM notes WHERE source_path = ?1")
            .map_err(CoreError::IndexQuery)?;
        let rows = stmt
            .query_map(params![source_path], |row| row.get::<_, String>(0))
            .map_err(CoreError::IndexQuery)?;
        rows.collect::<rusqlite::Result<Vec<String>>>()
            .map_err(CoreError::IndexQuery)?
    };

    for id in &ids {
        tx.execute("DELETE FROM notes_fts WHERE note_id = ?1", params![id])
            .map_err(CoreError::IndexQuery)?;
    }
    // tasks ve links, notes'a ON DELETE CASCADE ile bağlı.
    let removed = tx
        .execute("DELETE FROM notes WHERE source_path = ?1", params![source_path])
        .map_err(CoreError::IndexQuery)?;

    Ok(removed)
}

/// Vault'ta artık bulunmayan notları index'ten düşürür.
fn remove_missing(tx: &Transaction<'_>, seen_paths: &[String]) -> CoreResult<usize> {
    // Aynı gerekçe: eksik yolu atlamak, silinmiş notu index'te bırakır.
    let indexed: Vec<String> = {
        let mut stmt = tx
            .prepare("SELECT source_path FROM notes")
            .map_err(CoreError::IndexQuery)?;
        let rows = stmt
            .query_map([], |row| row.get::<_, String>(0))
            .map_err(CoreError::IndexQuery)?;
        rows.collect::<rusqlite::Result<Vec<String>>>()
            .map_err(CoreError::IndexQuery)?
    };

    let seen: std::collections::HashSet<&str> = seen_paths.iter().map(String::as_str).collect();
    let mut removed = 0;
    for path in indexed {
        if !seen.contains(path.as_str()) {
            removed += delete_by_path(tx, &path)?;
        }
    }
    Ok(removed)
}

fn set_meta(tx: &Transaction<'_>, key: &str, value: &str) -> CoreResult<()> {
    tx.execute(
        "INSERT INTO index_meta (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        params![key, value],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
}
