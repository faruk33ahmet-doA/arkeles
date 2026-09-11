/*!
Çekirdek hata tipleri.

Anayasa madde 19.5: "Vault içeriği hiçbir log satırına yazılmaz.
Log'da yalnızca ID ve dosya yolu bulunur."

Bu yüzden hata varyantları NOT İÇERİĞİ TAŞIMAZ. Sağlık ve Finans verisi
hassastır; bir parse hatasının mesajı o satırın metnini asla göstermez.
*/

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    #[error("yapılandırma okunamadı")]
    Config(#[source] std::io::Error),

    #[error("index açılamadı")]
    IndexOpen(#[source] rusqlite::Error),

    #[error("index sorgusu başarısız")]
    IndexQuery(#[source] rusqlite::Error),

    #[error("uygulama dizini bulunamadı")]
    AppDirUnavailable,

    #[error("vault yapılandırılmadı")]
    VaultNotConfigured,

    #[error("seçilen klasör okunamıyor")]
    VaultUnreadable,

    /*
     * Aşağıdakiler MEKANİK MUTASYON hatalarıdır (madde 8.1, 20).
     * Hepsi arayüzde ayrı davranış gerektirir, bu yüzden ayrı varyant:
     * çakışma kullanıcıya sorulur, diğerleri sakin bir durumdur.
     */

    /// Madde 20.3: dosya beklenenden farklı — SESSİZCE ÜZERİNE YAZILMAZ.
    #[error("içerik başka bir kaynak tarafından değiştirildi")]
    Conflict,

    /// Hedef satır beklenen görev değil, ya da yazılamaz bir istek.
    #[error("hedef bulunamadı veya değişmiş")]
    TargetMismatch,

    /// Madde 16.3: `arkeles_id` taşımayan not SALT OKUNUR.
    #[error("bu not yönetilmiyor, yazılamaz")]
    NoteUnmanaged,

    /// Madde 8.2: istenen şey semantik mutasyon — Hermes'in alanı.
    #[error("bu işlem Hermes'in alanında")]
    SemanticMutationRefused,

    /// Madde 8.1: frontmatter oluşturmak yapı kurmaktır, mekanik değil.
    #[error("notun frontmatter bölümü yok")]
    NoFrontmatter,

    /// Madde 10.2: ARKELÉS yeni dosya AÇMAZ.
    #[error("gelen kutusu dosyası yok")]
    InboxMissing,

    /// Madde 19: vault dışına yazma girişimi.
    #[error("yol vault dışında")]
    PathOutsideVault,

    #[error("dosya yazılamadı")]
    WriteFailed(#[source] std::io::Error),

    /*
     * HERMES hataları — Sprint 4.
     *
     * Madde 18.4: ulaşılamamak bir ARIZA DEĞİLDİR; yine de bir İŞ
     * gönderilirken oluşursa kullanıcı bilmelidir (madde 25.4).
     * Madde 19: hiçbiri stack trace TAŞIMAZ.
     */
    #[error("Hermes'e ulaşılamadı")]
    HermesUnreachable,

    #[error("Hermes kimlik doğrulaması başarısız")]
    HermesUnauthorized,

    #[error("Hermes zamanında yanıt vermedi")]
    HermesTimeout,

    /// Hermes'in KISA hata mesajı. `data` alanı (stack trace) atılmıştır.
    #[error("{0}")]
    HermesRejected(String),

    /// Hermes'in cevabı beklenen biçimde değil (JSON değil, alan eksik).
    #[error("Hermes beklenmeyen bir cevap verdi")]
    HermesMalformed,

    /// Madde 19: allowlist dışı aksiyon adı.
    #[error("bu aksiyon tanımlı değil")]
    ActionNotAllowed,
}

impl CoreError {
    /// Arayüzün dallanabilmesi için kararlı bir kod. Metin değişebilir,
    /// bu kod DEĞİŞMEZ — çakışma UI'si buna göre açılır (madde 4).
    pub fn code(&self) -> &'static str {
        match self {
            CoreError::Conflict => "conflict",
            CoreError::TargetMismatch => "target_mismatch",
            CoreError::NoteUnmanaged => "note_unmanaged",
            CoreError::SemanticMutationRefused => "semantic_refused",
            CoreError::NoFrontmatter => "no_frontmatter",
            CoreError::InboxMissing => "inbox_missing",
            CoreError::PathOutsideVault => "path_outside_vault",
            CoreError::WriteFailed(_) => "write_failed",
            CoreError::VaultNotConfigured => "vault_not_configured",
            CoreError::VaultUnreadable => "vault_unreadable",
            CoreError::Config(_) => "config",
            CoreError::IndexOpen(_) | CoreError::IndexQuery(_) => "index",
            CoreError::AppDirUnavailable => "app_dir",
            CoreError::HermesUnreachable => "hermes_unreachable",
            CoreError::HermesUnauthorized => "hermes_unauthorized",
            CoreError::HermesTimeout => "hermes_timeout",
            CoreError::HermesRejected(_) => "hermes_rejected",
            CoreError::HermesMalformed => "hermes_malformed_response",
            CoreError::ActionNotAllowed => "action_not_allowed",
        }
    }
}

pub type CoreResult<T> = Result<T, CoreError>;

/*
 * Tauri komutları hata döndürebilmek için Serialize ister.
 * Webview'e yalnızca İNSAN OKUR bir mesaj gider — kaynak zincir gitmez,
 * çünkü zincirde dosya yolu gibi ayrıntılar olabilir (madde 19.5).
 */
impl Serialize for CoreError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;
        // Arayüz `code` ile dallanır, `message` ile konuşur.
        // Kaynak zincir GİTMEZ: dosya yolu içerebilir (madde 19.5).
        let mut state = serializer.serialize_struct("CoreError", 2)?;
        state.serialize_field("code", self.code())?;
        state.serialize_field("message", &self.to_string())?;
        state.end()
    }
}
