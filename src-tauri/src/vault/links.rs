/*!
Wiki bağlantı ayrıştırma — `[[hedef]]`.

Anayasa madde 7.2: Obsidian tek doğruluk kaynağı. Bağlantılar Obsidian'ın
kendi sözdizimidir; ARKELÉS onları YORUMLAMAZ, yalnız çıkarır ve index'ler.
Hedefin var olup olmadığı BURADA sorgulanmaz (index'te `links.target_ref`
yabancı anahtar değildir) çünkü hedef henüz taranmamış olabilir.

Desteklenen biçimler:
  [[Hedef]]
  [[Hedef|görünen ad]]      → "Hedef"
  [[Hedef#başlık]]          → "Hedef"
  [[Hedef#başlık|ad]]       → "Hedef"
  ![[Hedef]]                → gömme de bir ilişkidir, sayılır
*/

/// Gövdeden bağlantı hedeflerini çıkarır. Tekrarlar temizlenir, sıra korunur.
pub fn parse(body: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut in_code_fence = false;

    for line in body.lines() {
        let trimmed = line.trim_start();

        // Kod bloğu içindeki [[...]] bağlantı değildir — örnek metindir.
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_code_fence = !in_code_fence;
            continue;
        }
        if in_code_fence {
            continue;
        }

        collect_in_line(line, &mut out);
    }

    out
}

fn collect_in_line(line: &str, out: &mut Vec<String>) {
    let mut rest = line;
    while let Some(open) = rest.find("[[") {
        let after = &rest[open + 2..];
        let Some(close) = after.find("]]") else {
            return; // kapanmayan bağlantı — yok sayılır
        };
        let inner = &after[..close];
        if let Some(target) = normalize(inner) {
            if !out.iter().any(|existing| existing == &target) {
                out.push(target);
            }
        }
        rest = &after[close + 2..];
    }
}

/// `Hedef#başlık|ad` → `Hedef`
fn normalize(inner: &str) -> Option<String> {
    let before_alias = inner.split('|').next().unwrap_or(inner);
    let before_anchor = before_alias.split('#').next().unwrap_or(before_alias);
    let target = before_anchor.trim();
    if target.is_empty() {
        return None;
    }
    Some(target.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basit_baglanti() {
        assert_eq!(parse("bkz [[WIF Notu]] devam"), vec!["WIF Notu"]);
    }

    #[test]
    fn takma_adli_baglanti_hedefi_verir() {
        assert_eq!(parse("[[WIF Notu|haftalık]]"), vec!["WIF Notu"]);
    }

    #[test]
    fn baslik_capasi_temizlenir() {
        assert_eq!(parse("[[WIF Notu#Kararlar]]"), vec!["WIF Notu"]);
    }

    #[test]
    fn capa_ve_takma_ad_birlikte() {
        assert_eq!(parse("[[WIF Notu#Kararlar|kararlar]]"), vec!["WIF Notu"]);
    }

    #[test]
    fn gomme_de_iliskidir() {
        assert_eq!(parse("![[Şema.png]]"), vec!["Şema.png"]);
    }

    #[test]
    fn ayni_satirda_birden_fazla() {
        assert_eq!(parse("[[A]] ve [[B]]"), vec!["A", "B"]);
    }

    #[test]
    fn tekrarlar_temizlenir_sira_korunur() {
        assert_eq!(parse("[[B]] [[A]] [[B]]"), vec!["B", "A"]);
    }

    #[test]
    fn kod_blogundaki_baglanti_sayilmaz() {
        let body = "[[Gerçek]]\n```\n[[Örnek]]\n```\n[[Gerçek2]]";
        assert_eq!(parse(body), vec!["Gerçek", "Gerçek2"]);
    }

    #[test]
    fn kapanmayan_baglanti_yok_sayilir() {
        assert!(parse("[[yarım").is_empty());
    }

    #[test]
    fn bos_baglanti_yok_sayilir() {
        assert!(parse("[[]] [[  ]]").is_empty());
    }

    #[test]
    fn turkce_ve_emoji_hedefler() {
        assert_eq!(parse("[[Öğrenciyiz 🌱 planı]]"), vec!["Öğrenciyiz 🌱 planı"]);
    }

    #[test]
    fn bos_govde() {
        assert!(parse("").is_empty());
    }
}
