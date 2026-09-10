/*!
Görev ayrıştırma — `- [ ]` ve `- [x]`.

Anayasa madde 8.3: görev durumu değişimi MEKANİK mutasyondur (ARKELÉS yazar).
Bu yüzden her göreve satır numarası eklenir — Sprint 2'de yazma hedefini
bulmak için (yazma anında ayrıca içerik hash'i doğrulanır, madde 20.2).

Desteklenen biçimler (Obsidian gerçeği):
  - [ ] açık            * [ ] açık            + [ ] açık
  - [x] bitti           - [X] bitti
  girintili alt görevler de sayılır
Vade:
  📅 YYYY-MM-DD         (Obsidian Tasks eklentisi)
  due:: YYYY-MM-DD      (Dataview satır içi alan)
Hiçbiri yoksa çağıran taraf notun frontmatter'ındaki `due`ya düşer.
*/

use crate::vault::note::{ParsedTask, TaskStatus};

/// Gövdeyi tarar. `line_offset`, gövdenin dosya içindeki başlangıç satırıdır
/// (frontmatter satırları + açılış/kapanış çitleri) — böylece satır numarası
/// DOSYAYA göre doğru olur, gövdeye göre değil.
pub fn parse(body: &str, line_offset: u32) -> Vec<ParsedTask> {
    let mut tasks = Vec::new();
    let mut in_code_fence = false;

    for (i, raw_line) in body.lines().enumerate() {
        let line = raw_line.trim_end_matches('\r');
        let trimmed = line.trim_start();

        // Kod bloğu içindeki `- [ ]` görev DEĞİLDİR — örnek koddur.
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        let Some((status, rest)) = match_checkbox(trimmed) else {
            continue;
        };

        let title = clean_title(rest);
        if title.is_empty() {
            continue;
        }

        tasks.push(ParsedTask {
            line_number: line_offset + i as u32 + 1,
            title,
            status,
            due: extract_due(rest),
        });
    }

    tasks
}

/// `- [ ] ...` başlangıcını tanır, (durum, kalan metin) döner.
fn match_checkbox(trimmed: &str) -> Option<(TaskStatus, &str)> {
    let after_bullet = trimmed
        .strip_prefix("- ")
        .or_else(|| trimmed.strip_prefix("* "))
        .or_else(|| trimmed.strip_prefix("+ "))?;

    let after_bullet = after_bullet.trim_start();

    if let Some(rest) = after_bullet.strip_prefix("[ ]") {
        return Some((TaskStatus::Open, rest));
    }
    if let Some(rest) = after_bullet.strip_prefix("[x]") {
        return Some((TaskStatus::Done, rest));
    }
    if let Some(rest) = after_bullet.strip_prefix("[X]") {
        return Some((TaskStatus::Done, rest));
    }
    None
}

/// Vade işaretlerini ve fazla boşluğu başlıktan temizler.
///
/// `pub` çünkü `writer::rewrite_checkbox` hedef doğrulaması yaparken AYNI
/// temizliği uygulamak zorunda. İki kopya olsa biri değişince diğeri
/// sessizce bozulur ve yanlış satıra yazardık.
pub fn title_from_rest(rest: &str) -> String {
    clean_title(rest)
}

fn clean_title(rest: &str) -> String {
    let mut title = rest.trim().to_string();

    // 📅 YYYY-MM-DD
    if let Some(pos) = title.find('\u{1f4c5}') {
        title.truncate(pos);
    }
    // due:: YYYY-MM-DD
    if let Some(pos) = title.find("due::") {
        title.truncate(pos);
    }

    title.trim().to_string()
}

fn extract_due(rest: &str) -> Option<String> {
    if let Some(pos) = rest.find('\u{1f4c5}') {
        if let Some(date) = first_iso_date(&rest[pos..]) {
            return Some(date);
        }
    }
    if let Some(pos) = rest.find("due::") {
        if let Some(date) = first_iso_date(&rest[pos + 5..]) {
            return Some(date);
        }
    }
    None
}

