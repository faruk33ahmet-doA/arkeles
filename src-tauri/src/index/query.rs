/*!
Index sorguları — Anayasa madde 15.2, 34.1.

15.2  Bütün okuma index üzerinden yapılır.
34.1  Bugün görünümü sorgusu (2.000 not) < 10 ms.

BU DOSYADA YARGI YOKTUR (madde 8.2, 11.4). Sorgular yalnız SAYAR ve
FİLTRELER. "Kritik" tanımı bile bir kural değil, sıralamadır: vadesi
geçmiş olanlar önce. Öncelik SKORU üretmek Hermes'in işidir.

Tarih karşılaştırmaları DİZGE üzerinden yapılır; ISO 8601'in sözlüksel
sırası kronolojik sırayla aynıdır (bkz. vault::time testleri).
*/

use rusqlite::{params, Connection};

use crate::error::{CoreError, CoreResult};
use crate::types::{
    DashboardView, LifeScore, NoteSummary, SearchHit, Task, TodayView, WorkspaceSummary,
};
use crate::vault::time;

/// Panel için kaç kritik görev gösterilir. Madde 24.3: tek bakış kuralı —
/// kaydırma gerekiyorsa bilgi EKLENMEZ, çıkarılır.
const CRITICAL_LIMIT: usize = 6;

pub fn today(conn: &Connection) -> CoreResult<TodayView> {
    let today = time::today_iso();

    let overdue = tasks_where(
        conn,
        "t.status = 'open' AND t.due IS NOT NULL AND t.due < ?1",
        params![today],
        "t.due ASC",
        None,
    )?;

    let due = tasks_where(
        conn,
        "t.due = ?1",
        params![today],
        "t.status ASC, t.title ASC",
        None,
    )?;

    let mut stmt = conn
        .prepare(
            "SELECT id, title, modified_at, managed FROM notes
             WHERE substr(modified_at, 1, 10) = ?1
             ORDER BY modified_at DESC
             LIMIT 20",
        )
        .map_err(CoreError::IndexQuery)?;

    let touched_notes = stmt
        .query_map(params![today], |row| {
            Ok(NoteSummary {
                id: row.get(0)?,
                title: row.get(1)?,
                modified_at: row.get(2)?,
                managed: row.get::<_, i64>(3)? != 0,
            })
        })
        .map_err(CoreError::IndexQuery)?
        .filter_map(Result::ok)
        .collect();

    Ok(TodayView {
        overdue,
        due,
        touched_notes,
    })
}

pub fn dashboard(conn: &Connection) -> CoreResult<DashboardView> {
    let today = time::today_iso();

    // Kritik = vadesi geçmiş + bugün vadesi olan açık görevler, vade sırasıyla.
    let critical_tasks = tasks_where(
        conn,
        "t.status = 'open' AND t.due IS NOT NULL AND t.due <= ?1",
        params![today],
        "t.due ASC",
        Some(CRITICAL_LIMIT),
    )?;

    let mut stmt = conn
        .prepare(
            "SELECT workspace, COUNT(*) FROM tasks
             WHERE status = 'open' AND workspace IS NOT NULL
             GROUP BY workspace
             ORDER BY COUNT(*) DESC, workspace ASC",
        )
        .map_err(CoreError::IndexQuery)?;

    let workspaces = stmt
        .query_map([], |row| {
            Ok(WorkspaceSummary {
                name: row.get(0)?,
                open_tasks: row.get::<_, i64>(1)? as u32,
            })
        })
        .map_err(CoreError::IndexQuery)?
        .filter_map(Result::ok)
        .collect();

    Ok(DashboardView {
        life_score: life_score(conn)?,
        workspaces,
        critical_tasks,
    })
}

