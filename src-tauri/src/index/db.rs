/*!
Index bağlantı yönetimi — Anayasa madde 15.3, 34.1.

"Index Rust çekirdeğinde, APP-DATA DİZİNİNDE tutulur. Vault'un içinde DEĞİL."

OKUMA/YAZMA AYRIMI (Sprint 2'de eklendi — Sprint 1 borcu #1):
Sprint 1'de tek bir `Mutex<Connection>` vardı ve açılış taraması onu tuttuğu
için arayüzün ilk okumaları taramanın bitmesini bekliyordu (p95 ~400 ms).

Çözüm: iki ayrı bağlantı. SQLite WAL modunda BİR yazar ile ÇOK okuyucu
eşzamanlı çalışabilir; okuyucu, yazarın işini beklemek yerine son
COMMIT'lenmiş anlık görüntüyü okur. Böylece madde 21.3 gerçekten sağlanır:
"ilk görünen şey son bilinen veridir, arkada tazelenir."
*/

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{CoreError, CoreResult};
use crate::index::{builder, query, schema, work, ScanReport};
use crate::types::{DashboardView, IndexStatus, SearchHit, TodayView, VaultStatus};

const INDEX_FILE: &str = "index.sqlite3";

pub struct IndexHandle {
    /// Yazma yolu: tarama ve artımlı güncelleme. Tek yazar (WAL gereği).
    write_conn: Mutex<Connection>,
    /// Okuma yolu: sorgular. Yazarı BEKLEMEZ.
    read_conn: Mutex<Connection>,
    /// Sprint 1 borcu #4: sessizce yutulan satır hataları artık SAYILIYOR.
    row_errors: AtomicU64,
}

