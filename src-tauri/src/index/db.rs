/*!
Index bağlantı yönetimi — Anayasa madde 15.3.

"Index Rust çekirdeğinde, APP-DATA DİZİNİNDE tutulur. Vault'un içinde DEĞİL —
 vault kirlenmez."

Bağlantı bir Mutex arkasında tutulur. Sprint 0'da tek okuyucu var; Sprint 1'de
dosya izleyici ayrı bir yazar olacak ve o zaman WAL modu + ayrı bağlantı havuzu
değerlendirilecek. Şimdi havuz kurmak erken optimizasyondur.
*/

use std::path::PathBuf;
use std::sync::Mutex;

use rusqlite::Connection;
use tauri::{AppHandle, Manager};

use crate::error::{CoreError, CoreResult};
use crate::index::schema;

/// Index dosyasının app-data içindeki adı.
const INDEX_FILE: &str = "index.sqlite3";

pub struct IndexHandle {
    conn: Mutex<Connection>,
}

impl IndexHandle {
    /// Index'i açar, şemayı uygular.
    ///
    /// Anayasa madde 9.3: index her an silinebilir. Bu fonksiyon silinmiş bir
    /// index'i sıfırdan kurabilir — çağıran taraf için fark yoktur.
    pub fn open(app: &AppHandle) -> CoreResult<Self> {
        let path = Self::file_path(app)?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(CoreError::Config)?;
        }

        let conn = Connection::open(&path).map_err(CoreError::IndexOpen)?;

        // Anayasa madde 34.1: sorgu bütçesi < 10 ms.
        // WAL + normal senkron, okuma ağırlıklı yükte doğru ayar.
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

    /// Index'lenmiş not sayısı. Anayasa madde 11.5: SAYMAK analiz değildir.
    pub fn note_count(&self) -> CoreResult<u64> {
        let conn = self.conn.lock().expect("index mutex zehirlendi");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM notes", [], |row| row.get(0))
            .map_err(CoreError::IndexQuery)?;
        Ok(count as u64)
    }

    /// Şema sürümü — arayüzün index hazırlığını görmesi için.
    pub fn schema_version(&self) -> CoreResult<i64> {
        let conn = self.conn.lock().expect("index mutex zehirlendi");
        conn.pragma_query_value(None, "user_version", |row| row.get(0))
            .map_err(CoreError::IndexQuery)
    }
}
