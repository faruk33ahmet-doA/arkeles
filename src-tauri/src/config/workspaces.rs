/*!
Çalışma alanı kaydı — Anayasa madde 36.3.

BU DOSYA KOD DEĞİL, VERİDİR. WIF ve GEN için özel mantık YOKTUR; her kurum
aynı alanları doldurur ve aynı yolla çözümlenir. Yeni bir kurum eklemek
buraya bir satır eklemektir.

`aliases`: frontmatter'da yazılabilecek varyantlar. Kullanıcı `workspace: WIF`
da yazabilir `workspace: wif` de — ikisi de aynı kuruma düşer. Bu, madde
7.2'nin sonucu: Obsidian tek gerçek, ve kullanıcı orada nasıl yazdıysa öyle
yazmıştır; normalleştirmeyi ARKELÉS yapar, kullanıcıya biçim dayatmaz.
*/

/// Bir çalışma alanının tanımı.
#[derive(Debug, Clone)]
pub struct WorkspaceDef {
    /// Kanonik kimlik. Index'te ve IPC'de bu kullanılır.
    pub id: &'static str,
    /// Arayüzde görünen ad.
    pub label: &'static str,
    /// Frontmatter'da eşleşecek varyantlar (küçük harfe indirilmiş).
    pub aliases: &'static [&'static str],
    /// Madde 36.2/36.4: gerçek veri yüzeyi geliştirildi mi?
    pub active: bool,
}

/*
 * Anayasa madde 36.3 sırası. Sprint 5: YEDİSİ DE AKTİF.
 *
 * Hepsi aynı şablonla, aynı yolla çalışır — kuruma özel bileşen YOK.
 * Vault'ta verisi olmayan bir kurum boş durum doktrinine düşer (madde 26:
 * kırık değil, HENÜZ boş). `active` alanı veri olarak kalır: yeni bir kurum
 * eklenirken yüzeyi hazır değilse `false` ile görünür-ama-boş başlar.
 */
pub const WORKSPACES: &[WorkspaceDef] = &[
    WorkspaceDef {
        id: "wif",
        label: "WIF",
        aliases: &["wif"],
        active: true,
    },
    WorkspaceDef {
        id: "gen",
        label: "GEN",
        aliases: &["gen"],
        active: true,
    },
    WorkspaceDef {
        id: "tuga",
        label: "TüGA",
        aliases: &["tuga", "tüga"],
        active: true,
    },
    WorkspaceDef {
        id: "burkon",
        label: "Burkon",
        aliases: &["burkon"],
        active: true,
    },
    WorkspaceDef {
        id: "merci",
        label: "Merci",
        aliases: &["merci"],
        active: true,
    },
    WorkspaceDef {
        id: "kepder",
        label: "KEPDER",
        aliases: &["kepder"],
        active: true,
    },
    WorkspaceDef {
        id: "ogrenciyiz",
        label: "Öğrenciyiz.biz.tr",
        aliases: &["ogrenciyiz", "öğrenciyiz", "ogrenciyiz.biz.tr", "öğrenciyiz.biz.tr"],
        active: true,
    },
];

/// Ham frontmatter değerini kanonik kimliğe çevirir.
///
/// Eşleşme yoksa `None` — ve bu ÖNEMLİ: eşleşmeyen bir değer YANLIŞ bir
/// kuruma DÜŞMEZ. Tanınmayan çalışma alanı, çalışma alanı yokmuş gibi
/// davranır (madde 6.3: şüphede varsayım yapmayız).
pub fn resolve(raw: &str) -> Option<&'static str> {
    let needle = normalize(raw);
    if needle.is_empty() {
        return None;
    }

    WORKSPACES
        .iter()
        .find(|ws| ws.id == needle || ws.aliases.contains(&needle.as_str()))
        .map(|ws| ws.id)
}

