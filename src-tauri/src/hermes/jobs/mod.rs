/*!
İş defteri — Sprint 4 madde 3; Sprint 5'te GERÇEK Hermes turuna bağlandı.

ARKELÉS İŞİ YAPMAZ (madde 7.3). Bu modül yalnız:
  - hangi isteğin gönderildiğini kaydeder
  - Hermes'in bildirdiği sonucu deftere yazar

Durum ARKELÉS tarafından TAHMİN EDİLMEZ; her geçişin tek bir kaynağı var:
  queued     ARKELÉS kaydetti; Hermes turu henüz kabul etmedi
  running    Hermes istemi kabul etti (`turn::run` → on_started)
  completed  Hermes turun bittiğini bildirdi
  failed     Hermes hata bildirdi YA DA Hermes'e ulaşılamadı (kod ayırır)
  cancelled  kullanıcı iptal etti; Hermes turu kesti ya da tur hiç başlamadı

Hermes'e özgü adlar (metod, olay) BURADA YOK — hepsi `turn.rs`'te.
*/

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};

use rusqlite::{params, Connection, OptionalExtension};

use crate::config::workspaces::WORKSPACES;
use crate::error::{CoreError, CoreResult};
use crate::hermes::contract;
use crate::hermes::turn::Outcome;
use crate::types::Job;
use crate::vault::time;

/// Aktivite geçmişinde bir sayfada dönen en fazla iş.
const HISTORY_LIMIT: usize = 200;
/// Listede görünen başlık uzunluğu.
const SUMMARY_CHARS: usize = 120;
/// Hermes'e giden metin sınırı — sınırsız metin defteri ve Hermes'i şişirir.
const MAX_INPUT_CHARS: usize = 8000;
/// Aynı anda yürüyen tur sayısı: ARKELÉS Hermes oturumlarını tüketmesin.
const MAX_CONCURRENT: usize = 2;

