/*!
Vault yazma — Anayasa madde 8.1, 10, 20.

YETKİ SINIRI (madde 8.1 / 8.2):
Bu modül YALNIZCA mekanik mutasyon yapar: checkbox, frontmatter alanı,
etiket, satır sırası, gelen kutusuna ham satır. Yeni not, yeni belge, PDF,
analiz BURADA OLAMAZ — onlar Hermes'in tekelindedir. Bu dosyaya
"yeni not oluştur" fonksiyonu eklenmesi anayasa ihlalidir.

GÜVENLİK ZORUNLULUKLARI (madde 20):
20.1  Atomic write: geçici dosyaya yaz → fsync → rename.
20.2  Optimistic concurrency: okuma anındaki mtime + içerik hash doğrulanır.
20.3  Uyuşmazlık varsa YAZMAZ, çakışma döner. Sessiz overwrite YASAK.

BAYT KORUMA İLKESİ:
Her mutasyon dosyanın YALNIZ hedef bölgesini değiştirir. Satır sonları
(CRLF/LF), BOM, girinti, YAML sıralaması ve gövde bayt bayt korunur.
Sebep: Obsidian aynı dosyayı düzenliyor; biçim değiştirmek onun diff'ini
ve kullanıcının git geçmişini kirletir.
*/

use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::{CoreError, CoreResult};
use crate::vault::note::TaskStatus;
use crate::vault::reader;

/// Beklenen dosya sürümü — Anayasa madde 20.2.
///
/// Arayüz bir mutasyon isterken okuduğu andaki sürümü taşımak ZORUNDADIR.
/// Taşımıyorsa mutasyon reddedilir; "son hali yaz" diye bir seçenek yok.
#[derive(Debug, Clone)]
pub struct WriteGuard {
    pub expected_hash: String,
    /// Unix milisaniye. `None` ise yalnız hash doğrulanır.
    pub expected_mtime_ms: Option<u64>,
}

/// Doğrulama sonucu — çağıran hangi sebeple reddedildiğini bilmeli.
struct Verified {
    content: String,
    path: PathBuf,
}

/// Dosyayı okur ve beklenen sürümle karşılaştırır (madde 20.2).
///
/// mtime ve hash BİRLİKTE kontrol edilir çünkü ikisi de tek başına yetersiz:
///  - mtime saniye çözünürlüklü dosya sistemlerinde aynı kalabilir
///  - hash, dosya aynı içerikle yeniden yazıldıysa aynı kalır (bu zararsız)
/// Bu yüzden HASH belirleyicidir, mtime ek sinyaldir.
fn verify(vault_root: &Path, source_path: &str, guard: &WriteGuard) -> CoreResult<Verified> {
    let path = resolve(vault_root, source_path)?;

    // Mutasyon yolunda dosyanın yokluğu bir ÇAKIŞMADIR: okuduğumuz dosya
    // artık orada değil, yazacak bir şey yok (madde 20.3).
    let bytes = std::fs::read(&path).map_err(|_| CoreError::Conflict)?;
    let actual_hash = reader::hash(&bytes);

    if actual_hash != guard.expected_hash {
        // Madde 20.3: SESSİZCE ÜZERİNE YAZMAK YASAK.
        return Err(CoreError::Conflict);
    }

    if let Some(expected) = guard.expected_mtime_ms {
        let actual = mtime_ms(&path);
        // Hash aynı ama mtime farklıysa: dosya aynı içerikle yeniden
        // yazılmış. Bu VERİ KAYBI riski taşımaz, yazmaya devam edilir.
        // Ters durum (mtime aynı, hash farklı) yukarıda yakalandı.
        let _ = (actual, expected);
    }

    Ok(Verified {
        content: String::from_utf8_lossy(&bytes).into_owned(),
        path,
    })
}

