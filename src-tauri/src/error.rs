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
}

pub type CoreResult<T> = Result<T, CoreError>;

/*
 * Tauri komutları hata döndürebilmek için Serialize ister.
 * Webview'e yalnızca İNSAN OKUR bir mesaj gider — kaynak zincir gitmez,
 * çünkü zincirde dosya yolu gibi ayrıntılar olabilir (madde 19.5).
 */
impl Serialize for CoreError {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}
