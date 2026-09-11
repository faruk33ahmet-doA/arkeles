/*!
İş modülü sorguları — Sprint 3, Anayasa madde 36.3.

BU DOSYADA KURUMA ÖZEL KOD YOKTUR. Her sorgu `workspace_id` parametresi
alır; WIF ve GEN aynı kodu kullanır. Yeni bir kurumu aktifleştirmek
`config::workspaces` içinde `active: true` yapmaktır.

Madde 8.2: burada YARGI yok. Sayımlar sayımdır, gruplamalar görev
işaretinin kendisinden gelir. "Öncelik" yalnız Hermes vault'a yazdıysa
okunur (madde 11.2).
*/

use rusqlite::{params, Connection};

use crate::config::workspaces;
use crate::error::{CoreError, CoreResult};
use crate::index::query::collect_rows;
use crate::types::{
    DocumentRef, MeetingSummary, NoteDetail, NoteLink, NoteSummary, Task, Workspace,
    WorkspaceOverview,
};

use crate::vault::time;

/// Genel bakışta gösterilecek son not sayısı — madde 24.3 (tek bakış).
const RECENT_LIMIT: usize = 5;

/// Çalışma alanı listesi + sayımlar — Sprint 3 madde 2.
///
/// 7 kurumun TAMAMI döner (madde 36.1: zihinsel model baştan bütün).
/// Pasif olanlar da sayımlarını taşır; veri varsa görünür.
pub fn list_workspaces(conn: &Connection) -> CoreResult<Vec<Workspace>> {
    let mut out = Vec::with_capacity(workspaces::WORKSPACES.len());

    for def in workspaces::WORKSPACES {
        let outstanding: i64 = conn
            .query_row(
                &format!(
                    "SELECT COUNT(*) FROM tasks WHERE workspace = ?1 AND status IN {}",
                    crate::vault::note::TaskStatus::OUTSTANDING_SQL
                ),
                params![def.id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        let notes: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM notes WHERE workspace = ?1",
                params![def.id],
                |row| row.get(0),
            )
            .unwrap_or(0);

        out.push(Workspace {
            id: def.id.to_string(),
            label: def.label.to_string(),
            active: def.active,
            outstanding_tasks: outstanding as u32,
            note_count: notes as u32,
        });
    }

    Ok(out)
}

/// Genel Bakış — Sprint 3 madde 3.
pub fn overview(conn: &Connection, workspace_id: &str) -> CoreResult<(WorkspaceOverview, u64)> {
    let mut row_errors = 0u64;

    let count = |predicate: &str| -> i64 {
        conn.query_row(
            &format!("SELECT COUNT(*) FROM tasks WHERE workspace = ?1 AND {predicate}"),
            params![workspace_id],
            |row| row.get(0),
        )
        .unwrap_or(0)
    };

    let active_tasks = count("status IN ('open','in_progress')") as u32;
    let waiting_tasks = count("status = 'deferred'") as u32;
    let done_tasks = count("status = 'done'") as u32;

    let last_activity: Option<String> = conn
        .query_row(
            "SELECT MAX(modified_at) FROM notes WHERE workspace = ?1",
            params![workspace_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    /*
     * Öncelik — Anayasa madde 11.2.
     *
     * ARKELÉS ÖNCELİK ANALİZİ YAPMAZ. Hermes bir nota
     * `arkeles_type: priority` + `workspace` + `value` yazdıysa okunur.
     * Yoksa None döner ve arayüz alanı GİZLER (Sprint 3 madde 3).
     */
    let priority: Option<String> = conn
        .query_row(
            "SELECT json_extract(frontmatter, '$.value') FROM notes
             WHERE workspace = ?1
               AND json_extract(frontmatter, '$.arkeles_type') = 'priority'
             ORDER BY modified_at DESC LIMIT 1",
            params![workspace_id],
            |row| row.get(0),
        )
        .ok()
        .flatten();

    let mut stmt = conn
        .prepare(
            "SELECT id, title, modified_at, managed FROM notes
             WHERE workspace = ?1 AND kind IN ('note','project')
             ORDER BY modified_at DESC LIMIT ?2",
        )
        .map_err(CoreError::IndexQuery)?;

    let rows = stmt
        .query_map(params![workspace_id, RECENT_LIMIT as i64], note_summary)
        .map_err(CoreError::IndexQuery)?;
    let (recent_notes, e1) = collect_rows(rows, "notes/recent");
    row_errors += e1;

    // Bugün ve sonrası — geçmiş toplantı "yaklaşan" değildir.
    let today = time::today_iso();
    let next_meeting = conn
        .query_row(
            "SELECT id, title, event_date, workspace FROM notes
             WHERE workspace = ?1 AND kind = 'meeting'
               AND event_date IS NOT NULL AND event_date >= ?2
             ORDER BY event_date ASC LIMIT 1",
            params![workspace_id, today],
            meeting_summary,
        )
        .ok();

    Ok((
        WorkspaceOverview {
            workspace_id: workspace_id.to_string(),
            active_tasks,
            waiting_tasks,
            done_tasks,
            last_activity,
            priority,
            recent_notes,
            next_meeting,
        },
        row_errors,
    ))
}

/// Çalışma alanının görevleri — Sprint 3 madde 6.
///
/// SIRALAMA yargı değil, düzendir: önce bitmemişler, sonra vade, sonra başlık.
pub fn tasks(conn: &Connection, workspace_id: &str) -> CoreResult<(Vec<Task>, u64)> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.title, t.status, t.note_id, t.workspace, t.due, n.managed
             FROM tasks t JOIN notes n ON n.id = t.note_id
             WHERE t.workspace = ?1
             ORDER BY
               CASE t.status
                 WHEN 'in_progress' THEN 0
                 WHEN 'open'        THEN 1
                 WHEN 'deferred'    THEN 2
                 WHEN 'done'        THEN 3
                 ELSE 4
               END,
               t.due IS NULL, t.due ASC, t.title ASC",
        )
        .map_err(CoreError::IndexQuery)?;

    let rows = stmt
        .query_map(params![workspace_id], task_row)
        .map_err(CoreError::IndexQuery)?;

    Ok(collect_rows(rows, "tasks/workspace"))
}

/// Notlar — Sprint 3 madde 10. `kind` ile süzülebilir (projeler, toplantılar).
pub fn notes(
    conn: &Connection,
    workspace_id: &str,
    kind: &str,
) -> CoreResult<(Vec<NoteSummary>, u64)> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, modified_at, managed FROM notes
             WHERE workspace = ?1 AND kind = ?2
             ORDER BY modified_at DESC",
        )
        .map_err(CoreError::IndexQuery)?;

    let rows = stmt
        .query_map(params![workspace_id, kind], note_summary)
        .map_err(CoreError::IndexQuery)?;

    Ok(collect_rows(rows, "notes/kind"))
}