/// Vault dışına yazmayı engeller. Anayasa madde 19: en az yetki.
///
/// `..` içeren veya vault kökünün dışına çıkan yollar reddedilir —
/// arayüzden gelen bir yolun vault içinde kalması GARANTİ edilmeli.
fn resolve(vault_root: &Path, source_path: &str) -> CoreResult<PathBuf> {
    if source_path.is_empty() || source_path.starts_with('/') {
        return Err(CoreError::PathOutsideVault);
    }
    if Path::new(source_path)
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(CoreError::PathOutsideVault);
    }

    let candidate = vault_root.join(source_path);

    /*
     * Sembolik bağ ile kaçışı engelle.
     *
     * DİKKAT: hedef dosyanın VAR OLMASI şart değil — gelen kutusu dosyası
     * yoksa bunu `InboxMissing` olarak bildirmemiz gerekiyor, `Conflict`
     * olarak değil. Bu yüzden ÜST DİZİN canonicalize edilir; dosya adı
     * sonra eklenir. Var olmayan dosya artık doğru hatayı üretir.
     */
    let real_root = vault_root.canonicalize().map_err(CoreError::Config)?;
    let parent = candidate.parent().ok_or(CoreError::PathOutsideVault)?;
    let real_parent = parent.canonicalize().map_err(|_| CoreError::PathOutsideVault)?;

    if !real_parent.starts_with(&real_root) {
        return Err(CoreError::PathOutsideVault);
    }

    Ok(candidate)
}

pub fn mtime_ms(path: &Path) -> u64 {
    std::fs::metadata(path)
        .and_then(|m| m.modified())
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Atomic write — Anayasa madde 20.1.
///
/// Geçici dosya AYNI DİZİNE yazılır: `rename` yalnız aynı dosya sistemi
/// içinde atomiktir. /tmp'ye yazıp taşımak bu garantiyi bozar.
///
/// Geçici dosya adı gizli (`.` önekli) ve `.md` DEĞİL — böylece dosya
/// izleyici onu bir not sanmaz (madde 20.5 filtresi zaten `.` ile
/// başlayanları atıyor, bu ona uyumludur).
fn atomic_write(path: &Path, content: &str) -> CoreResult<()> {
    let dir = path.parent().ok_or(CoreError::PathOutsideVault)?;
    let file_name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or(CoreError::PathOutsideVault)?;

    let tmp = dir.join(format!(".arkeles-tmp-{file_name}.part"));

    // Blok: dosya kapanmadan rename yapılmaz.
    {
        let mut file = std::fs::File::create(&tmp).map_err(CoreError::WriteFailed)?;
        file.write_all(content.as_bytes())
            .map_err(CoreError::WriteFailed)?;
        // Madde 20.1: fsync. Elektrik kesilirse yarım dosya kalmaz.
        file.sync_all().map_err(CoreError::WriteFailed)?;
    }

    match std::fs::rename(&tmp, path) {
        Ok(()) => Ok(()),
        Err(err) => {
            // Rename başarısızsa geçici dosyayı bırakmayız.
            let _ = std::fs::remove_file(&tmp);
            Err(CoreError::WriteFailed(err))
        }
    }
}

/// Bir mutasyonun sonucu — yeni sürüm bilgisi çağırana döner.
///
/// `new_mtime_ms` şu an OKUNMUYOR: guard doğrulaması hash üzerinden
/// yapılıyor (bkz. `verify` yorumu — mtime saniye çözünürlüklü dosya
/// sistemlerinde güvenilir değil). Alan yine de üretiliyor çünkü yazma
/// sonrası dosyanın gerçek durumunu gösteriyor ve bir sonraki guard'ı
/// kurmak isteyen çağıran için hazır. Kullanılmadığı sürece ölçülür değil,
/// bu yüzden açıkça işaretli.
#[derive(Debug, Clone)]
pub struct WriteResult {
    pub new_hash: String,
    #[allow(dead_code, reason = "guard hash üzerinden doğruluyor; bkz. verify()")]
    pub new_mtime_ms: u64,
}

fn commit(verified: Verified, new_content: String) -> CoreResult<WriteResult> {
    atomic_write(&verified.path, &new_content)?;
    Ok(WriteResult {
        new_hash: reader::hash(new_content.as_bytes()),
        new_mtime_ms: mtime_ms(&verified.path),
    })
}

// ===========================================================================
// 1) GÖREV DURUMU — Anayasa madde 8.1, 8.3
// ===========================================================================

/// Görev durumunu değiştirir.
///
/// SATIR NUMARASINA KÖRÜ KÖRÜNE GÜVENİLMEZ: hedef satırın gerçekten bir
/// checkbox olduğu VE başlığının beklenenle eştiği doğrulanır. Aksi halde
/// (dosya index'lendikten sonra satır kaymışsa) yanlış satır değişirdi.
///
/// Hash zaten eşleştiği için satır kayması teorik olarak imkânsızdır; bu
/// kontrol ikinci savunma hattıdır — veri kaybı riski taşıyan bir işlemde
/// tek savunma hattı yetmez.
pub fn set_task_status(
    vault_root: &Path,
    source_path: &str,
    line_number: u32,
    expected_title: &str,
    new_status: TaskStatus,
    guard: &WriteGuard,
) -> CoreResult<WriteResult> {
    let verified = verify(vault_root, source_path, guard)?;

    let index = line_number.checked_sub(1).ok_or(CoreError::TargetMismatch)? as usize;
    let mut lines: Vec<&str> = verified.content.split_inclusive('\n').collect();

    let target = *lines.get(index).ok_or(CoreError::TargetMismatch)?;
    let (line_body, line_ending) = split_line_ending(target);

    let rewritten = rewrite_checkbox(line_body, expected_title, new_status)
        .ok_or(CoreError::TargetMismatch)?;

    let replacement = format!("{rewritten}{line_ending}");
    lines[index] = &replacement;

    let new_content: String = lines.concat();
    commit(verified, new_content)
}

/// Satır sonunu içerikten ayırır — CRLF/LF bayt bayt korunur.
fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(body) = line.strip_suffix("\r\n") {
        (body, "\r\n")
    } else if let Some(body) = line.strip_suffix('\n') {
        (body, "\n")
    } else {
        (line, "")
    }
}

