/*!
İş kuyruğu — Sprint 4 madde 3.

ARKELÉS İŞİ YAPMAZ (madde 7.3). Bu modül yalnız:
  - hangi isteğin gönderildiğini kaydeder
  - Hermes'in bildirdiği durumu okur ve saklar
  - sonuç referanslarını tutar

Durum ARKELÉS tarafından TAHMİN EDİLMEZ. Hermes "running" demiyorsa iş
"running" gösterilmez.
*/

use rusqlite::{params, Connection};
use serde_json::{json, Value};

use crate::error::{CoreError, CoreResult};
use crate::hermes::{contract, rpc};
use crate::types::{Job, JobOutput};
use crate::vault::time;

/// Aktivite geçmişinde bir sayfada dönen en fazla iş.
const HISTORY_LIMIT: usize = 200;

/// Hermes'in bildirebileceği durumlar. Bunun dışı KABUL EDİLMEZ.
const VALID_STATUSES: &[&str] = &["queued", "running", "completed", "failed", "cancelled"];

fn new_job_id() -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static COUNTER: AtomicU64 = AtomicU64::new(1);
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("job-{secs}-{}", COUNTER.fetch_add(1, Ordering::Relaxed))
}

/*
 * İş gönderir — Sprint 4 madde 2, 6.
 *
 * GÜVENLİK (madde 19):
 *  - `action` ALLOWLIST'ten doğrulanır; keyfi metod adı çalıştırılamaz
 *  - kullanıcı metni PARAMETRE olarak gider, hiçbir yere interpolate edilmez
 *  - Hermes'e ulaşılamazsa iş "failed" olarak kaydedilir, sessizce kaybolmaz
 *
 * OPTİMİSTİK (madde 6): kayıt ÖNCE yapılır, Hermes çağrısı SONRA. Böylece
 * arayüz işi ilk karede görür ve Hermes'in cevabı beklenmez.
 */
pub fn submit(
    conn: &Connection,
    action_id: &str,
    workspace: Option<&str>,
    input: &str,
) -> CoreResult<Job> {
    // Madde 19: allowlist dışı aksiyon çekirdekte reddedilir.
    let action = contract::find_action(action_id).ok_or(CoreError::ActionNotAllowed)?;

    let text = input.trim();
    if text.is_empty() {
        return Err(CoreError::TargetMismatch);
    }

    let id = new_job_id();
    let now = time::now_iso();
    // Özet KISALTILIR: defterin işi başlık tutmak, metin saklamak değil.
    let summary: String = text.chars().take(120).collect();

    conn.execute(
        "INSERT INTO jobs (id, action, workspace, summary, status, created_at, source)
         VALUES (?1, ?2, ?3, ?4, 'queued', ?5, 'arkeles')",
        params![id, action.id, workspace, summary, now],
    )
    .map_err(CoreError::IndexQuery)?;

    Ok(read_one(conn, &id)?)
}

/*
 * Kuyruktaki işi Hermes'e iletir — arka planda çalışır.
 *
 * Kullanıcı metni `params` içinde GİDER; hiçbir komut satırına, hiçbir
 * dizgeye interpolate EDİLMEZ (Sprint 4 madde 2).
 */
pub fn dispatch(conn: &Connection, job_id: &str) -> CoreResult<()> {
    let job = read_one(conn, job_id)?;
    let Some(action) = contract::find_action(&job.action) else {
        return fail(conn, job_id, "action_not_allowed", "Aksiyon tanımlı değil");
    };

    let params = json!({
        "action": action.id,
        "workspace": job.workspace,
        "input": job.summary,
        "origin": "arkeles",
    });

    /*
     * Hermes'in metodu: iş gönderme için `command.dispatch` kullanılır
     * (yerel kurulumdan doğrulanan metod ailesi). Metod adı SABİT KODLU —
     * arayüzden gelmez.
     */
    match rpc::call("command.dispatch", params) {
        Ok(result) => {
            let hermes_ref = result
                .get("session_id")
                .or_else(|| result.get("id"))
                .and_then(Value::as_str)
                .map(str::to_string);

            conn.execute(
                "UPDATE jobs SET status = 'running', started_at = ?2, hermes_ref = ?3
                 WHERE id = ?1 AND status = 'queued'",
                params![job_id, time::now_iso(), hermes_ref],
            )
            .map_err(CoreError::IndexQuery)?;
            Ok(())
        }
        Err(err) => {
            // Madde 15 (hata doktrini): hata SESSİZCE KAYBOLMAZ.
            let (code, message) = describe(&err);
            fail(conn, job_id, code, &message)
        }
    }
}

