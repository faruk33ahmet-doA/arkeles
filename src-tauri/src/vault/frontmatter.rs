/*!
Frontmatter ayrıştırma — Anayasa madde 39.3.

"Zorunlu test kapsamı: Rust markdown parser'ı ve index katmanı.
 En riskli ve en sessiz bozulan yer burasıdır."

Bu yüzden bu modül HATA FIRLATMAZ. Bozuk YAML bir not'u index dışında
bırakmaz — frontmatter boş sayılır, gövde yine okunur, not yine aranabilir.
Sebep madde 26.2 ve 18.4 mantığı: tek bir bozuk dosya uygulamayı
kırmamalı, sessizce ve zararsızca yönetilmeyen sayılmalı.
*/

use serde_json::{Map, Value};

/// Markdown içeriğini (frontmatter, gövde) olarak böler.
///
/// Frontmatter yoksa `(boş map, tüm içerik)` döner.
/// Frontmatter bozuksa `(boş map, frontmatter SONRASI gövde)` döner —
/// yani bozuk YAML gövdeye sızmaz.
pub fn split(content: &str) -> (Map<String, Value>, &str) {
    let Some(rest) = strip_opening_fence(content) else {
        return (Map::new(), content);
    };

    // Kapanış çitini bul: satır başında tam olarak "---".
    let Some((raw_yaml, body)) = split_at_closing_fence(rest) else {
        // Açılış var, kapanış yok → frontmatter sayılmaz, hepsi gövdedir.
        return (Map::new(), content);
    };

    (parse_yaml_map(raw_yaml), body)
}

/// BOM ve baştaki boş satırları atlayıp açılış `---` çitini tüketir.
fn strip_opening_fence(content: &str) -> Option<&str> {
    // BOM: madde 39.3 test vakası.
    let s = content.strip_prefix('\u{feff}').unwrap_or(content);

    // Frontmatter DOSYANIN İLK satırında olmak zorundadır (Obsidian davranışı).
    let after = s.strip_prefix("---")?;
    // Çitten sonra yalnız satır sonu gelebilir (CRLF dahil — test vakası).
    let after = after.strip_prefix("\r\n").or_else(|| after.strip_prefix('\n'))?;
    Some(after)
}

fn split_at_closing_fence(rest: &str) -> Option<(&str, &str)> {
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            let yaml = &rest[..offset];
            let body = &rest[offset + line.len()..];
            return Some((yaml, body));
        }
        offset += line.len();
    }
    None
}

/// YAML'ı JSON haritasına çevirir. Bozuksa BOŞ harita döner, panik yok.
fn parse_yaml_map(raw: &str) -> Map<String, Value> {
    if raw.trim().is_empty() {
        return Map::new();
    }
    match serde_yaml::from_str::<serde_yaml::Value>(raw) {
        Ok(serde_yaml::Value::Mapping(map)) => {
            let mut out = Map::new();
            for (k, v) in map {
                // Anahtar dizge değilse atlanır — sessizce (madde 19.5: içerik loglanmaz).
                if let Some(key) = yaml_key_to_string(&k) {
                    out.insert(key, yaml_to_json(v));
                }
            }
            out
        }
        // Mapping olmayan frontmatter (liste, düz dizge) anlamsızdır → boş.
        _ => Map::new(),
    }
}