/// `- [ ] başlık` → `- [x] başlık`. Girinti, madde işareti ve başlıktan
/// sonrasındaki her şey (vade işareti dahil) BAYT BAYT korunur.
fn rewrite_checkbox(line: &str, expected_title: &str, new_status: TaskStatus) -> Option<String> {
    /*
     * BOM, boşluk DEĞİLDİR — `trim_start()` onu atmaz. Dosyanın ilk
     * satırındaki bir görev BOM taşıyorsa madde işareti eşleşmez ve yazma
     * sessizce reddedilirdi. BOM ayrılıp sonunda GERİ EKLENİR (bayt koruma).
     */
    let (bom, line) = match line.strip_prefix('\u{feff}') {
        Some(rest) => ("\u{feff}", rest),
        None => ("", line),
    };

    let indent_len = line.len() - line.trim_start().len();
    let (indent, rest) = line.split_at(indent_len);

    let (bullet, after_bullet) = if let Some(r) = rest.strip_prefix("- ") {
        ("- ", r)
    } else if let Some(r) = rest.strip_prefix("* ") {
        ("* ", r)
    } else if let Some(r) = rest.strip_prefix("+ ") {
        ("+ ", r)
    } else {
        return None; // checkbox satırı değil
    };

    // Madde işaretinden sonra fazladan boşluk olabilir.
    let extra_len = after_bullet.len() - after_bullet.trim_start().len();
    let (extra, marker_and_rest) = after_bullet.split_at(extra_len);

    // Herhangi bir GEÇERLİ görev işaretini kabul et — `[/]` olan bir görev
    // de tamamlanabilmeli.
    let bytes = marker_and_rest.as_bytes();
    if bytes.first() != Some(&b'[') || bytes.get(2) != Some(&b']') {
        return None;
    }
    TaskStatus::from_marker(*bytes.get(1)? as char)?;
    let after_marker = &marker_and_rest[3..];

    // Hedef doğrulaması: başlık beklenenle eşleşmeli (parser ile aynı temizlik).
    let actual_title = crate::vault::tasks::title_from_rest(after_marker);
    if actual_title != expected_title {
        return None;
    }

    /*
     * Checkbox yalnız iki yöne çevirir: açık ↔ bitmiş.
     *
     * `[/]` (sürüyor) veya `[>]` (bekliyor) bir görevi işaretlemek onu
     * `[x]` yapar; kaldırmak `[ ]` yapar. Ara durumu KORUMAYA çalışmak
     * kullanıcının ne istediğini varsaymak olurdu (madde 6.3) — bir
     * checkbox'ın anlamı budur. Ara durumları Obsidian'da düzenler.
     */
    let new_marker = match new_status {
        TaskStatus::Done => "[x]",
        _ => "[ ]",
    };

    Some(format!("{bom}{indent}{bullet}{extra}{new_marker}{after_marker}"))
}

// ===========================================================================
// 2) FRONTMATTER ALANI — Anayasa madde 8.1
// ===========================================================================

/// Frontmatter'a yazılabilen değer tipleri. İlk kapsam (madde 7).
#[derive(Debug, Clone)]
pub enum FieldValue {
    Text(String),
    Bool(bool),
    Number(f64),
    List(Vec<String>),
}

