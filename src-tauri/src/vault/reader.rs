/*!
Vault okuma — Anayasa madde 15.1, 15.2.

15.1  Markdown dosyaları ARAYÜZ TARAFINDAN doğrudan taranmaz.
15.2  Bütün okuma index üzerinden yapılır.

Yani bu modülün müşterisi arayüz DEĞİL, index oluşturucudur. Arayüz
`vault_status` gibi özetler alır; not içeriğine index'ten erişir.

SPRINT 0 KAPSAMI: yalnız vault dizininin varlık/erişilebilirlik kontrolü.
PARSER YAZILMADI — Sprint 0 teslim kriteri bunu açıkça dışarıda bırakıyor.

Sprint 1'de buraya eklenecek:
  - frontmatter (YAML) ayrıştırma
  - gövde ve `- [ ]` görev satırı çıkarma
  - [[wiki]] bağlantı çıkarma
  - senkronizasyon artefaktı filtresi (madde 20.5)
  - test vakaları: bozuk YAML, eksik frontmatter, boş dosya, BOM, CRLF,
    emoji, 5 MB not (madde 39.3)
*/

use std::path::Path;

/// Vault dizini okunabilir durumda mı?
///
/// Anayasa madde 18.3 mantığı burada da geçerli: erişilemez olmak bir
/// çökme sebebi değildir, arayüzde sakin bir durumdur.
pub fn is_readable(path: &Path) -> bool {
    path.is_dir() && std::fs::read_dir(path).is_ok()
}

/// Anayasa madde 20.5: senkronizasyon artefaktları yok sayılır.
///
/// iCloud/Dropbox, vault içinde ARKELÉS'in veri sanacağı çöp dosyalar
/// üretir. Bu filtre Sprint 1'de tarayıcı tarafından kullanılacak;
/// mantığı ve testleri şimdi yazıldı çünkü kuralın kendisi kesin.
pub fn is_sync_artifact(file_name: &str) -> bool {
    file_name.ends_with(".icloud")
        || file_name.starts_with('.')
        || file_name.starts_with("~$")
        || file_name.contains("conflicted copy")
        || file_name.contains("(Çakışan")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn icloud_placeholder_filtrelenir() {
        assert!(is_sync_artifact(".not.md.icloud"));
        assert!(is_sync_artifact("plan.md.icloud"));
    }

    #[test]
    fn gizli_dosyalar_filtrelenir() {
        assert!(is_sync_artifact(".DS_Store"));
        assert!(is_sync_artifact(".obsidian"));
    }

    #[test]
    fn office_gecici_dosyalari_filtrelenir() {
        assert!(is_sync_artifact("~$rapor.docx"));
    }

    #[test]
    fn dropbox_cakisma_kopyalari_filtrelenir() {
        assert!(is_sync_artifact("plan (Ali's conflicted copy 2026-09-11).md"));
    }

    #[test]
    fn normal_notlar_gecer() {
        assert!(!is_sync_artifact("plan.md"));
        assert!(!is_sync_artifact("WIF — Haftalık Ekip Notu.md"));
        assert!(!is_sync_artifact("2026-09-11.md"));
    }
}
