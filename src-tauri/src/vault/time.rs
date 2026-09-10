/*!
Zaman biçimlendirme — Anayasa madde 14.1.

`chrono` eklemiyoruz. İhtiyacımız tek: unix saniyesini ISO 8601 UTC'ye
çevirmek ve "bugün"ü bulmak. Bunun için tam bir takvim kütüphanesi
gereksiz bağımlılıktır.

Takvim yargısı (hangi görev "geciken"dir vb.) burada YAPILMAZ — o karar
index sorgusunda tarih DİZGE karşılaştırmasıyla verilir; ISO 8601
sözlüksel sıralaması kronolojik sıralamayla aynıdır.
*/

/// Unix saniyesi → `YYYY-MM-DDTHH:MM:SSZ`
pub fn unix_to_iso(secs: u64) -> String {
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    let rem = secs % 86_400;
    format!(
        "{y:04}-{m:02}-{d:02}T{:02}:{:02}:{:02}Z",
        rem / 3600,
        (rem % 3600) / 60,
        rem % 60
    )
}

/// Bugünün tarihi, `YYYY-MM-DD` (UTC).
pub fn today_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (y, m, d) = civil_from_days((secs / 86_400) as i64);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Şu anın ISO 8601 UTC damgası.
pub fn now_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    unix_to_iso(secs)
}

/// Howard Hinnant'ın `civil_from_days` algoritması — proleptik Gregoryen.
/// Sıçrama yıllarını doğru işler, tablo gerektirmez.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unix_sifir_epoch() {
        assert_eq!(unix_to_iso(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn bilinen_damga() {
        // 2026-09-11T00:00:00Z = 1789084800
        assert_eq!(unix_to_iso(1_789_084_800), "2026-09-11T00:00:00Z");
    }

    #[test]
    fn sicrama_gunu_dogru() {
        // 2024-02-29T00:00:00Z = 1709164800
        assert_eq!(unix_to_iso(1_709_164_800), "2024-02-29T00:00:00Z");
    }

    #[test]
    fn saat_dakika_saniye() {
        assert_eq!(unix_to_iso(1_789_084_800 + 3661), "2026-09-11T01:01:01Z");
    }

    #[test]
    fn iso_sozluksel_sirasi_kronolojiktir() {
        // Bu, "geciken görev" sorgusunun dizge karşılaştırmasıyla
        // çalışabilmesinin dayanağı.
        let a = unix_to_iso(1_709_164_800);
        let b = unix_to_iso(1_789_084_800);
        assert!(a < b);
    }

    #[test]
    fn today_iso_bicimi_dogru() {
        let t = today_iso();
        assert_eq!(t.len(), 10);
        assert_eq!(t.as_bytes()[4], b'-');
        assert_eq!(t.as_bytes()[7], b'-');
    }
}