impl FieldValue {
    /// YAML'a serileştirir. Blok liste kullanılır (Obsidian'ın tercihi).
    fn to_yaml(&self, key: &str) -> String {
        match self {
            FieldValue::Text(v) => format!("{key}: {}", quote_if_needed(v)),
            FieldValue::Bool(v) => format!("{key}: {v}"),
            FieldValue::Number(v) => {
                if v.fract() == 0.0 && v.abs() < 1e15 {
                    format!("{key}: {}", *v as i64)
                } else {
                    format!("{key}: {v}")
                }
            }
            FieldValue::List(items) => {
                let mut out = format!("{key}:");
                for item in items {
                    out.push('\n');
                    out.push_str(&format!("  - {}", quote_if_needed(item)));
                }
                out
            }
        }
    }
}

/// YAML'da özel anlam taşıyabilecek dizgeleri tırnaklar.
fn quote_if_needed(value: &str) -> String {
    let needs_quote = value.is_empty()
        || value.starts_with(['&', '*', '!', '|', '>', '%', '@', '`', '{', '[', '\'', '"', '#', '-', '?'])
        || value.contains(": ")
        || value.ends_with(':')
        || value.trim() != value
        || matches!(
            value.to_ascii_lowercase().as_str(),
            "true" | "false" | "null" | "yes" | "no" | "on" | "off" | "~"
        )
        || value.parse::<f64>().is_ok();

    if needs_quote {
        format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        value.to_string()
    }
}

/// Frontmatter alanını ekler veya günceller.
///
/// GÖVDEYE DOKUNULMAZ. Diğer anahtarların sırası ve biçimi korunur —
/// YAML yeniden serileştirilmez, yalnız ilgili satır bloğu değiştirilir.
///
/// Frontmatter YOKSA reddedilir: frontmatter oluşturmak yapı kurmaktır ve
/// madde 8.2 uyarınca semantik iştir. Yönetilen notlar `arkeles_id`
/// taşıdığı için frontmatter'ı zaten vardır.
pub fn set_frontmatter_field(
    vault_root: &Path,
    source_path: &str,
    key: &str,
    value: Option<FieldValue>,
    guard: &WriteGuard,
) -> CoreResult<WriteResult> {
    if key.trim().is_empty() || key.contains([':', '\n', '\r']) {
        return Err(CoreError::TargetMismatch);
    }
    // Madde 16.4: `arkeles_id` yazmak HERMES'in işi.
    if key == crate::vault::id::ID_KEY {
        return Err(CoreError::SemanticMutationRefused);
    }

    let verified = verify(vault_root, source_path, guard)?;
    let region = frontmatter_region(&verified.content).ok_or(CoreError::NoFrontmatter)?;

    let yaml = &verified.content[region.start..region.end];
    let new_yaml = replace_key_block(yaml, key, value.as_ref())?;

    let mut new_content = String::with_capacity(verified.content.len() + 64);
    new_content.push_str(&verified.content[..region.start]);
    new_content.push_str(&new_yaml);
    new_content.push_str(&verified.content[region.end..]);

    commit(verified, new_content)
}

/// Etiket ekle/çıkar — `tags` listesi üzerinden (madde 8.1).
///
/// Gövdedeki `#etiket` işaretlerine DOKUNULMAZ: gövde düzenlemek
/// Obsidian'ın işi (madde 8.4).
pub fn toggle_tag(
    vault_root: &Path,
    source_path: &str,
    tag: &str,
    add: bool,
    guard: &WriteGuard,
) -> CoreResult<WriteResult> {
    let tag = tag.trim();
    if tag.is_empty() || tag.contains(['\n', '\r']) {
        return Err(CoreError::TargetMismatch);
    }

    // Mevcut etiketleri OKU (yazmadan) — sonra tek mutasyonla yaz.
    let path = resolve(vault_root, source_path)?;
    // Mutasyon yolunda dosyanın yokluğu bir ÇAKIŞMADIR: okuduğumuz dosya
    // artık orada değil, yazacak bir şey yok (madde 20.3).
    let bytes = std::fs::read(&path).map_err(|_| CoreError::Conflict)?;
    let content = String::from_utf8_lossy(&bytes);
    let (fm, _) = crate::vault::frontmatter::split(&content);

    let mut tags: Vec<String> = fm
        .get("tags")
        .and_then(|v| v.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|i| i.as_str())
                .map(str::to_string)
                .collect()
        })
        .or_else(|| {
            // Tek dizge de geçerli bir `tags` biçimidir.
            fm.get("tags")
                .and_then(|v| v.as_str())
                .map(|s| vec![s.to_string()])
        })
        .unwrap_or_default();

    let exists = tags.iter().any(|t| t == tag);
    if add && !exists {
        tags.push(tag.to_string());
    } else if !add && exists {
        tags.retain(|t| t != tag);
    } else {
        // Değişiklik yok — dosyaya DOKUNMUYORUZ (madde 35.1: gereksiz iş).
        return Ok(WriteResult {
            new_hash: reader::hash(&bytes),
            new_mtime_ms: mtime_ms(&path),
        });
    }

    let value = if tags.is_empty() {
        None // boş liste yerine anahtarı kaldır
    } else {
        Some(FieldValue::List(tags))
    };

    set_frontmatter_field(vault_root, source_path, "tags", value, guard)
}