/*
 * Hayat skoru — Anayasa madde 11.
 *
 * 11.1  HERMES hesaplar ve vault'a yazar.
 * 11.2  ARKELÉS yalnız OKUR. Formülü bilmez, HESAPLAMAZ.
 *
 * Bu yüzden burada hiçbir aritmetik yoktur. Skor, vault'ta `arkeles_type:
 * life_score` frontmatter'ı taşıyan notun `value` ve `computed_at`
 * alanlarından OKUNUR. Hermes o notu yazmadıysa skor yoktur — hata değil,
 * henüz yok (madde 26.2).
 */
fn life_score(conn: &Connection) -> CoreResult<Option<LifeScore>> {
    let row: Option<String> = conn
        .query_row(
            "SELECT frontmatter FROM notes
             WHERE json_extract(frontmatter, '$.arkeles_type') = 'life_score'
             ORDER BY modified_at DESC LIMIT 1",
            [],
            |row| row.get(0),
        )
        .ok();

    let Some(raw) = row else { return Ok(None) };
    let Ok(fm) = serde_json::from_str::<serde_json::Value>(&raw) else {
        return Ok(None);
    };

    // Değer sayı değilse skor yok sayılır — ARKELÉS düzeltmeye ÇALIŞMAZ.
    let value = fm.get("value").and_then(serde_json::Value::as_f64);
    let computed_at = fm
        .get("computed_at")
        .and_then(serde_json::Value::as_str)
        .map(str::to_string);

    match (value, computed_at) {
        (Some(value), Some(computed_at)) => Ok(Some(LifeScore { value, computed_at })),
        _ => Ok(None),
    }
}

/// FTS5 üzerinden not arama — madde 27.4 (Cmd+K içinden arama).
pub fn search(conn: &Connection, query: &str, limit: u32) -> CoreResult<Vec<SearchHit>> {
    let trimmed = query.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    // FTS5 sözdizimi kullanıcı metnini operatör sanabilir. Tırnak içine alıp
    // sonuna * ekliyoruz: önek araması, operatör enjeksiyonu yok.
    let sanitized = trimmed.replace('"', " ");
    let fts_query = format!("\"{sanitized}\"*");

    let mut stmt = conn
        .prepare(
            "SELECT f.note_id, n.title, snippet(notes_fts, 2, '', '', '…', 12)
             FROM notes_fts f
             JOIN notes n ON n.id = f.note_id
             WHERE notes_fts MATCH ?1
             ORDER BY rank
             LIMIT ?2",
        )
        .map_err(CoreError::IndexQuery)?;

    let hits = stmt
        .query_map(params![fts_query, limit], |row| {
            Ok(SearchHit {
                note_id: row.get(0)?,
                title: row.get(1)?,
                snippet: row.get(2)?,
            })
        })
        .map_err(CoreError::IndexQuery)?
        .filter_map(Result::ok)
        .collect();

    Ok(hits)
}

pub fn note_count(conn: &Connection) -> CoreResult<u32> {
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
        .map_err(CoreError::IndexQuery)?;
    Ok(count as u32)
}

pub fn meta(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM index_meta WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .ok()
}

/// Görev sorgularının tek gövdesi — `notes` ile birleştirip `managed` alır.
fn tasks_where(
    conn: &Connection,
    predicate: &str,
    args: impl rusqlite::Params,
    order: &str,
    limit: Option<usize>,
) -> CoreResult<Vec<Task>> {
    let limit_clause = limit.map(|n| format!(" LIMIT {n}")).unwrap_or_default();
    let sql = format!(
        "SELECT t.id, t.title, t.status, t.note_id, t.workspace, t.due, n.managed
         FROM tasks t
         JOIN notes n ON n.id = t.note_id
         WHERE {predicate}
         ORDER BY {order}{limit_clause}"
    );

    let mut stmt = conn.prepare(&sql).map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map(args, |row| {
            Ok(Task {
                id: row.get(0)?,
                title: row.get(1)?,
                status: row.get(2)?,
                note_id: row.get(3)?,
                workspace: row.get(4)?,
                due: row.get(5)?,
                managed: row.get::<_, i64>(6)? != 0,
            })
        })
        .map_err(CoreError::IndexQuery)?;

    Ok(rows.filter_map(Result::ok).collect())
}