/// Toplantılar — Sprint 3 madde 9. Yalnız mevcut Obsidian verisi.
pub fn meetings(
    conn: &Connection,
    workspace_id: &str,
) -> CoreResult<(Vec<MeetingSummary>, u64)> {
    let mut stmt = conn
        .prepare(
            "SELECT id, title, event_date, workspace FROM notes
             WHERE workspace = ?1 AND kind = 'meeting' AND event_date IS NOT NULL
             ORDER BY event_date DESC",
        )
        .map_err(CoreError::IndexQuery)?;

    let rows = stmt
        .query_map(params![workspace_id], meeting_summary)
        .map_err(CoreError::IndexQuery)?;

    Ok(collect_rows(rows, "notes/meetings"))
}

/*
 * Belgeler — Sprint 3 madde 8.
 *
 * Belgelerin frontmatter'ı yoktur; çalışma alanı ilişkisi KLASÖRDEN
 * çıkarılmaz (yanlış eşleşme riski, madde 6.3). İlişki, belgeye BAĞLANAN
 * notun çalışma alanından gelir: `links.target_ref` belgenin dosya adına
 * veya yoluna eşleşiyorsa o belge o kuruma aittir.
 *
 * Hiçbir nottan bağlanmayan belge hiçbir kuruma DÜŞMEZ.
 */
pub fn documents(
    conn: &Connection,
    workspace_id: &str,
) -> CoreResult<(Vec<DocumentRef>, u64)> {
    let mut stmt = conn
        .prepare(
            "SELECT DISTINCT d.id, d.source_path, d.file_name, d.extension,
                    d.size_bytes, d.modified_at
             FROM documents d
             JOIN links l
               ON l.target_ref = d.file_name
               OR l.target_ref = d.source_path
               OR l.target_ref = replace(d.file_name, '.' || d.extension, '')
             JOIN notes n ON n.id = l.source_id
             WHERE n.workspace = ?1
             ORDER BY d.modified_at DESC",
        )
        .map_err(CoreError::IndexQuery)?;

    let rows = stmt
        .query_map(params![workspace_id], |row| {
            Ok(DocumentRef {
                id: row.get(0)?,
                source_path: row.get(1)?,
                file_name: row.get(2)?,
                extension: row.get(3)?,
                size_bytes: row.get::<_, i64>(4)? as u32,
                modified_at: row.get(5)?,
            })
        })
        .map_err(CoreError::IndexQuery)?;

    Ok(collect_rows(rows, "documents"))
}