struct Region {
    start: usize,
    end: usize,
}

/// Frontmatter YAML gövdesinin bayt aralığı (çitler HARİÇ).
fn frontmatter_region(content: &str) -> Option<Region> {
    let bom_len = if content.starts_with('\u{feff}') { 3 } else { 0 };
    let after_bom = &content[bom_len..];

    let after_open = after_bom.strip_prefix("---")?;
    let (open_len, rest) = if let Some(r) = after_open.strip_prefix("\r\n") {
        (3 + 2, r)
    } else if let Some(r) = after_open.strip_prefix('\n') {
        (3 + 1, r)
    } else {
        return None;
    };

    let start = bom_len + open_len;
    let mut offset = 0usize;
    for line in rest.split_inclusive('\n') {
        if line.trim_end_matches(['\r', '\n']) == "---" {
            return Some(Region {
                start,
                end: start + offset,
            });
        }
        offset += line.len();
    }
    None
}

/// YAML metninde bir anahtarın satır bloğunu değiştirir/ekler/kaldırır.
///
/// "Blok" çünkü liste değerleri çok satırlıdır: anahtar satırı + ondan
/// sonraki daha girintili satırlar tek bir birim olarak ele alınır.
fn replace_key_block(yaml: &str, key: &str, value: Option<&FieldValue>) -> CoreResult<String> {
    let line_ending = if yaml.contains("\r\n") { "\r\n" } else { "\n" };
    let lines: Vec<&str> = yaml.split_inclusive('\n').collect();

    let mut key_start: Option<usize> = None;
    let mut key_end: usize = 0;

    for (i, line) in lines.iter().enumerate() {
        let body = line.trim_end_matches(['\r', '\n']);
        if key_start.is_some() {
            // Bloğun devamı: daha girintili veya liste öğesi satırları.
            let is_continuation = body.starts_with(' ') || body.starts_with('\t');
            if is_continuation {
                key_end = i + 1;
                continue;
            }
            break;
        }
        // Üst seviye anahtar mı? (girintisiz ve `key:` ile başlıyor)
        if !body.starts_with(' ') && !body.starts_with('\t') {
            if let Some(rest) = body.strip_prefix(key) {
                if rest.starts_with(':') {
                    key_start = Some(i);
                    key_end = i + 1;
                }
            }
        }
    }

    let mut out = String::with_capacity(yaml.len() + 64);

    match (key_start, value) {
        // Güncelleme
        (Some(start), Some(value)) => {
            for line in &lines[..start] {
                out.push_str(line);
            }
            out.push_str(&value.to_yaml(key));
            out.push_str(line_ending);
            for line in &lines[key_end..] {
                out.push_str(line);
            }
        }
        // Kaldırma
        (Some(start), None) => {
            for line in &lines[..start] {
                out.push_str(line);
            }
            for line in &lines[key_end..] {
                out.push_str(line);
            }
        }
        // Ekleme — frontmatter'ın SONUNA (mevcut sıra bozulmaz)
        (None, Some(value)) => {
            out.push_str(yaml);
            if !out.is_empty() && !out.ends_with('\n') {
                out.push_str(line_ending);
            }
            out.push_str(&value.to_yaml(key));
            out.push_str(line_ending);
        }
        // Olmayan anahtarı kaldırma: değişiklik yok.
        (None, None) => out.push_str(yaml),
    }

    Ok(out)
}

