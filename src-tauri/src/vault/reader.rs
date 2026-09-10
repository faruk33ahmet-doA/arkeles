/*!
Vault okuma — Anayasa madde 15.1, 15.2.

15.1  Markdown dosyaları ARAYÜZ TARAFINDAN doğrudan taranmaz.
15.2  Bütün okuma index üzerinden yapılır.

Yani bu modülün müşterisi arayüz DEĞİL, index oluşturucudur.

Hata felsefesi: tek bir bozuk dosya taramayı ÇÖKERTMEZ. Okunamayan dosya
atlanır, bozuk frontmatter boş sayılır (bkz. frontmatter.rs). Sebep madde
18.4 mantığı — arıza sakin bir durum, kırmızı bir uyarı değil.
*/

use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::vault::note::ParsedNote;
use crate::vault::{frontmatter, id, links, tasks};

/// Anayasa madde 34.1: bir notun index'lenmesi ucuz olmalı. 5 MB'ın üstündeki
/// dosyalar (madde 39.3 test vakası) gövdesi KIRPILARAK index'lenir — not
/// kaybolmaz, aranabilir kalır, ama bellek patlamaz.
const MAX_BODY_BYTES: usize = 1024 * 1024;

/// Vault dizini okunabilir durumda mı?
pub fn is_readable(path: &Path) -> bool {
    path.is_dir() && std::fs::read_dir(path).is_ok()
}

/// Anayasa madde 20.5: senkronizasyon artefaktları yok sayılır.
pub fn is_sync_artifact(file_name: &str) -> bool {
    file_name.ends_with(".icloud")
        || file_name.starts_with('.')
        || file_name.starts_with("~$")
        || file_name.contains("conflicted copy")
        || file_name.contains("(Çakışan")
}

/// Vault içindeki bütün markdown dosyalarını toplar.
///
/// Gizli klasörler (`.obsidian`, `.git`, `.trash`) atlanır — madde 20.5.
pub fn collect_markdown_files(vault_root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    walk(vault_root, &mut out);
    out
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return; // erişilemeyen klasör sessizce atlanır
    };

    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if is_sync_artifact(&name) {
            continue;
        }

        let path = entry.path();
        match entry.file_type() {
            Ok(ft) if ft.is_dir() => walk(&path, out),
            Ok(ft) if ft.is_file() => {
                if path.extension().is_some_and(|e| e.eq_ignore_ascii_case("md")) {
                    out.push(path);
                }
            }
            _ => {}
        }
    }
}

/// Tek bir notu okur ve ayrıştırır.
///
/// `None` döner yalnızca dosya okunamıyorsa. Bozuk içerik `None` DEĞİLDİR —
/// frontmatter boş sayılır, not yine index'lenir (madde 26.2 mantığı).
pub fn read_note(vault_root: &Path, path: &Path) -> Option<ParsedNote> {
    let raw = std::fs::read(path).ok()?;
    let content = String::from_utf8_lossy(&raw);

    let modified_at = file_modified_iso(path);
    let source_path = relative_path(vault_root, path);
    let content_hash = hash(&raw);

    let (fm, body_full) = frontmatter::split(&content);
    let body = truncate_on_char_boundary(body_full, MAX_BODY_BYTES);

    // Görev satır numaraları DOSYAYA göre olmalı: gövdenin başladığı satırı bul.
    let body_offset = content.len() - body_full.len();
    let line_offset = content[..body_offset].lines().count() as u32;

    // Anayasa madde 16.1/16.3: kalıcı ULID varsa yönetilir, yoksa yönetilmez.
    let declared_id = frontmatter::string_field(&fm, id::ID_KEY);
    let (note_id, managed) = match declared_id {
        Some(candidate) if id::is_valid_ulid(&candidate) => (candidate, true),
        _ => (id::transient_id(&source_path), false),
    };

    Some(ParsedNote {
        id: note_id,
        managed,
        title: derive_title(&fm, body, path),
        workspace: derive_workspace(&fm, &source_path),
        tasks: tasks::parse(body, line_offset),
        links: links::parse(body),
        body: body.to_string(),
        frontmatter: fm,
        source_path,
        content_hash,
        modified_at,
    })
}

/// Başlık sırası: frontmatter `title` → ilk `# H1` → dosya adı.
fn derive_title(
    fm: &serde_json::Map<String, serde_json::Value>,
    body: &str,
    path: &Path,
) -> String {
    if let Some(t) = frontmatter::string_field(fm, "title") {
        return t;
    }
    for line in body.lines().take(40) {
        if let Some(h1) = line.trim_start().strip_prefix("# ") {
            let t = h1.trim();
            if !t.is_empty() {
                return t.to_string();
            }
        }
    }
    path.file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| "Adsız".to_string())
}

/// Çalışma alanı (madde 36.3): frontmatter `workspace` → en üst klasör adı.
fn derive_workspace(
    fm: &serde_json::Map<String, serde_json::Value>,
    source_path: &str,
) -> Option<String> {
    if let Some(w) = frontmatter::string_field(fm, "workspace") {
        return Some(w);
    }
    // "İş/WIF/notlar/a.md" → "İş" değil "WIF" daha anlamlı olurdu ama
    // klasör düzenini VARSAYMAK yanlış olur (madde 6.3: şüphede eklemeyiz).
    // Bu yüzden yalnız TEK seviyeli yerleşimde ilk klasörü alırız.
    let mut parts = source_path.split('/');
    let first = parts.next()?;
    if parts.next().is_some() {
        Some(first.to_string())
    } else {
        None // kökteki notun çalışma alanı yoktur
    }
}

fn relative_path(vault_root: &Path, path: &Path) -> String {
    path.strip_prefix(vault_root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

fn file_modified_iso(path: &Path) -> String {
    let secs = std::fs::metadata(path)
        .and_then(|m| m.modified())
        .and_then(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| std::io::Error::other(e.to_string()))
        })
        .map(|d| d.as_secs())
        .unwrap_or(0);
    crate::vault::time::unix_to_iso(secs)
}

/// Madde 15.6: içerik hash'i. Aynıysa yeniden indeksleme yapılmaz.
pub fn hash(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    // İlk 16 bayt yeterli: çakışma olasılığı pratikte sıfır, index küçük kalır.
    digest[..16].iter().fold(String::with_capacity(32), |mut acc, b| {
        use std::fmt::Write;
        let _ = write!(acc, "{b:02x}");
        acc
    })
}

/// UTF-8 sınırını bozmadan kırpar — emoji ortasından kesmez.
fn truncate_on_char_boundary(s: &str, max: usize) -> &str {
    if s.len() <= max {
        return s;
    }
    let mut end = max;
    while end > 0 && !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
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

    #[test]
    fn hash_kararli_ve_icerige_duyarli() {
        assert_eq!(hash(b"abc"), hash(b"abc"));
        assert_ne!(hash(b"abc"), hash(b"abd"));
        assert_eq!(hash(b"abc").len(), 32);
    }

    #[test]
    fn kirpma_utf8_sinirini_bozmaz() {
        // "🌱" 4 bayt. 2 baytta kırpmak istenirse 0'a düşmeli.
        let s = "🌱";
        assert_eq!(truncate_on_char_boundary(s, 2), "");
        assert_eq!(truncate_on_char_boundary(s, 4), "🌱");
    }
}