/// Not detayı — Sprint 3 madde 10, 11.
pub fn note_detail(conn: &Connection, note_id: &str) -> CoreResult<NoteDetail> {
    let (id, title, source_path, managed, workspace, kind, modified_at, frontmatter) = conn
        .query_row(
            "SELECT id, title, source_path, managed, workspace, kind, modified_at,
                    COALESCE(frontmatter, '{}')
             FROM notes WHERE id = ?1",
            params![note_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, i64>(3)? != 0,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, String>(7)?,
                ))
            },
        )
        .map_err(|_| CoreError::TargetMismatch)?;

    // Gövde FTS kopyasından okunur (madde 9.4: bu bir KOPYADIR).
    let body: String = conn
        .query_row(
            "SELECT body FROM notes_fts WHERE note_id = ?1",
            params![note_id],
            |row| row.get(0),
        )
        .unwrap_or_default();

    let mut task_stmt = conn
        .prepare(
            "SELECT t.id, t.title, t.status, t.note_id, t.workspace, t.due, n.managed
             FROM tasks t JOIN notes n ON n.id = t.note_id
             WHERE t.note_id = ?1 ORDER BY t.line_number ASC",
        )
        .map_err(CoreError::IndexQuery)?;
    let (tasks, _) = collect_rows(
        task_stmt
            .query_map(params![note_id], task_row)
            .map_err(CoreError::IndexQuery)?,
        "tasks/note",
    );

    // --- Giden bağlantılar (madde 11) ---
    let mut out_stmt = conn
        .prepare(
            "SELECT l.target_ref, n.id
             FROM links l
             LEFT JOIN notes n
               ON n.title = l.target_ref
               OR n.source_path = l.target_ref || '.md'
             WHERE l.source_id = ?1
             ORDER BY l.target_ref ASC",
        )
        .map_err(CoreError::IndexQuery)?;
    let (outgoing, _) = collect_rows(
        out_stmt
            .query_map(params![note_id], link_row)
            .map_err(CoreError::IndexQuery)?,
        "links/outgoing",
    );

    // --- Gelen bağlantılar: başlığa VEYA dosya adına işaret edenler ---
    let stem = source_path
        .rsplit('/')
        .next()
        .unwrap_or(&source_path)
        .trim_end_matches(".md")
        .to_string();

    let mut in_stmt = conn
        .prepare(
            "SELECT n.title, n.id
             FROM links l JOIN notes n ON n.id = l.source_id
             WHERE (l.target_ref = ?1 OR l.target_ref = ?2) AND l.source_id <> ?3
             ORDER BY n.title ASC",
        )
        .map_err(CoreError::IndexQuery)?;
    let (incoming, _) = collect_rows(
        in_stmt
            .query_map(params![title, stem, note_id], link_row)
            .map_err(CoreError::IndexQuery)?,
        "links/incoming",
    );

    Ok(NoteDetail {
        id,
        title,
        source_path,
        managed,
        workspace,
        kind,
        modified_at,
        frontmatter,
        body,
        tasks,
        outgoing,
        incoming,
    })
}

// --- satır dönüştürücüleri ---------------------------------------------------

fn task_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Task> {
    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        status: row.get(2)?,
        note_id: row.get(3)?,
        workspace: row.get(4)?,
        due: row.get(5)?,
        managed: row.get::<_, i64>(6)? != 0,
    })
}

fn note_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteSummary> {
    Ok(NoteSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        modified_at: row.get(2)?,
        managed: row.get::<_, i64>(3)? != 0,
    })
}

fn meeting_summary(row: &rusqlite::Row<'_>) -> rusqlite::Result<MeetingSummary> {
    Ok(MeetingSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        date: row.get(2)?,
        workspace: row.get(3)?,
    })
}

fn link_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<NoteLink> {
    Ok(NoteLink {
        title: row.get(0)?,
        note_id: row.get(1)?,
    })
}
