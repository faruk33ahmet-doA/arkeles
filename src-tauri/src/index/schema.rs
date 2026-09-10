/*!
SQLite index şeması — Anayasa madde 9, 15.

KRİTİK ANAYASA KURALI (madde 9.3, 9.4):
Bu veritabanı TÜRETİLMİŞ VERİDİR. Kaynak veri Obsidian vault'udur.
Silme testi: bu dosyayı silince hiçbir BİLGİ kaybolmaz — yalnız hız kaybolur,
ve o da vault'tan yeniden taranarak geri gelir.

Bu yüzden şemada:
  - Hiçbir alan "yalnız burada yaşayan" veri tutmaz
  - Her satırın kaynağı bir vault dosyasıdır (`source_path`)
  - Yabancı anahtar zorlamaları yumuşaktır: index her an sıfırlanabilir

15.5  Şema sürümü değişince index otomatik YENİDEN KURULUR.
      Migration yazılmaz — türetilmiş veriye migration gereksiz karmaşıklıktır.

Sprint 0: şema tanımlı ve uygulanıyor. Tablolar BOŞ — parser Sprint 1.
*/

use rusqlite::Connection;

use crate::error::{CoreError, CoreResult};

/// Anayasa madde 15.5. Bu sayı artınca index sıfırlanır ve yeniden kurulur.
pub const SCHEMA_VERSION: i64 = 2;

/// Şemayı uygular. Sürüm uyuşmazsa her şeyi silip yeniden kurar.
pub fn apply(conn: &Connection) -> CoreResult<()> {
    let current: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(CoreError::IndexOpen)?;

    if current == SCHEMA_VERSION {
        return Ok(());
    }

    // Anayasa madde 15.5: migration YOK, yeniden kurulum VAR.
    // Türetilmiş veri olduğu için bu kayıpsızdır (madde 9.3).
    conn.execute_batch(DROP_ALL).map_err(CoreError::IndexOpen)?;
    conn.execute_batch(CREATE_ALL).map_err(CoreError::IndexOpen)?;

    conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(CoreError::IndexOpen)?;

    Ok(())
}

const DROP_ALL: &str = r#"
DROP TABLE IF EXISTS index_meta;
DROP TABLE IF EXISTS notes_fts;
DROP TABLE IF EXISTS tasks;
DROP TABLE IF EXISTS links;
DROP TABLE IF EXISTS notes;
"#;

const CREATE_ALL: &str = r#"
-- ---------------------------------------------------------------------------
-- notes — vault'taki her markdown dosyasının projeksiyonu.
--
-- `id`          Anayasa madde 16.1: frontmatter'daki kalıcı ULID.
--               Madde 16.3: taşımayan notlar "yönetilmeyen" (managed = 0) ve
--               id olarak dosya yolunun hash'i kullanılır — geçici kimlik.
-- `source_path` Kaynak dosya. Madde 16.2: kimlik DEĞİL, yalnız konum.
-- `content_hash` Madde 15.6: hash aynıysa yeniden indeksleme yapılmaz.
-- ---------------------------------------------------------------------------
CREATE TABLE notes (
    id            TEXT PRIMARY KEY,
    source_path   TEXT NOT NULL UNIQUE,
    title         TEXT NOT NULL,
    managed       INTEGER NOT NULL DEFAULT 0,
    content_hash  TEXT NOT NULL,
    modified_at   TEXT NOT NULL,
    created_at    TEXT,
    frontmatter   TEXT
) STRICT;

CREATE INDEX idx_notes_modified ON notes(modified_at DESC);
CREATE INDEX idx_notes_managed  ON notes(managed);

-- ---------------------------------------------------------------------------
-- tasks — notların içindeki `- [ ]` satırları.
--
-- `line_number` mekanik mutasyonun (madde 8.1) hedefini bulmak için.
--               Yazma sırasında ayrıca içerik hash'i doğrulanır (madde 20.2).
-- `status`      "open" | "done". Madde 8.3: durum değişimi mekanik.
-- ---------------------------------------------------------------------------
CREATE TABLE tasks (
    id           TEXT PRIMARY KEY,
    note_id      TEXT NOT NULL,
    line_number  INTEGER NOT NULL,
    title        TEXT NOT NULL,
    status       TEXT NOT NULL,
    workspace    TEXT,
    due          TEXT,
    FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE
) STRICT;

CREATE INDEX idx_tasks_due    ON tasks(due) WHERE due IS NOT NULL;
CREATE INDEX idx_tasks_status ON tasks(status);
CREATE INDEX idx_tasks_note   ON tasks(note_id);

-- ---------------------------------------------------------------------------
-- links — notlar arası [[wiki]] bağlantıları.
-- Hedef henüz index'te olmayabilir; bu yüzden `target_id` yabancı anahtar DEĞİL.
-- ---------------------------------------------------------------------------
CREATE TABLE links (
    source_id  TEXT NOT NULL,
    target_ref TEXT NOT NULL,
    PRIMARY KEY (source_id, target_ref),
    FOREIGN KEY (source_id) REFERENCES notes(id) ON DELETE CASCADE
) STRICT;

-- ---------------------------------------------------------------------------
-- notes_fts — tam metin arama (madde 27.4: Cmd+K içinden not arama).
-- `content` burada tutulur ama bu bir KOPYADIR (madde 9.4), kaynak değil.
-- ---------------------------------------------------------------------------
CREATE VIRTUAL TABLE notes_fts USING fts5(
    note_id UNINDEXED,
    title,
    body,
    tokenize = 'unicode61 remove_diacritics 2'
);

-- ---------------------------------------------------------------------------
-- index_meta — index'in kendi durumu. Anayasa madde 9.4: türetilmiş veri.
-- Buradaki hiçbir satır BİLGİ taşımaz, yalnız taramanın ne zaman yapıldığını
-- söyler. Silinse vault'tan yeniden kurulur.
-- ---------------------------------------------------------------------------
CREATE TABLE index_meta (
    key   TEXT PRIMARY KEY,
    value TEXT NOT NULL
) STRICT;
"#;