/// Hermes'ten bir işin güncel durumunu çeker ve deftere yazar.
///
/// Hermes tanımadığı bir durum bildirirse GÖRMEZDEN GELİNİR: uydurma
/// durum deftere girmez.
pub fn refresh(conn: &Connection, job_id: &str) -> CoreResult<()> {
    let job = read_one(conn, job_id)?;
    let Some(reference) = job_ref(conn, job_id)? else {
        return Ok(()); // henüz Hermes'e ulaşmadı
    };
    if matches!(job.status.as_str(), "completed" | "failed" | "cancelled") {
        return Ok(()); // bitmiş iş tekrar sorulmaz
    }

    let Ok(result) = rpc::call("session.status", json!({ "session_id": reference })) else {
        return Ok(()); // ulaşılamadı — madde 18.4: hata değil, sakin durum
    };

    apply_remote_status(conn, job_id, &result)
}

/// Hermes'in cevabını deftere uygular. Tanınmayan alanlar YOK SAYILIR.
pub fn apply_remote_status(conn: &Connection, job_id: &str, result: &Value) -> CoreResult<()> {
    let status = result.get("status").and_then(Value::as_str).unwrap_or("");
    if !VALID_STATUSES.contains(&status) {
        return Ok(()); // uydurma durum deftere girmez
    }

    // Sprint 4 madde 7: SAHTE PROGRESS ÜRETİLMEZ. Hermes sayı vermiyorsa None.
    let progress = result
        .get("progress")
        .and_then(Value::as_f64)
        .filter(|p| (0.0..=100.0).contains(p))
        .map(|p| p as i64);

    let finished = matches!(status, "completed" | "failed" | "cancelled");

    conn.execute(
        "UPDATE jobs SET status = ?2, progress = ?3,
                finished_at = CASE WHEN ?4 THEN ?5 ELSE finished_at END
         WHERE id = ?1",
        params![job_id, status, progress, finished, time::now_iso()],
    )
    .map_err(CoreError::IndexQuery)?;

    if status == "failed" {
        let message = result
            .get("error")
            .and_then(Value::as_str)
            .unwrap_or("Hermes işi tamamlayamadı")
            .chars()
            .take(200)
            .collect::<String>();
        conn.execute(
            "UPDATE jobs SET error_code = 'hermes_failed', error_message = ?2 WHERE id = ?1",
            params![job_id, message],
        )
        .map_err(CoreError::IndexQuery)?;
    }

    // Sonuç referansları — madde 8. ARKELÉS sonuç UYDURMAZ.
    if let Some(outputs) = result.get("outputs").and_then(Value::as_array) {
        conn.execute("DELETE FROM job_outputs WHERE job_id = ?1", params![job_id])
            .map_err(CoreError::IndexQuery)?;

        for output in outputs {
            let kind = output.get("kind").and_then(Value::as_str).unwrap_or("");
            if !matches!(kind, "note" | "document" | "external") {
                continue; // tanınmayan tür saklanmaz
            }
            let Some(reference) = output.get("ref").and_then(Value::as_str) else {
                continue;
            };
            let label = output
                .get("label")
                .and_then(Value::as_str)
                .unwrap_or(reference);

            conn.execute(
                "INSERT OR REPLACE INTO job_outputs (job_id, kind, ref, label)
                 VALUES (?1, ?2, ?3, ?4)",
                params![job_id, kind, reference, label],
            )
            .map_err(CoreError::IndexQuery)?;
        }
    }

    Ok(())
}

