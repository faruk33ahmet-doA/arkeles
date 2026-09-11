/*!
Ayrıştırılmış not modeli — Anayasa madde 16.

Bu tipler index'e yazılmak üzere üretilir; webview'e HİÇ çıkmaz.
Webview'e çıkan tipler `types/` altında ve `ts-rs` ile üretilir (madde 14.2).
*/

use serde_json::{Map, Value};

/*
 * Görev durumu — Anayasa madde 8.3.
 *
 * Obsidian'ın (ve Tasks eklentisinin) GERÇEK işaretleri:
 *   [ ]  açık          [x] [X]  bitmiş
 *   [/]  sürüyor       [>] [<]  ertelenmiş / bekliyor
 *   [-]  iptal
 *
 * Bu liste UYDURULMADI — kullanıcının vault'unda halihazırda bu işaretler
 * bulunabilir ve onları "açık" saymak veriyi YANLIŞ göstermek olurdu.
 * Tanınmayan bir işaret görev SAYILMAZ (parser onu atlar); böylece
 * `[!]` gibi kişisel işaretler yanlış gruba düşmez.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskStatus {
    Open,
    InProgress,
    Deferred,
    Done,
    Cancelled,
}

impl TaskStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            TaskStatus::Open => "open",
            TaskStatus::InProgress => "in_progress",
            TaskStatus::Deferred => "deferred",
            TaskStatus::Done => "done",
            TaskStatus::Cancelled => "cancelled",
        }
    }

    /// Markdown işaretinden durum. Tanınmayan işaret `None`.
    pub fn from_marker(marker: char) -> Option<Self> {
        match marker {
            ' ' => Some(TaskStatus::Open),
            'x' | 'X' => Some(TaskStatus::Done),
            '/' => Some(TaskStatus::InProgress),
            '>' | '<' => Some(TaskStatus::Deferred),
            '-' => Some(TaskStatus::Cancelled),
            _ => None,
        }
    }

    /*
     * Hâlâ iş bekleyen durumlar — SQL sorgularının tek kaynağı.
     *
     * Bu liste iki yerde kullanılıyordu (kurum sayımı ve genel bakış) ve
     * elle kopyalanmıştı; biri değişince diğeri sessizce yanlış sayardı.
     */
    pub const OUTSTANDING_SQL: &'static str = "('open','in_progress','deferred')";
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

/*
 * Not türü — Anayasa madde 8.2 sınırı içinde.
 *
 * Tür, notun KENDİ frontmatter'ından okunur (`arkeles_type`). ARKELÉS bir
 * notun ne olduğunu ÇIKARSAMAZ — başlığına, klasörüne veya içeriğine bakıp
 * "bu bir proje" demek bir YARGIDIR ve madde 8.2 uyarınca Hermes'in işidir.
 * Frontmatter söylemiyorsa not, nottur.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteKind {
    Note,
    Project,
    Meeting,
    Document,
    /*
     * Hermes'in yazdığı MAKİNE KAYITLARI: hayat skoru, öncelik, rapor
     * çıktısı gibi. Bunlar teknik olarak birer nottur ama kullanıcının
     * "Notlar" listesinde yerleri YOKTUR — orada gürültü yaparlar.
     *
     * ARKELÉS onları OKUR (panelde gösterilen öncelik buradan gelir) ama
     * insan yüzeylerinde LİSTELEMEZ.
     */
    System,
}

impl NoteKind {
    pub fn as_str(self) -> &'static str {
        match self {
            NoteKind::Note => "note",
            NoteKind::Project => "project",
            NoteKind::Meeting => "meeting",
            NoteKind::Document => "document",
            NoteKind::System => "system",
        }
    }

    /// `arkeles_type` değerinden tür. Tanınmayan değer düz "note" olur.
    pub fn from_frontmatter(raw: Option<&str>) -> Self {
        match raw.map(str::trim) {
            Some("project") | Some("proje") => NoteKind::Project,
            Some("meeting") | Some("toplanti") | Some("toplantı") => NoteKind::Meeting,
            Some("document") | Some("belge") => NoteKind::Document,
            // Hermes'in makine kayıtları — insan listelerinde görünmez.
            Some("priority") | Some("life_score") | Some("report") => NoteKind::System,
            _ => NoteKind::Note,
        }
    }
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
    /// Kanonik çalışma alanı kimliği (madde 36.3). Çözümlenemezse None.
    pub workspace: Option<String>,
    /// Notun türü — frontmatter'dan OKUNUR, çıkarsanmaz.
    pub kind: NoteKind,
    /// Toplantı tarihi (`date` frontmatter'ı), yalnız `kind == Meeting` için anlamlı.
    pub event_date: Option<String>,
}
