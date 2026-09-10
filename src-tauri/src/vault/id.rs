/*!
Not kimliği — Anayasa madde 16.

16.1  Yönetilen her not frontmatter'da kalıcı bir `arkeles_id` (ULID) taşır.
16.2  Dosya adı İNSAN İÇİNDİR, kimlik değildir.
16.3  `arkeles_id` taşımayan notlar "yönetilmeyen"dir: okunur, aranır,
      gösterilir — ama mekanik mutasyon uygulanmaz.
16.4  Bir nota `arkeles_id` YAZMAK semantik bir işlemdir → HERMES yapar.
      Bu yüzden bu modülde ULID ÜRETİMİ YOKTUR, yalnız OKUMA/DOĞRULAMA vardır.

Sprint 0: doğrulama fonksiyonu ve geçici kimlik türetimi. Frontmatter'dan
          okuma Sprint 1 (parser ile birlikte).
*/

/// Frontmatter'da aranan anahtar. Anayasa madde 16.1.
pub const ID_KEY: &str = "arkeles_id";

/// ULID biçim doğrulaması: 26 karakter, Crockford Base32.
///
/// Anayasa madde 16.4 gereği ARKELÉS ULID ÜRETMEZ — yalnız Hermes'in
/// yazdığını tanır. Bu fonksiyon o tanımayı yapar.
pub fn is_valid_ulid(candidate: &str) -> bool {
    const ULID_LEN: usize = 26;
    if candidate.len() != ULID_LEN {
        return false;
    }
    // Crockford Base32: I, L, O, U harfleri kullanılmaz.
    candidate
        .bytes()
        .all(|b| matches!(b, b'0'..=b'9' | b'A'..=b'H' | b'J' | b'K' | b'M' | b'N' | b'P'..=b'T' | b'V'..=b'Z'))
}

/// Yönetilmeyen notlar için geçici, kararlı kimlik.
///
/// Anayasa madde 16.3: bu notlar gösterilir ama mekanik mutasyon almaz.
/// Kimlik dosya yolundan türetilir; dosya taşınırsa değişir — bu KABUL
/// EDİLEBİLİR, çünkü zaten kalıcı kimliği yok. Prefix ile açıkça işaretlenir
/// ki gerçek ULID ile karıştırılmasın.
pub fn transient_id(source_path: &str) -> String {
    format!("path:{}", fnv1a64(source_path.as_bytes()))
}

/// FNV-1a 64-bit. Kriptografik değil — yalnız kimlik türetimi için.
/// Anayasa madde 14.1: bunun için bağımlılık eklemeye değmez.
fn fnv1a64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x0000_0100_0000_01b3;

    let mut hash = OFFSET;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gecerli_ulid_kabul_edilir() {
        assert!(is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAV"));
        assert!(is_valid_ulid("01JQ8XKZ00000000000000000A"));
    }

    #[test]
    fn yanlis_uzunluk_reddedilir() {
        assert!(!is_valid_ulid(""));
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FA")); // 25
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAVX")); // 27
    }

    #[test]
    fn crockford_disi_harf_reddedilir() {
        // I, L, O, U Crockford Base32'de yok.
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAI"));
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAL"));
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAO"));
        assert!(!is_valid_ulid("01ARZ3NDEKTSV4RRFFQ69G5FAU"));
    }

    #[test]
    fn gecici_kimlik_kararlidir() {
        let a = transient_id("Notlar/okuma listesi.md");
        let b = transient_id("Notlar/okuma listesi.md");
        assert_eq!(a, b);
    }

    #[test]
    fn gecici_kimlik_ulid_ile_karismaz() {
        let id = transient_id("a.md");
        assert!(id.starts_with("path:"));
        assert!(!is_valid_ulid(&id));
    }
}