impl IndexHandle {
    pub fn open(app: &AppHandle) -> CoreResult<Self> {
        let path = Self::file_path(app)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::Config)?;
        }

        let write_conn = Self::open_connection(&path, true)?;
        schema::apply(&write_conn)?;

        // Şema uygulandıktan SONRA açılır ki okuyucu hazır bir dosya görsün.
        let read_conn = Self::open_connection(&path, false)?;

        Ok(Self {
            write_conn: Mutex::new(write_conn),
            read_conn: Mutex::new(read_conn),
            row_errors: AtomicU64::new(0),
        })
    }

    fn open_connection(path: &Path, writer: bool) -> CoreResult<Connection> {
        let conn = Connection::open(path).map_err(CoreError::IndexOpen)?;

        // WAL: okuyucu/yazıcı eşzamanlılığının şartı.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(CoreError::IndexOpen)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(CoreError::IndexOpen)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(CoreError::IndexOpen)?;

        if writer {
            // Yazar, başka bir yazarla çakışırsa kısa süre bekler, hemen
            // hata vermez. Tek yazarımız var; bu yalnız emniyet payı.
            conn.busy_timeout(std::time::Duration::from_millis(3_000))
                .map_err(CoreError::IndexOpen)?;
        } else {
            // Okuyucu ASLA uzun beklemez — madde 34.1: sorgu bütçesi 10 ms.
            // WAL'de okuyucu zaten kilitlenmez; bu değer son emniyet.
            conn.busy_timeout(std::time::Duration::from_millis(50))
                .map_err(CoreError::IndexOpen)?;
        }

        Ok(conn)
    }

    fn file_path(app: &AppHandle) -> CoreResult<PathBuf> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|_| CoreError::AppDirUnavailable)?;
        Ok(dir.join(INDEX_FILE))
    }

    fn writer(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.write_conn.lock().unwrap_or_else(|p| p.into_inner())
    }

    fn reader(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.read_conn.lock().unwrap_or_else(|p| p.into_inner())
    }

    /// Sprint 1 borcu #4: satır hatası sayacı. Sorgular buraya rapor eder.
    fn count_row_errors(&self, n: u64) {
        if n > 0 {
            self.row_errors.fetch_add(n, Ordering::Relaxed);
        }
    }

    // ---- Yazma yolu (yalnız index'e; vault'a DEĞİL — madde 15.4) -----------

    pub fn full_scan(&self, vault_root: &Path) -> CoreResult<ScanReport> {
        let mut conn = self.writer();
        let report = builder::full_scan(&mut conn, vault_root)?;
        self.count_row_errors(report.row_errors as u64);
        Ok(report)
    }

    pub fn sync_one(&self, vault_root: &Path, path: &Path) -> CoreResult<bool> {
        let mut conn = self.writer();
        builder::sync_one(&mut conn, vault_root, path)
    }

    /// Vault değiştiğinde index'i sıfırlar. Madde 9.3: kayıpsız işlem.
    pub fn clear(&self) -> CoreResult<()> {
        let conn = self.writer();
        conn.execute_batch(
            "DELETE FROM notes_fts; DELETE FROM links; DELETE FROM tasks;
             DELETE FROM notes; DELETE FROM index_meta;",
        )
        .map_err(CoreError::IndexQuery)?;
        Ok(())
    }

    // ---- Okuma yolu -------------------------------------------------------

    pub fn today(&self) -> CoreResult<TodayView> {
        let (view, errors) = query::today(&self.reader())?;
        self.count_row_errors(errors);
        Ok(view)
    }

    pub fn dashboard(&self) -> CoreResult<DashboardView> {
        let (view, errors) = query::dashboard(&self.reader())?;
        self.count_row_errors(errors);
        Ok(view)
    }

    pub fn search(&self, q: &str, limit: u32) -> CoreResult<Vec<SearchHit>> {
        let (hits, errors) = query::search(&self.reader(), q, limit)?;
        self.count_row_errors(errors);
        Ok(hits)
    }

    pub fn note_count(&self) -> CoreResult<u32> {
        query::note_count(&self.reader())
    }

    /// Bir notun mekanik mutasyon için gereken kimlik bilgisi (madde 16.3).
    pub fn note_write_target(&self, note_id: &str) -> CoreResult<query::WriteTarget> {
        query::write_target(&self.reader(), note_id)
    }

    /// Bir görevin index'teki hali — mutasyon hedefini doğrulamak için.
    pub fn task_write_target(&self, task_id: &str) -> CoreResult<query::TaskTarget> {
        query::task_target(&self.reader(), task_id)
    }

    pub fn vault_status(&self, path: Option<String>) -> VaultStatus {
        let conn = self.reader();
        VaultStatus {
            path,
            note_count: query::note_count(&conn).unwrap_or(0),
            indexed_at: query::meta(&conn, "indexed_at"),
        }
    }

    pub fn status(&self, rebuilding: bool) -> IndexStatus {
        let conn = self.reader();
        let version = conn
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap_or(0);

        IndexStatus {
            ready: version == schema::SCHEMA_VERSION,
            schema_version: version as u32,
            rebuilding,
            last_scan_ms: query::meta(&conn, "last_scan_ms").and_then(|v| v.parse().ok()),
            // Sprint 1 borcu #4: hatalar artık GÖRÜNÜR.
            row_errors: self.row_errors.load(Ordering::Relaxed) as u32,
        }
    }

    /// Tarama süresini kaydeder — madde 34.2: bütçe ölçülür ve raporlanır.
    pub fn record_scan_ms(&self, ms: u64) {
        let conn = self.writer();
        let _ = conn.execute(
            "INSERT INTO index_meta (key, value) VALUES ('last_scan_ms', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![ms.to_string()],
        );
    }
}

impl IndexHandle {
    /// Yol üzerinden not kimliği. Yazma sonrası arayüze doğru kimliği
    /// döndürmek için (madde 16.2: yol kimlik değil, ama kimliğe köprü).
    pub fn note_id_by_path(&self, source_path: &str) -> Option<String> {
        self.reader()
            .query_row(
                "SELECT id FROM notes WHERE source_path = ?1",
                rusqlite::params![source_path],
                |row| row.get::<_, String>(0),
            )
            .ok()
    }
}

/*
 * İş modülü okuma yolu — Sprint 3.
 *
 * Hepsi OKUMA bağlantısını kullanır: tarama sürerken de cevap verirler
 * (Sprint 1 borcu #1).
 */
impl IndexHandle {
    pub fn workspaces(&self) -> CoreResult<Vec<crate::types::Workspace>> {
        work::list_workspaces(&self.reader())
    }

    pub fn workspace_overview(
        &self,
        id: &str,
    ) -> CoreResult<crate::types::WorkspaceOverview> {
        let (view, errors) = work::overview(&self.reader(), id)?;
        self.count_row_errors(errors);
        Ok(view)
    }