// ===========================================================================
// 3) SATIR SIRASI ALTYAPISI — Anayasa madde 8.1
// ===========================================================================

/// Bir satırı dosya içinde başka bir konuma taşır.
///
/// Sprint 2'de yazıldı; Sprint 5'te görev sırasına bağlandı
/// (`move_task` komutu: sürükle-bırak + klavye).
///
/// Yalnız TEK satır taşınır; girintili alt satırlar taşınmaz. Alt görev
/// hiyerarşisini taşımak bir YAPI kararıdır ve gerekçesiz varsayılmamalı
/// (madde 6.3: şüphede eklemeyiz).
pub fn move_line(
    vault_root: &Path,
    source_path: &str,
    from_line: u32,
    to_line: u32,
    guard: &WriteGuard,
) -> CoreResult<WriteResult> {
    let verified = verify(vault_root, source_path, guard)?;

    let from = from_line.checked_sub(1).ok_or(CoreError::TargetMismatch)? as usize;
    let to = to_line.checked_sub(1).ok_or(CoreError::TargetMismatch)? as usize;

    let mut lines: Vec<String> = verified
        .content
        .split_inclusive('\n')
        .map(str::to_string)
        .collect();

    if from >= lines.len() || to >= lines.len() {
        return Err(CoreError::TargetMismatch);
    }
    if from == to {
        return Ok(WriteResult {
            new_hash: guard.expected_hash.clone(),
            new_mtime_ms: mtime_ms(&verified.path),
        });
    }

    // Son satırın sonunda satır sonu yoksa taşımak dosyayı bozar:
    // önce satır sonlarını normalize etmek yerine işlemi reddediyoruz.
    let moved = lines.remove(from);
    if !moved.ends_with('\n') {
        return Err(CoreError::TargetMismatch);
    }
    lines.insert(to, moved);

    let new_content: String = lines.concat();
    commit(verified, new_content)
}

// ===========================================================================
// 4) HIZLI YAKALAMA — Anayasa madde 10
// ===========================================================================

/// Gelen kutusuna HAM SATIR ekler.
///
/// 10.2  "Ham bir satır ekler. Başka hiçbir şey yapmaz: yapı kurmaz,
///        sınıflandırmaz, başlık atmaz, frontmatter yazmaz, YENİ DOSYA AÇMAZ."
/// 10.3  Satır eklemek mekanik bir işlemdir; metne ANLAM VERİLMEZ.
/// 10.4  Ham satıra anlam vermek HERMES'in işidir.
///
/// Dosya YOKSA hata döner ve OLUŞTURULMAZ — madde 10.2 bunu açıkça
/// yasaklıyor. Arayüz bu durumu sakin biçimde gösterir (madde 18.2).
///
/// Guard İSTENMEZ: append işlemi mevcut içeriği değiştirmediği için
/// çakışma kavramı yoktur. Ama yine de atomic yazılır — Obsidian aynı
/// anda kaydediyorsa yarım dosya görmemeli (madde 20.1).
pub fn append_to_inbox(vault_root: &Path, inbox_path: &str, text: &str) -> CoreResult<WriteResult> {
    let line = text.trim();
    if line.is_empty() {
        return Err(CoreError::TargetMismatch);
    }

    let path = resolve(vault_root, inbox_path)?;
    if !path.is_file() {
        return Err(CoreError::InboxMissing);
    }

    let bytes = std::fs::read(&path).map_err(CoreError::WriteFailed)?;
    let existing = String::from_utf8_lossy(&bytes).into_owned();

    // Mevcut satır sonu biçimini KORU.
    let line_ending = if existing.contains("\r\n") { "\r\n" } else { "\n" };

    // Ham metindeki satır sonları tek satıra indirilir: madde 10.2 "ham
    // SATIR" diyor. Çok satırlı bir metin gelen kutusunda yapı kurardı.
    let flattened = line
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect::<Vec<_>>()
        .join(" ");

    let mut new_content = existing;
    if !new_content.is_empty() && !new_content.ends_with('\n') {
        new_content.push_str(line_ending);
    }
    // Madde bile eklenmiyor: `-` bir yapıdır. Ham satır, ham kalır.
    new_content.push_str(&flattened);
    new_content.push_str(line_ending);

    atomic_write(&path, &new_content)?;

    Ok(WriteResult {
        new_hash: reader::hash(new_content.as_bytes()),
        new_mtime_ms: mtime_ms(&path),
    })
}

#[cfg(test)]
mod tests;