/// Metinde ilk `YYYY-MM-DD` desenini bulur. Takvim geçerliliği DOĞRULANMAZ —
/// bu bir tarih kütüphanesi değil, ayrıştırıcıdır; anlam Hermes'in işi (madde 8.2).
fn first_iso_date(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    for start in 0..bytes.len().saturating_sub(9) {
        let window = &bytes[start..start + 10];
        let shaped = window[..4].iter().all(u8::is_ascii_digit)
            && window[4] == b'-'
            && window[5..7].iter().all(u8::is_ascii_digit)
            && window[7] == b'-'
            && window[8..10].iter().all(u8::is_ascii_digit);
        if shaped {
            // Rakamlar ASCII olduğu için bu dilim her zaman geçerli UTF-8.
            return std::str::from_utf8(window).ok().map(str::to_string);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn titles(body: &str) -> Vec<String> {
        parse(body, 0).into_iter().map(|t| t.title).collect()
    }

    #[test]
    fn acik_ve_bitmis_gorev() {
        let tasks = parse("- [ ] açık\n- [x] bitti\n", 0);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].status, TaskStatus::Open);
        assert_eq!(tasks[1].status, TaskStatus::Done);
    }

    #[test]
    fn buyuk_x_de_bitmis_sayilir() {
        let tasks = parse("- [X] bitti\n", 0);
        assert_eq!(tasks[0].status, TaskStatus::Done);
    }

    #[test]
    fn yildiz_ve_arti_madde_isaretleri() {
        assert_eq!(titles("* [ ] a\n+ [ ] b\n"), vec!["a", "b"]);
    }

    #[test]
    fn girintili_alt_gorevler_sayilir() {
        assert_eq!(titles("- [ ] üst\n    - [ ] alt\n"), vec!["üst", "alt"]);
    }

    #[test]
    fn satir_numarasi_dosyaya_gore_1_tabanli() {
        // 4 satırlık frontmatter varsayımı: offset 4.
        let tasks = parse("boş\n- [ ] görev\n", 4);
        assert_eq!(tasks[0].line_number, 6);
    }

    #[test]
    fn kod_blogundaki_checkbox_gorev_degildir() {
        let body = "- [ ] gerçek\n```md\n- [ ] örnek\n```\n- [ ] gerçek2\n";
        assert_eq!(titles(body), vec!["gerçek", "gerçek2"]);
    }

    #[test]
    fn obsidian_tasks_vade_isareti() {
        let tasks = parse("- [ ] rapor 📅 2026-09-11\n", 0);
        assert_eq!(tasks[0].due.as_deref(), Some("2026-09-11"));
        assert_eq!(tasks[0].title, "rapor");
    }

    #[test]
    fn dataview_vade_alani() {
        let tasks = parse("- [ ] rapor due:: 2026-10-02\n", 0);
        assert_eq!(tasks[0].due.as_deref(), Some("2026-10-02"));
        assert_eq!(tasks[0].title, "rapor");
    }

    #[test]
    fn vade_yoksa_none() {
        let tasks = parse("- [ ] rapor\n", 0);
        assert!(tasks[0].due.is_none());
    }

    #[test]
    fn bos_baslikli_checkbox_atlanir() {
        assert!(parse("- [ ]\n- [ ]   \n", 0).is_empty());
    }

    #[test]
    fn checkbox_olmayan_madde_atlanir() {
        assert!(parse("- normal madde\n* başka\n", 0).is_empty());
    }

    #[test]
    fn emoji_ve_turkce_baslik_bozulmaz() {
        let tasks = parse("- [ ] Öğrenciyiz için içerik 🌱 hazırla\n", 0);
        assert_eq!(tasks[0].title, "Öğrenciyiz için içerik 🌱 hazırla");
    }

    #[test]
    fn crlf_satirlar() {
        let tasks = parse("- [ ] a\r\n- [x] b\r\n", 0);
        assert_eq!(tasks.len(), 2);
        assert_eq!(tasks[0].title, "a");
    }

    #[test]
    fn bos_govde_panik_yapmaz() {
        assert!(parse("", 0).is_empty());
    }
}