    pub fn workspace_tasks(&self, id: &str) -> CoreResult<Vec<crate::types::Task>> {
        let (rows, errors) = work::tasks(&self.reader(), id)?;
        self.count_row_errors(errors);
        Ok(rows)
    }

    pub fn workspace_notes(
        &self,
        id: &str,
        kind: &str,
    ) -> CoreResult<Vec<crate::types::NoteSummary>> {
        let (rows, errors) = work::notes(&self.reader(), id, kind)?;
        self.count_row_errors(errors);
        Ok(rows)
    }

    pub fn workspace_meetings(
        &self,
        id: &str,
    ) -> CoreResult<Vec<crate::types::MeetingSummary>> {
        let (rows, errors) = work::meetings(&self.reader(), id)?;
        self.count_row_errors(errors);
        Ok(rows)
    }

    pub fn workspace_documents(
        &self,
        id: &str,
    ) -> CoreResult<Vec<crate::types::DocumentRef>> {
        let (rows, errors) = work::documents(&self.reader(), id)?;
        self.count_row_errors(errors);
        Ok(rows)
    }

    pub fn note_detail(&self, note_id: &str) -> CoreResult<crate::types::NoteDetail> {
        work::note_detail(&self.reader(), note_id)
    }
}

impl IndexHandle {
    /// Belge yolunu index'ten çözer. Arayüzden gelen yola GÜVENİLMEZ (madde 19).
    pub fn document_path(&self, document_id: &str) -> Option<String> {
        self.reader()
            .query_row(
                "SELECT source_path FROM documents WHERE id = ?1",
                rusqlite::params![document_id],
                |row| row.get::<_, String>(0),
            )
            .ok()
    }
}

/*
 * Hermes iş kuyruğu — Sprint 4.
 *
 * Defter index veritabanında yaşar (madde 9.3: silme testi jobs tablosunun
 * başında açıklandı). Yazma bağlantısını kullanır; okuma yolu okuma
 * bağlantısını — böylece tarama sürerken de kuyruk görünür.
 */
impl IndexHandle {
    pub fn submit_job(
        &self,
        action: &str,
        workspace: Option<&str>,
        input: &str,
    ) -> CoreResult<crate::types::Job> {
        let conn = self.writer();
        crate::hermes::jobs::submit(&conn, action, workspace, input)
    }

    pub fn active_jobs(&self) -> CoreResult<Vec<crate::types::Job>> {
        crate::hermes::jobs::active(&self.reader())
    }

    pub fn job_history(
        &self,
        since: &str,
        workspace: Option<&str>,
        status: Option<&str>,
    ) -> CoreResult<Vec<crate::types::Job>> {
        crate::hermes::jobs::history(&self.reader(), since, workspace, status)
    }

    pub fn retry_job(&self, job_id: &str) -> CoreResult<()> {
        let conn = self.writer();
        crate::hermes::jobs::retry(&conn, job_id)
    }

    pub fn cancel_job(&self, job_id: &str) -> CoreResult<()> {
        let conn = self.writer();
        crate::hermes::jobs::cancel(&conn, job_id)
    }

    pub fn hermes_summary(
        &self,
        health: crate::types::HermesHealth,
    ) -> CoreResult<crate::types::HermesSummary> {
        crate::hermes::summary(&self.reader(), health)
    }

    /// Arka plan sürücüsü: kuyruktaki işleri gönderir, çalışanları tazeler.
    ///
    /// Değişiklik olduysa `true` — çağıran arayüze olay yayınlar.
    pub fn drive_jobs(&self) -> bool {
        let mut changed = false;

        let queued = {
            let conn = self.reader();
            crate::hermes::jobs::queued_ids(&conn).unwrap_or_default()
        };
        for job_id in queued {
            let conn = self.writer();
            if crate::hermes::jobs::dispatch(&conn, &job_id).is_ok() {
                changed = true;
            } else {
                // dispatch kendi hatasını deftere yazdı; yine de değişiklik.
                changed = true;
            }
        }

        let running = {
            let conn = self.reader();
            crate::hermes::jobs::running_ids(&conn).unwrap_or_default()
        };
        for job_id in running {
            let conn = self.writer();
            if crate::hermes::jobs::refresh(&conn, &job_id).is_ok() {
                changed = true;
            }
        }

        changed
    }
}
