/*!
Index bağlantı yönetimi — Anayasa madde 15.3.

"Index Rust çekirdeğinde, APP-DATA DİZİNİNDE tutulur. Vault'un içinde DEĞİL —
 vault kirlenmez."

Bağlantı bir Mutex arkasında. Dosya izleyici ayrı bir iş parçacığından
yazdığı için kilit gerçek bir gereksinim; WAL modu okuma/yazma çakışmasını
azaltır.
*/

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{CoreError, CoreResult};
use crate::index::{builder, query, schema, ScanReport};
use crate::types::{DashboardView, IndexStatus, SearchHit, TodayView, VaultStatus};

const INDEX_FILE: &str = "index.sqlite3";

pub struct IndexHandle {
    conn: Mutex<Connection>,
}

impl IndexHandle {
    /// Index'i açar, şemayı uygular.
    ///
    /// Madde 9.3: silinmiş bir index'i sıfırdan kurabilir — çağıran için fark yok.
    /// Madde 15.5: şema sürümü değişince içerik silinip yeniden kurulur.
    pub fn open(app: &AppHandle) -> CoreResult<Self> {
        let path = Self::file_path(app)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::Config)?;
        }

        let conn = Connection::open(&path).map_err(CoreError::IndexOpen)?;

        // Madde 34.1: sorgu bütçesi < 10 ms. WAL + normal senkron,
        // okuma ağırlıklı yükte doğru ayar.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(CoreError::IndexOpen)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(CoreError::IndexOpen)?;
        conn.pragma_update(None, "foreign_keys", true)
            .map_err(CoreError::IndexOpen)?;

        schema::apply(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    fn file_path(app: &AppHandle) -> CoreResult<PathBuf> {
        let dir = app
            .path()
            .app_data_dir()
            .map_err(|_| CoreError::AppDirUnavailable)?;
        Ok(dir.join(INDEX_FILE))
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Connection> {
        // Zehirlenmiş kilit, bir yazma sırasında panik olduğunu gösterir.
        // Index türetilmiş veri olduğu için (madde 9.4) kurtarma yolu basittir:
        // sürecin devam etmesi ve bir sonraki tam taramanın durumu düzeltmesi.
        self.conn.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    // ---- Yazma yolu (yalnız index'e; vault'a DEĞİL — madde 15.4) -----------

    pub fn full_scan(&self, vault_root: &Path) -> CoreResult<ScanReport> {
        let mut conn = self.lock();
        builder::full_scan(&mut conn, vault_root)
    }

    pub fn sync_one(&self, vault_root: &Path, path: &Path) -> CoreResult<bool> {
        let mut conn = self.lock();
        builder::sync_one(&mut conn, vault_root, path)
    }

    /// Vault değiştiğinde index'i sıfırlar. Madde 9.3: kayıpsız işlem.
    pub fn clear(&self) -> CoreResult<()> {
        let conn = self.lock();
        conn.execute_batch(
            "DELETE FROM notes_fts; DELETE FROM links; DELETE FROM tasks;
             DELETE FROM notes; DELETE FROM index_meta;",
        )
        .map_err(CoreError::IndexQuery)?;
        Ok(())
    }

    // ---- Okuma yolu -------------------------------------------------------

    pub fn today(&self) -> CoreResult<TodayView> {
        query::today(&self.lock())
    }

    pub fn dashboard(&self) -> CoreResult<DashboardView> {
        query::dashboard(&self.lock())
    }

    pub fn search(&self, q: &str, limit: u32) -> CoreResult<Vec<SearchHit>> {
        query::search(&self.lock(), q, limit)
    }

    pub fn note_count(&self) -> CoreResult<u32> {
        query::note_count(&self.lock())
    }

    pub fn vault_status(&self, path: Option<String>) -> VaultStatus {
        let conn = self.lock();
        VaultStatus {
            path,
            note_count: query::note_count(&conn).unwrap_or(0),
            indexed_at: query::meta(&conn, "indexed_at"),
        }
    }

    pub fn status(&self, rebuilding: bool) -> IndexStatus {
        let conn = self.lock();
        let version = conn
            .pragma_query_value(None, "user_version", |row| row.get::<_, i64>(0))
            .unwrap_or(0);

        IndexStatus {
            ready: version == schema::SCHEMA_VERSION,
            schema_version: version as u32,
            rebuilding,
            last_scan_ms: query::meta(&conn, "last_scan_ms").and_then(|v| v.parse().ok()),
        }
    }

    /// Tarama süresini kaydeder — madde 34.2: bütçe ölçülür ve raporlanır.
    pub fn record_scan_ms(&self, ms: u64) {
        let conn = self.lock();
        let _ = conn.execute(
            "INSERT INTO index_meta (key, value) VALUES ('last_scan_ms', ?1)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            rusqlite::params![ms.to_string()],
        );
    }
}