/*
 * Kurum adını eşleştirme için normalleştirir.
 *
 * DİKKAT — BURADA TÜRKÇE 'I' → 'ı' KURALI UYGULANMAZ.
 *
 * Türkçe küçültme kuralı 'I'yı 'ı'ya çevirir ve bu, Türkçe KELİMELER için
 * doğrudur. Ama kurum adları kısaltmadır ve ASCII 'I' ile yazılır:
 * "WIF" → "wıf" olurdu ve hiçbir şeye eşleşmezdi. (Bu hata bir kez yapıldı
 * ve WIF'in bütün verisi görünmez oldu; test onu tutuyor.)
 *
 * Yalnız 'İ' elle eşlenir: Unicode varsayılanı onu "i + birleşen nokta"
 * yapar ve o da eşleşmeyi bozar.
 *
 * Türkçe küçültme gerektiren bir kurum adı çıkarsa çözüm `aliases`'a
 * açık bir varyant eklemektir — tahmin etmek değil.
 */
fn normalize(raw: &str) -> String {
    raw.trim()
        .chars()
        .flat_map(|c| match c {
            'İ' => vec!['i'],
            other => other.to_lowercase().collect::<Vec<char>>(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn yedi_calisma_alani_tanimli() {
        // Anayasa madde 36.3.
        assert_eq!(WORKSPACES.len(), 7);
    }

    #[test]
    fn sprint_5_yedi_alan_da_aktif() {
        let active: Vec<&str> = WORKSPACES.iter().filter(|w| w.active).map(|w| w.id).collect();
        assert_eq!(
            active,
            vec!["wif", "gen", "tuga", "burkon", "merci", "kepder", "ogrenciyiz"]
        );
    }

    #[test]
    fn kimlikler_tekil() {
        let mut ids: Vec<&str> = WORKSPACES.iter().map(|w| w.id).collect();
        ids.sort();
        let count = ids.len();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    /*
     * GERİLEME TESTİ: "WIF" içindeki ASCII 'I' harfi.
     *
     * Türkçe küçültme kuralı ('I' → 'ı') uygulanırsa "wıf" çıkar ve WIF'in
     * BÜTÜN verisi görünmez olur. Bu bir kez gerçekleşti.
     */
    #[test]
    #[allow(non_snake_case, reason = "büyük ASCII 'I' testin konusu")]
    fn ascii_I_harfi_turkce_kuralla_bozulmaz() {
        assert_eq!(resolve("WIF"), Some("wif"), "ASCII 'I' → 'i' olmalı, 'ı' değil");
        assert_eq!(normalize("WIF"), "wif");
        assert_eq!(normalize("KEPDER"), "kepder");
        assert_eq!(resolve("KEPDER"), Some("kepder"));
    }

    #[test]
    #[allow(non_snake_case, reason = "büyük ASCII 'I' testin konusu")]
    fn buyuk_I_noktali_i_ye_donusur() {
        // Unicode varsayılanı 'İ'yi "i + birleşen nokta" yapar; eşleşmeyi bozar.
        assert_eq!(normalize("İ"), "i");
        assert_eq!(normalize("İSTANBUL").chars().count(), 8);
    }

    #[test]
    fn buyuk_kucuk_harf_farki_eleniyor() {
        assert_eq!(resolve("WIF"), Some("wif"));
        assert_eq!(resolve("wif"), Some("wif"));
        assert_eq!(resolve("  Wif  "), Some("wif"));
        assert_eq!(resolve("GEN"), Some("gen"));
    }

    #[test]
    fn turkce_karakterli_kurumlar() {
        assert_eq!(resolve("TüGA"), Some("tuga"));
        assert_eq!(resolve("tuga"), Some("tuga"));
        assert_eq!(resolve("Öğrenciyiz.biz.tr"), Some("ogrenciyiz"));
        assert_eq!(resolve("ogrenciyiz"), Some("ogrenciyiz"));
    }

    #[test]
    fn taninmayan_deger_yanlis_kuruma_dusmez() {
        // Sprint 3 test gereksinimi: workspace alanı olmayan/tanınmayan not
        // YANLIŞ kuruma düşmemeli.
        assert_eq!(resolve("bilinmeyen"), None);
        assert_eq!(resolve(""), None);
        assert_eq!(resolve("   "), None);
        assert_eq!(resolve("wi"), None);
        assert_eq!(resolve("wiff"), None);
        // "WIF Projesi" bir kurum adı DEĞİL — tam eşleşme gerekir.
        assert_eq!(resolve("WIF Projesi"), None);
    }

}
