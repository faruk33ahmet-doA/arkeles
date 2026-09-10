/*!
Ayrıştırılmış not modeli — Anayasa madde 16.

Bu tipler index'e yazılmak üzere üretilir; webview'e HİÇ çıkmaz.
Webview'e çıkan tipler `types/` altında ve `ts-rs` ile üretilir (madde 14.2).
*/

use serde_json::{Map, Value};

/// Görev durumu — Anayasa madde 8.3: durum değişimi mekanik mutasyondur.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    Done,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::Done => "done",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParsedTask {
    /// 1 tabanlı satır numarası. Mekanik mutasyonun hedefini bulmak için
    /// (madde 8.1); yazma sırasında ayrıca içerik hash'i doğrulanır (madde 20.2).
    pub line_number: u32,
    pub title: String,
    pub status: TaskStatus,
    /// ISO 8601 tarih (YYYY-MM-DD). Yoksa None.
    pub due: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ParsedNote {
    /// Anayasa madde 16.1: frontmatter'daki kalıcı ULID.
    /// Yoksa madde 16.3 uyarınca dosya yolundan türetilmiş geçici kimlik.
    pub id: String,
    /// Madde 16.3: `arkeles_id` taşıyor mu? Taşımıyorsa mekanik mutasyon almaz.
    pub managed: bool,
    pub title: String,
    /// Madde 16.2: kimlik DEĞİL, yalnız konum. Vault köküne göre göreli.
    pub source_path: String,
    pub frontmatter: Map<String, Value>,
    pub tasks: Vec<ParsedTask>,
    /// [[wiki]] hedefleri, ham metin olarak.
    pub links: Vec<String>,
    /// FTS için gövde. Madde 9.4: bu bir KOPYADIR, kaynak değil.
    pub body: String,
    /// Madde 15.6: hash aynıysa yeniden indeksleme yapılmaz.
    pub content_hash: String,
    /// ISO 8601 UTC.
    pub modified_at: String,
    /// Frontmatter'dan gelen çalışma alanı (madde 36.3). Yoksa üst klasör adı.
    pub workspace: Option<String>,
}