fn new_job_id() -> String {
    use std::sync::atomic::AtomicU64;
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("job-{secs}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

// --- uçuştaki işler (süreç içi) ----------------------------------------------

/// Turu şu an bu süreçte yürüyen işler. Aynı iş İKİ KEZ gönderilmez.
static IN_FLIGHT: Mutex<Vec<String>> = Mutex::new(Vec::new());
/// Defter değişti mi? Sürücü bunu görünce arayüze tek olay yayınlar.
static DIRTY: AtomicBool = AtomicBool::new(false);

fn in_flight() -> MutexGuard<'static, Vec<String>> {
    IN_FLIGHT.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Kuyruktaki bir sonraki işi sahiplenir. Sınır doluysa veya iş yoksa `None`.
pub fn claim_next(conn: &Connection) -> CoreResult<Option<String>> {
    let queued = queued_ids(conn)?;
    let mut flight = in_flight();
    if flight.len() >= MAX_CONCURRENT {
        return Ok(None);
    }
    let next = queued.into_iter().find(|id| !flight.contains(id));
    if let Some(id) = &next {
        flight.push(id.clone());
    }
    Ok(next)
}

pub fn release(job_id: &str) {
    in_flight().retain(|id| id != job_id);
}

pub fn mark_dirty() {
    DIRTY.store(true, Ordering::Relaxed);
}

pub fn take_dirty() -> bool {
    DIRTY.swap(false, Ordering::Relaxed)
}

// --- gönderme ------------------------------------------------------------------

/*
 * İş kaydeder — Sprint 4 madde 2, 6.
 *
 * GÜVENLİK (madde 19):
 *  - `action` ve `workspace` ALLOWLIST'ten doğrulanır
 *  - kullanıcı metni düz metin olarak saklanır ve gider; hiçbir yere
 *    komut olarak interpolate edilmez
 *
 * OPTİMİSTİK (madde 6): yalnız kayıt yapılır ve hemen döner; Hermes turu
 * arka plan sürücüsünde yürür.
 */
pub fn submit(
    conn: &Connection,
    action_id: &str,
    workspace: Option<&str>,
    input: &str,
) -> CoreResult<Job> {
    let action = contract::find_action(action_id).ok_or(CoreError::ActionNotAllowed)?;

    // İstem başlığına giren her şey bilinen bir değer olmalı.
    if let Some(workspace) = workspace {
        if !WORKSPACES.iter().any(|w| w.id == workspace) {
            return Err(CoreError::TargetMismatch);
        }
    }

    let text = input.trim();
    if text.is_empty() {
        return Err(CoreError::TargetMismatch);
    }
    let text: String = text.chars().take(MAX_INPUT_CHARS).collect();
    let summary: String = text.chars().take(SUMMARY_CHARS).collect();

    let id = new_job_id();
    conn.execute(
        "INSERT INTO jobs (id, action, workspace, summary, input, status, created_at, source)
         VALUES (?1, ?2, ?3, ?4, ?5, 'queued', ?6, 'arkeles')",
        params![id, action.id, workspace, summary, text, time::now_iso()],
    )
    .map_err(CoreError::IndexQuery)?;

    read_one(conn, &id)
}

/// Hermes'e gidecek başlık ve metin.
pub struct Prepared {
    pub title: String,
    pub prompt: String,
}

/// Kuyruktaki işin istemini hazırlar. İş artık kuyrukta değilse (iptal
/// edildi) `TargetMismatch` — tur hiç başlamaz.
pub fn prepare(conn: &Connection, job_id: &str) -> CoreResult<Prepared> {
    let (action, workspace, input): (String, Option<String>, String) = conn
        .query_row(
            "SELECT action, workspace, input FROM jobs WHERE id = ?1 AND status = 'queued'",
            params![job_id],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .optional()
        .map_err(CoreError::IndexQuery)?
        .ok_or(CoreError::TargetMismatch)?;

    let action = contract::find_action(&action).ok_or(CoreError::ActionNotAllowed)?;
    Ok(Prepared {
        title: contract::session_title(action, workspace.as_deref()),
        prompt: contract::prompt_for(action, workspace.as_deref(), &input),
    })
}

/// Hermes istemi kabul etti → running. İş bu arada iptal edildiyse `false`
/// döner ve çağıran turu Hermes'te de keser.
pub fn mark_started(conn: &Connection, job_id: &str, session_ref: &str) -> CoreResult<bool> {
    let changed = conn
        .execute(
            "UPDATE jobs SET status = 'running', started_at = ?2, hermes_ref = ?3
             WHERE id = ?1 AND status = 'queued'",
            params![job_id, time::now_iso(), session_ref],
        )
        .map_err(CoreError::IndexQuery)?;
    Ok(changed == 1)
}

/// Hermes'in bildirdiği sonucu yazar. Bitmiş iş YENİDEN YAZILMAZ.
pub fn apply_outcome(conn: &Connection, job_id: &str, outcome: &Outcome) -> CoreResult<()> {
    let (status, code, message) = match outcome {
        Outcome::Completed => ("completed", None, None),
        Outcome::Cancelled => ("cancelled", None, None),
        Outcome::Failed { code, message } => ("failed", Some(*code), Some(message.as_str())),
    };
    conn.execute(
        "UPDATE jobs SET status = ?2, finished_at = ?3, error_code = ?4, error_message = ?5
         WHERE id = ?1 AND status IN ('queued', 'running')",
        params![job_id, status, time::now_iso(), code, message],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
}

/// Bağlantı düzeyi hata → failed. Madde 15: hata SESSİZCE KAYBOLMAZ.
pub fn fail(conn: &Connection, job_id: &str, err: &CoreError) -> CoreResult<()> {
    let (code, message) = describe(err);
    conn.execute(
        "UPDATE jobs SET status = 'failed', finished_at = ?2, error_code = ?3, error_message = ?4
         WHERE id = ?1 AND status IN ('queued', 'running')",
        params![job_id, time::now_iso(), code, message],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
}

/*
 * İptal — Sprint 4 madde 16.
 *
 * queued  → yerel olarak iptal edilir (tur başladıysa sürücü onu da keser)
 * running → Hermes oturum kimliği döner; çağıran turu Hermes'te keser ve
 *           "cancelled" durumunu HERMES bildirir — ARKELÉS önceden yazmaz
 */
pub fn cancel(conn: &Connection, job_id: &str) -> CoreResult<Option<String>> {
    let (status, reference): (String, Option<String>) = conn
        .query_row(
            "SELECT status, hermes_ref FROM jobs WHERE id = ?1",
            params![job_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )
        .optional()
        .map_err(CoreError::IndexQuery)?
        .ok_or(CoreError::TargetMismatch)?;

    match status.as_str() {
        "queued" => {
            conn.execute(
                "UPDATE jobs SET status = 'cancelled', finished_at = ?2
                 WHERE id = ?1 AND status = 'queued'",
                params![job_id, time::now_iso()],
            )
            .map_err(CoreError::IndexQuery)?;
            Ok(None)
        }
        "running" => Ok(reference),
        _ => Ok(None),
    }
}

/// Başarısız işi yeniden kuyruğa alır — Sprint 4 madde 15.
///
/// OTOMATİK SONSUZ RETRY YOK: bu yalnız kullanıcı istediğinde çağrılır.
pub fn retry(conn: &Connection, job_id: &str) -> CoreResult<()> {
    conn.execute(
        "UPDATE jobs SET status = 'queued', started_at = NULL, finished_at = NULL,
                progress = NULL, hermes_ref = NULL, error_code = NULL, error_message = NULL
         WHERE id = ?1 AND status = 'failed'",
        params![job_id],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
}

/*
 * Açılışta yetim işler.
 *
 * Tur bağlantıya bağlıdır ve bağlantı önceki süreçle kapandı: sonucu ARKELÉS
 * artık öğrenemez. "Tamamlandı" demek de "sürüyor" demek de UYDURMA olur;
 * dürüst olan başarısız + açıklama (kullanıcı isterse yeniden dener).
 */
pub fn recover_orphans(conn: &Connection) -> CoreResult<usize> {
    conn.execute(
        "UPDATE jobs SET status = 'failed', finished_at = ?1, error_code = 'hermes_unreachable',
                error_message = 'Uygulama kapanırken iş sürüyordu; Hermes sonucu bildirmedi.'
         WHERE status = 'running'",
        params![time::now_iso()],
    )
    .map_err(CoreError::IndexQuery)
}

/// Hata → (kod, kullanıcıya gösterilebilir kısa mesaj).
///
/// Sprint 4 madde 19: stack trace ASLA kullanıcıya gitmez.
fn describe(err: &CoreError) -> (&'static str, String) {
    let message = match err {
        CoreError::HermesUnreachable => "Hermes'e ulaşılamadı.".to_string(),
        CoreError::HermesUnauthorized => "Hermes kimlik doğrulaması başarısız.".to_string(),
        CoreError::HermesTimeout => "Hermes zamanında yanıt vermedi.".to_string(),
        CoreError::HermesRejected(message) => message.clone(),
        CoreError::HermesMalformed => "Hermes beklenmeyen bir cevap verdi.".to_string(),
        CoreError::ActionNotAllowed => "Bu aksiyon tanımlı değil.".to_string(),
        _ => return ("unknown", "İş gönderilemedi.".into()),
    };
    (err.code(), message)
}

#[cfg(test)]
fn job_ref(conn: &Connection, job_id: &str) -> Option<String> {
    conn.query_row(
        "SELECT hermes_ref FROM jobs WHERE id = ?1",
        params![job_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .ok()
    .flatten()
}

// --- okuma -------------------------------------------------------------------

pub fn read_one(conn: &Connection, job_id: &str) -> CoreResult<Job> {
    let mut jobs = query(conn, "WHERE j.id = ?1", params![job_id], 1)?;
    if jobs.is_empty() {
        return Err(CoreError::TargetMismatch);
    }
    Ok(jobs.remove(0))
}

/// Aktif işler (queued + running) — kuyruk yüzeyi.
pub fn active(conn: &Connection) -> CoreResult<Vec<Job>> {
    query(
        conn,
        "WHERE j.status IN ('queued','running')",
        params![],
        HISTORY_LIMIT,
    )
}

/// Geçmiş — Sprint 4 madde 14. `since` ISO tarihi (dahil).
pub fn history(
    conn: &Connection,
    since: &str,
    workspace: Option<&str>,
    status: Option<&str>,
) -> CoreResult<Vec<Job>> {
    let mut clause = String::from("WHERE substr(j.created_at, 1, 10) >= ?1");
    if workspace.is_some() {
        clause.push_str(" AND j.workspace = ?2");
    }
    if status.is_some() {
        clause.push_str(if workspace.is_some() {
            " AND j.status = ?3"
        } else {
            " AND j.status = ?2"
        });
    }

    match (workspace, status) {
        (Some(w), Some(s)) => query(conn, &clause, params![since, w, s], HISTORY_LIMIT),
        (Some(w), None) => query(conn, &clause, params![since, w], HISTORY_LIMIT),
        (None, Some(s)) => query(conn, &clause, params![since, s], HISTORY_LIMIT),
        (None, None) => query(conn, &clause, params![since], HISTORY_LIMIT),
    }
}

fn query(
    conn: &Connection,
    clause: &str,
    args: impl rusqlite::Params,
    limit: usize,
) -> CoreResult<Vec<Job>> {
    let sql = format!(
        "SELECT j.id, j.action, j.workspace, j.summary, j.status, j.created_at,
                j.started_at, j.finished_at, j.progress, j.error_code, j.error_message, j.source
         FROM jobs j {clause}
         ORDER BY j.created_at DESC
         LIMIT {limit}"
    );

    let mut stmt = conn.prepare(&sql).map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map(args, |row| {
            let action: String = row.get(1)?;
            Ok(Job {
                action_label: contract::find_action(&action)
                    .map(|a| a.label.to_string())
                    .unwrap_or_else(|| action.clone()),
                id: row.get(0)?,
                action,
                workspace: row.get(2)?,
                summary: row.get(3)?,
                status: row.get(4)?,
                created_at: row.get(5)?,
                started_at: row.get(6)?,
                finished_at: row.get(7)?,
                progress: row.get::<_, Option<i64>>(8)?.map(|p| p as u32),
                error_code: row.get(9)?,
                error_message: row.get(10)?,
                source: row.get(11)?,
            })
        })
        .map_err(CoreError::IndexQuery)?;

    rows.collect::<rusqlite::Result<Vec<Job>>>()
        .map_err(CoreError::IndexQuery)
}

/// Dashboard özeti için sayımlar — Sprint 4 madde 4, 13.
pub fn counts(conn: &Connection) -> CoreResult<(u32, u32, u32, u32)> {
    let today = time::today_iso();
    let count = |predicate: &str, arg: Option<&str>| -> i64 {
        let sql = format!("SELECT COUNT(*) FROM jobs WHERE {predicate}");
        match arg {
            Some(a) => conn.query_row(&sql, params![a], |r| r.get(0)).unwrap_or(0),
            None => conn.query_row(&sql, [], |r| r.get(0)).unwrap_or(0),
        }
    };

    Ok((
        count("status = 'running'", None) as u32,
        count("status = 'queued'", None) as u32,
        count(
            "status = 'completed' AND substr(created_at, 1, 10) = ?1",
            Some(&today),
        ) as u32,
        count(
            "status = 'failed' AND substr(created_at, 1, 10) = ?1",
            Some(&today),
        ) as u32,
    ))
}

pub fn last_completed(conn: &Connection) -> CoreResult<Option<Job>> {
    let mut jobs = query(conn, "WHERE j.status = 'completed'", params![], 1)?;
    Ok(if jobs.is_empty() { None } else { Some(jobs.remove(0)) })
}

/// Kuyrukta bekleyen işlerin kimlikleri — arka plan sürücüsü için.
fn queued_ids(conn: &Connection) -> CoreResult<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT id FROM jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 10")
        .map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(CoreError::IndexQuery)?;
    rows.collect::<rusqlite::Result<Vec<String>>>()
        .map_err(CoreError::IndexQuery)
}

#[cfg(test)]
mod tests;