fn yaml_key_to_string(key: &serde_yaml::Value) -> Option<String> {
    match key {
        serde_yaml::Value::String(s) => Some(s.clone()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        serde_yaml::Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

fn yaml_to_json(value: serde_yaml::Value) -> Value {
    match value {
        serde_yaml::Value::Null => Value::Null,
        serde_yaml::Value::Bool(b) => Value::Bool(b),
        serde_yaml::Value::Number(n) => n
            .as_i64()
            .map(Value::from)
            .or_else(|| n.as_f64().and_then(serde_json::Number::from_f64).map(Value::Number))
            .unwrap_or(Value::Null),
        serde_yaml::Value::String(s) => Value::String(s),
        serde_yaml::Value::Sequence(items) => {
            Value::Array(items.into_iter().map(yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let mut out = Map::new();
            for (k, v) in map {
                if let Some(key) = yaml_key_to_string(&k) {
                    out.insert(key, yaml_to_json(v));
                }
            }
            Value::Object(out)
        }
        serde_yaml::Value::Tagged(tagged) => yaml_to_json(tagged.value),
    }
}

/// Frontmatter'dan dizge alan okur. Sayı/bool da dizgeye çevrilir.
pub fn string_field(fm: &Map<String, Value>, key: &str) -> Option<String> {
    match fm.get(key)? {
        Value::String(s) if !s.trim().is_empty() => Some(s.trim().to_string()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frontmatter_yoksa_hepsi_govdedir() {
        let (fm, body) = split("# Başlık\n\nmetin");
        assert!(fm.is_empty());
        assert_eq!(body, "# Başlık\n\nmetin");
    }

    #[test]
    fn basit_frontmatter_okunur() {
        let (fm, body) = split("---\narkeles_id: 01ARZ3NDEKTSV4RRFFQ69G5FAV\nworkspace: WIF\n---\ngövde");
        assert_eq!(string_field(&fm, "arkeles_id").unwrap(), "01ARZ3NDEKTSV4RRFFQ69G5FAV");
        assert_eq!(string_field(&fm, "workspace").unwrap(), "WIF");
        assert_eq!(body, "gövde");
    }

    #[test]
    fn bom_atlanir() {
        let (fm, _) = split("\u{feff}---\nworkspace: GEN\n---\nx");
        assert_eq!(string_field(&fm, "workspace").unwrap(), "GEN");
    }

    #[test]
    fn crlf_satir_sonu_desteklenir() {
        let (fm, body) = split("---\r\nworkspace: TüGA\r\n---\r\ngövde\r\n");
        assert_eq!(string_field(&fm, "workspace").unwrap(), "TüGA");
        assert_eq!(body, "gövde\r\n");
    }

    #[test]
    fn bozuk_yaml_bosaltir_ama_govdeyi_korur() {
        // Girinti hatası → serde_yaml reddeder. Not kaybolmamalı.
        let (fm, body) = split("---\nfoo: [unclosed\n  bar: : :\n---\ngerçek gövde");
        assert!(fm.is_empty());
        assert_eq!(body, "gerçek gövde");
    }

    #[test]
    fn kapanis_citi_yoksa_frontmatter_sayilmaz() {
        let content = "---\nworkspace: WIF\ngövde devam";
        let (fm, body) = split(content);
        assert!(fm.is_empty());
        assert_eq!(body, content);
    }

    #[test]
    fn bos_frontmatter_sorun_degil() {
        let (fm, body) = split("---\n---\ngövde");
        assert!(fm.is_empty());
        assert_eq!(body, "gövde");
    }

    #[test]
    fn bos_dosya_panik_yapmaz() {
        let (fm, body) = split("");
        assert!(fm.is_empty());
        assert_eq!(body, "");
    }

    #[test]
    fn liste_ve_nested_deger_json_olur() {
        let (fm, _) = split("---\ntags:\n  - a\n  - b\nmeta:\n  k: v\n---\n");
        assert_eq!(fm["tags"], serde_json::json!(["a", "b"]));
        assert_eq!(fm["meta"]["k"], serde_json::json!("v"));
    }

    #[test]
    fn govde_icindeki_uc_tire_frontmatter_sanilmaz() {
        let (fm, body) = split("# Başlık\n\n---\n\nayraç");
        assert!(fm.is_empty());
        assert!(body.contains("ayraç"));
    }

    #[test]
    fn emoji_iceren_frontmatter_bozulmaz() {
        let (fm, _) = split("---\nworkspace: \"Merci 🌱\"\n---\nx");
        assert_eq!(string_field(&fm, "workspace").unwrap(), "Merci 🌱");
    }
}