/// İşi iptal eder — Sprint 4 madde 16. Hermes desteklemiyorsa hata döner
/// ve arayüz butonu zaten göstermez (capability-driven).
pub fn cancel(conn: &Connection, job_id: &str) -> CoreResult<()> {
    let Some(reference) = job_ref(conn, job_id)? else {
        // Henüz gönderilmemiş: yerel olarak iptal edilebilir.
        conn.execute(
            "UPDATE jobs SET status = 'cancelled', finished_at = ?2
             WHERE id = ?1 AND status = 'queued'",
            params![job_id, time::now_iso()],
        )
        .map_err(CoreError::IndexQuery)?;
        return Ok(());
    };

    rpc::call("session.cancel", json!({ "session_id": reference }))?;

    conn.execute(
        "UPDATE jobs SET status = 'cancelled', finished_at = ?2 WHERE id = ?1",
        params![job_id, time::now_iso()],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
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

fn fail(conn: &Connection, job_id: &str, code: &str, message: &str) -> CoreResult<()> {
    conn.execute(
        "UPDATE jobs SET status = 'failed', finished_at = ?2, error_code = ?3, error_message = ?4
         WHERE id = ?1",
        params![job_id, time::now_iso(), code, message],
    )
    .map_err(CoreError::IndexQuery)?;
    Ok(())
}

/// Hata → (kod, kullanıcıya gösterilebilir kısa mesaj).
///
/// Sprint 4 madde 19: stack trace ASLA kullanıcıya gitmez.
fn describe(err: &CoreError) -> (&'static str, String) {
    match err {
        CoreError::HermesUnreachable => ("hermes_unreachable", "Hermes'e ulaşılamadı.".into()),
        CoreError::HermesUnauthorized => (
            "hermes_unauthorized",
            "Hermes kimlik doğrulaması başarısız.".into(),
        ),
        CoreError::HermesTimeout => ("hermes_timeout", "Hermes zamanında yanıt vermedi.".into()),
        CoreError::HermesRejected(message) => ("hermes_rejected", message.clone()),
        CoreError::ActionNotAllowed => ("action_not_allowed", "Bu aksiyon tanımlı değil.".into()),
        _ => ("unknown", "İş gönderilemedi.".into()),
    }
}

fn job_ref(conn: &Connection, job_id: &str) -> CoreResult<Option<String>> {
    conn.query_row(
        "SELECT hermes_ref FROM jobs WHERE id = ?1",
        params![job_id],
        |row| row.get::<_, Option<String>>(0),
    )
    .map_err(|_| CoreError::TargetMismatch)
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
                outputs: Vec::new(),
            })
        })
        .map_err(CoreError::IndexQuery)?;

    let mut jobs = rows
        .collect::<rusqlite::Result<Vec<Job>>>()
        .map_err(CoreError::IndexQuery)?;

    for job in &mut jobs {
        job.outputs = outputs_of(conn, &job.id)?;
    }
    Ok(jobs)
}

fn outputs_of(conn: &Connection, job_id: &str) -> CoreResult<Vec<JobOutput>> {
    let mut stmt = conn
        .prepare("SELECT kind, ref, label FROM job_outputs WHERE job_id = ?1 ORDER BY kind, ref")
        .map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map(params![job_id], |row| {
            Ok(JobOutput {
                kind: row.get(0)?,
                reference: row.get(1)?,
                label: row.get(2)?,
            })
        })
        .map_err(CoreError::IndexQuery)?;
    rows.collect::<rusqlite::Result<Vec<JobOutput>>>()
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

/// Kuyrukta bekleyen işlerin kimlikleri — arka plan gönderici için.
pub fn queued_ids(conn: &Connection) -> CoreResult<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT id FROM jobs WHERE status = 'queued' ORDER BY created_at ASC LIMIT 10")
        .map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(CoreError::IndexQuery)?;
    rows.collect::<rusqlite::Result<Vec<String>>>()
        .map_err(CoreError::IndexQuery)
}

/// Durumu Hermes'ten tazelenmesi gereken işler.
pub fn running_ids(conn: &Connection) -> CoreResult<Vec<String>> {
    let mut stmt = conn
        .prepare("SELECT id FROM jobs WHERE status = 'running' ORDER BY created_at ASC LIMIT 20")
        .map_err(CoreError::IndexQuery)?;
    let rows = stmt
        .query_map([], |row| row.get::<_, String>(0))
        .map_err(CoreError::IndexQuery)?;
    rows.collect::<rusqlite::Result<Vec<String>>>()
        .map_err(CoreError::IndexQuery)
}

#[cfg(test)]
mod tests;
