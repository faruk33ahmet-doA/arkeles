/*!
Yazma testleri — Anayasa madde 39.3, 39.5.

39.5  "Vault'un bir kopyası test fixture'ı olarak tutulur.
       GERÇEK VAULT ÜZERİNDE TEST ÇALIŞTIRILMAZ."

Her test kendi geçici dizinini kurar ve düşerken siler. Hiçbir test
kullanıcının vault'una dokunmaz.
*/

use std::path::{Path, PathBuf};

use super::*;
use crate::vault::note::TaskStatus;

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "arkeles-w-{name}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&root).expect("fixture dizini");
        Self { root }
    }

    fn write(&self, rel: &str, content: &str) {
        let path = self.root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    fn write_bytes(&self, rel: &str, bytes: &[u8]) {
        std::fs::write(self.root.join(rel), bytes).unwrap();
    }

    fn read(&self, rel: &str) -> String {
        std::fs::read_to_string(self.root.join(rel)).unwrap()
    }

    fn read_bytes(&self, rel: &str) -> Vec<u8> {
        std::fs::read(self.root.join(rel)).unwrap()
    }

    fn guard(&self, rel: &str) -> WriteGuard {
        WriteGuard {
            expected_hash: crate::vault::reader::hash(&self.read_bytes(rel)),
            expected_mtime_ms: Some(mtime_ms(&self.root.join(rel))),
        }
    }

    fn path(&self) -> &Path {
        &self.root
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

// ===========================================================================
// GÖREV DURUMU — madde 8.1, 8.3
// ===========================================================================

#[test]
fn checkbox_acik_kapali_olur() {
    let fx = Fixture::new("t1");
    fx.write("a.md", "# Başlık\n\n- [ ] görev bir\n- [ ] görev iki\n");

    let result = set_task_status(
        fx.path(), "a.md", 3, "görev bir", TaskStatus::Done, &fx.guard("a.md"),
    )
    .expect("yazma başarılı olmalı");

    assert_eq!(fx.read("a.md"), "# Başlık\n\n- [x] görev bir\n- [ ] görev iki\n");
    assert!(!result.new_hash.is_empty());
}

#[test]
fn checkbox_kapali_acik_olur() {
    let fx = Fixture::new("t2");
    fx.write("a.md", "- [x] bitti\n");
    set_task_status(fx.path(), "a.md", 1, "bitti", TaskStatus::Open, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "- [ ] bitti\n");
}

#[test]
fn buyuk_x_de_acilir() {
    let fx = Fixture::new("t3");
    fx.write("a.md", "- [X] bitti\n");
    set_task_status(fx.path(), "a.md", 1, "bitti", TaskStatus::Open, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "- [ ] bitti\n");
}

#[test]
fn yalniz_hedef_satir_degisir_gerisi_bayt_bayt_korunur() {
    let fx = Fixture::new("t4");
    let original = "---\narkeles_id: 01ARZ3NDEKTSV4RRFFQ69G5FAV\n---\n# Başlık\n\n- [ ] bir\n\ttab\n- [ ] iki\n\nSon satır, sonu yok";
    fx.write("a.md", original);

    set_task_status(fx.path(), "a.md", 8, "iki", TaskStatus::Done, &fx.guard("a.md")).unwrap();

    let after = fx.read("a.md");
    assert_eq!(after, original.replace("- [ ] iki", "- [x] iki"));
    assert!(after.ends_with("Son satır, sonu yok"), "eksik satır sonu korunmalı");
}

#[test]
fn vade_isareti_ve_girinti_korunur() {
    let fx = Fixture::new("t5");
    fx.write("a.md", "    - [ ]   rapor 📅 2026-09-11 #etiket\n");

    set_task_status(fx.path(), "a.md", 1, "rapor", TaskStatus::Done, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "    - [x]   rapor 📅 2026-09-11 #etiket\n");
}

#[test]
fn crlf_satir_sonlari_korunur() {
    let fx = Fixture::new("t6");
    fx.write("a.md", "# H\r\n\r\n- [ ] görev\r\n- [ ] diğer\r\n");

    set_task_status(fx.path(), "a.md", 3, "görev", TaskStatus::Done, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "# H\r\n\r\n- [x] görev\r\n- [ ] diğer\r\n");
}

#[test]
fn bom_korunur() {
    let fx = Fixture::new("t7");
    let mut bytes = vec![0xEF, 0xBB, 0xBF];
    bytes.extend_from_slice("- [ ] görev\n".as_bytes());
    fx.write_bytes("a.md", &bytes);

    set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &fx.guard("a.md")).unwrap();

    let after = fx.read_bytes("a.md");
    assert_eq!(&after[..3], &[0xEF, 0xBB, 0xBF], "BOM korunmalı");
    assert!(String::from_utf8_lossy(&after).contains("- [x] görev"));
}

#[test]
fn turkce_ve_emoji_basliklar() {
    let fx = Fixture::new("t8");
    fx.write("a.md", "- [ ] Öğrenciyiz için içerik 🌱 hazırla\n");

    set_task_status(
        fx.path(), "a.md", 1, "Öğrenciyiz için içerik 🌱 hazırla",
        TaskStatus::Done, &fx.guard("a.md"),
    )
    .unwrap();
    assert_eq!(fx.read("a.md"), "- [x] Öğrenciyiz için içerik 🌱 hazırla\n");
}

#[test]
fn baslik_uyusmazsa_yazilmaz() {
    let fx = Fixture::new("t9");
    let original = "- [ ] gerçek görev\n";
    fx.write("a.md", original);

    let err = set_task_status(
        fx.path(), "a.md", 1, "BAŞKA BAŞLIK", TaskStatus::Done, &fx.guard("a.md"),
    )
    .expect_err("uyuşmayan başlık reddedilmeli");

    assert_eq!(err.code(), "target_mismatch");
    assert_eq!(fx.read("a.md"), original, "dosya DEĞİŞMEMELİ");
}

#[test]
fn checkbox_olmayan_satira_yazilmaz() {
    let fx = Fixture::new("t10");
    let original = "# Sadece başlık\n";
    fx.write("a.md", original);

    let err = set_task_status(
        fx.path(), "a.md", 1, "Sadece başlık", TaskStatus::Done, &fx.guard("a.md"),
    )
    .expect_err("checkbox olmayan satır reddedilmeli");

    assert_eq!(err.code(), "target_mismatch");
    assert_eq!(fx.read("a.md"), original);
}

#[test]
fn olmayan_satir_reddedilir() {
    let fx = Fixture::new("t11");
    fx.write("a.md", "- [ ] tek satır\n");
    let err = set_task_status(
        fx.path(), "a.md", 99, "tek satır", TaskStatus::Done, &fx.guard("a.md"),
    )
    .expect_err("aralık dışı satır reddedilmeli");
    assert_eq!(err.code(), "target_mismatch");
}

// ===========================================================================
// ÇAKIŞMA — madde 20.2, 20.3
// ===========================================================================

#[test]
fn baska_surec_degistirmisse_cakisma() {
    let fx = Fixture::new("c1");
    fx.write("a.md", "- [ ] görev\n");
    let guard = fx.guard("a.md");

    // Obsidian/Hermes araya girdi.
    fx.write("a.md", "- [ ] görev\nEklenen satır\n");

    let err = set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &guard)
        .expect_err("çakışma dönmeli");

    assert_eq!(err.code(), "conflict");
    assert_eq!(fx.read("a.md"), "- [ ] görev\nEklenen satır\n", "üzerine YAZILMAMALI");
}

#[test]
fn mtime_ayni_hash_farkliysa_cakisma() {
    let fx = Fixture::new("c2");
    fx.write("a.md", "- [ ] görev\n");
    let mtime = mtime_ms(&fx.path().join("a.md"));

    let guard = WriteGuard {
        expected_hash: crate::vault::reader::hash("- [ ] görev\n".as_bytes()),
        expected_mtime_ms: Some(mtime),
    };

    fx.write("a.md", "- [ ] BAŞKA içerik\n");

    let err = set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &guard)
        .expect_err("hash farkı çakışma vermeli");
    assert_eq!(err.code(), "conflict");
}

#[test]
fn dosya_silinmisse_cakisma() {
    let fx = Fixture::new("c3");
    fx.write("a.md", "- [ ] görev\n");
    let guard = fx.guard("a.md");
    std::fs::remove_file(fx.path().join("a.md")).unwrap();

    let err = set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &guard)
        .expect_err("silinen dosya çakışma vermeli");
    assert_eq!(err.code(), "conflict");
}

#[test]
fn vault_disina_yazma_reddedilir() {
    let fx = Fixture::new("c4");
    fx.write("a.md", "- [ ] görev\n");
    let guard = fx.guard("a.md");

    for hostile in ["../kacis.md", "/etc/passwd", "", "alt/../../kacis.md"] {
        let err = set_task_status(fx.path(), hostile, 1, "görev", TaskStatus::Done, &guard)
            .expect_err("vault dışı yol reddedilmeli");
        assert!(
            matches!(err.code(), "path_outside_vault" | "conflict"),
            "beklenmeyen kod {} ({hostile})", err.code()
        );
    }
}

// ===========================================================================
// FRONTMATTER — madde 8.1
// ===========================================================================

#[test]
fn frontmatter_mevcut_key_guncellenir() {
    let fx = Fixture::new("f1");
    fx.write("a.md", "---\nworkspace: WIF\nstatus: açık\n---\n# Gövde\nmetin\n");

    set_frontmatter_field(
        fx.path(), "a.md", "status", Some(FieldValue::Text("kapandı".into())), &fx.guard("a.md"),
    )
    .unwrap();

    assert_eq!(
        fx.read("a.md"),
        "---\nworkspace: WIF\nstatus: kapandı\n---\n# Gövde\nmetin\n"
    );
}

#[test]
fn frontmatter_yeni_key_eklenir_sira_bozulmaz() {
    let fx = Fixture::new("f2");
    fx.write("a.md", "---\nworkspace: WIF\n---\ngövde\n");

    set_frontmatter_field(
        fx.path(), "a.md", "durum", Some(FieldValue::Text("aktif".into())), &fx.guard("a.md"),
    )
    .unwrap();
    assert_eq!(fx.read("a.md"), "---\nworkspace: WIF\ndurum: aktif\n---\ngövde\n");
}

#[test]
fn frontmatter_govdeye_dokunmaz() {
    let fx = Fixture::new("f3");
    let body = "# Başlık\n\n- [ ] görev\n\n```yaml\nworkspace: TUZAK\n```\n";
    fx.write("a.md", &format!("---\nworkspace: WIF\n---\n{body}"));

    set_frontmatter_field(
        fx.path(), "a.md", "workspace", Some(FieldValue::Text("GEN".into())), &fx.guard("a.md"),
    )
    .unwrap();

    let after = fx.read("a.md");
    assert!(after.starts_with("---\nworkspace: GEN\n---\n"));
    assert!(after.ends_with(body), "gövde bayt bayt korunmalı");
}

#[test]
fn frontmatter_tipleri() {
    let fx = Fixture::new("f4");
    fx.write("a.md", "---\nx: 1\n---\ngövde\n");

    set_frontmatter_field(fx.path(), "a.md", "b", Some(FieldValue::Bool(true)), &fx.guard("a.md")).unwrap();
    set_frontmatter_field(fx.path(), "a.md", "n", Some(FieldValue::Number(42.0)), &fx.guard("a.md")).unwrap();
    set_frontmatter_field(fx.path(), "a.md", "f", Some(FieldValue::Number(1.5)), &fx.guard("a.md")).unwrap();
    set_frontmatter_field(
        fx.path(), "a.md", "l",
        Some(FieldValue::List(vec!["a".into(), "b".into()])), &fx.guard("a.md"),
    )
    .unwrap();

    let after = fx.read("a.md");
    assert!(after.contains("b: true"), "{after}");
    assert!(after.contains("n: 42"), "{after}");
    assert!(after.contains("f: 1.5"), "{after}");
    assert!(after.contains("l:\n  - a\n  - b"), "{after}");
}

#[test]
fn frontmatter_liste_key_guncellenince_eski_blok_silinir() {
    let fx = Fixture::new("f5");
    fx.write("a.md", "---\ntags:\n  - eski1\n  - eski2\nworkspace: WIF\n---\ngövde\n");

    set_frontmatter_field(
        fx.path(), "a.md", "tags", Some(FieldValue::List(vec!["yeni".into()])), &fx.guard("a.md"),
    )
    .unwrap();

    assert_eq!(fx.read("a.md"), "---\ntags:\n  - yeni\nworkspace: WIF\n---\ngövde\n");
}

#[test]
fn frontmatter_key_kaldirilir() {
    let fx = Fixture::new("f6");
    fx.write("a.md", "---\na: 1\nb: 2\n---\ngövde\n");
    set_frontmatter_field(fx.path(), "a.md", "a", None, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "---\nb: 2\n---\ngövde\n");
}

#[test]
fn frontmatter_yoksa_reddedilir() {
    let fx = Fixture::new("f7");
    let original = "# Sadece gövde\n";
    fx.write("a.md", original);

    let err = set_frontmatter_field(
        fx.path(), "a.md", "x", Some(FieldValue::Text("y".into())), &fx.guard("a.md"),
    )
    .expect_err("frontmatter yoksa reddedilmeli");

    assert_eq!(err.code(), "no_frontmatter");
    assert_eq!(fx.read("a.md"), original);
}

#[test]
fn arkeles_id_yazmak_reddedilir() {
    let fx = Fixture::new("f8");
    let original = "---\narkeles_id: 01ARZ3NDEKTSV4RRFFQ69G5FAV\n---\ngövde\n";
    fx.write("a.md", original);

    let err = set_frontmatter_field(
        fx.path(), "a.md", "arkeles_id",
        Some(FieldValue::Text("01ARZ3NDEKTSV4RRFFQ69G5FB1".into())), &fx.guard("a.md"),
    )
    .expect_err("arkeles_id yazımı reddedilmeli");

    assert_eq!(err.code(), "semantic_refused");
    assert_eq!(fx.read("a.md"), original);
}

#[test]
fn frontmatter_crlf_korunur() {
    let fx = Fixture::new("f9");
    fx.write("a.md", "---\r\nworkspace: WIF\r\n---\r\ngövde\r\n");

    set_frontmatter_field(
        fx.path(), "a.md", "workspace", Some(FieldValue::Text("GEN".into())), &fx.guard("a.md"),
    )
    .unwrap();
    assert_eq!(fx.read("a.md"), "---\r\nworkspace: GEN\r\n---\r\ngövde\r\n");
}

#[test]
fn bozuk_yaml_yazilabilir_ama_bozmadan() {
    let fx = Fixture::new("f10");
    fx.write("a.md", "---\nfoo: [unclosed\nworkspace: WIF\n---\ngövde\n");

    set_frontmatter_field(
        fx.path(), "a.md", "workspace", Some(FieldValue::Text("GEN".into())), &fx.guard("a.md"),
    )
    .unwrap();

    let after = fx.read("a.md");
    assert!(after.contains("foo: [unclosed"), "bozuk satıra dokunulmamalı");
    assert!(after.contains("workspace: GEN"));
}

#[test]
fn yaml_ozel_degerler_tirnaklanir() {
    let fx = Fixture::new("f11");
    fx.write("a.md", "---\nx: 1\n---\ngövde\n");

    for (value, expect) in [
        ("true", "\"true\""),
        ("123", "\"123\""),
        ("- tire", "\"- tire\""),
        ("iki: nokta", "\"iki: nokta\""),
        ("normal metin", "normal metin"),
    ] {
        set_frontmatter_field(
            fx.path(), "a.md", "v", Some(FieldValue::Text(value.into())), &fx.guard("a.md"),
        )
        .unwrap();
        assert!(
            fx.read("a.md").contains(&format!("v: {expect}")),
            "{value} → {}", fx.read("a.md")
        );
    }
}

#[test]
fn buyuk_not_yazilabilir() {
    let fx = Fixture::new("f12");
    let filler = "Uzun bir satır, tekrar tekrar.\n".repeat(40_000); // ~1.2 MB
    fx.write("a.md", &format!("---\nworkspace: WIF\n---\n{filler}- [ ] görev\n"));

    let line = 4 + filler.lines().count() as u32;
    set_task_status(fx.path(), "a.md", line, "görev", TaskStatus::Done, &fx.guard("a.md"))
        .expect("büyük notta yazma çalışmalı");
    assert!(fx.read("a.md").contains("- [x] görev"));
}

// ===========================================================================
// ETİKET — madde 8.1
// ===========================================================================

#[test]
fn etiket_eklenir_ve_cikarilir() {
    let fx = Fixture::new("g1");
    fx.write("a.md", "---\nworkspace: WIF\n---\ngövde\n");

    toggle_tag(fx.path(), "a.md", "acil", true, &fx.guard("a.md")).unwrap();
    assert!(fx.read("a.md").contains("tags:\n  - acil"), "{}", fx.read("a.md"));

    toggle_tag(fx.path(), "a.md", "acil", false, &fx.guard("a.md")).unwrap();
    let after = fx.read("a.md");
    assert!(!after.contains("acil"), "{after}");
    assert!(!after.contains("tags:"), "boş liste yerine anahtar kalkmalı: {after}");
}

#[test]
fn ayni_etiket_iki_kez_eklenmez_ve_dosyaya_dokunulmaz() {
    let fx = Fixture::new("g2");
    fx.write("a.md", "---\ntags:\n  - acil\n---\ngövde\n");
    let before = fx.read("a.md");

    toggle_tag(fx.path(), "a.md", "acil", true, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), before, "değişiklik yoksa dosyaya dokunulmaz");
}

#[test]
fn tek_dizge_tags_bicimi_desteklenir() {
    let fx = Fixture::new("g3");
    fx.write("a.md", "---\ntags: mevcut\n---\ngövde\n");

    toggle_tag(fx.path(), "a.md", "yeni", true, &fx.guard("a.md")).unwrap();
    let after = fx.read("a.md");
    assert!(after.contains("- mevcut"), "{after}");
    assert!(after.contains("- yeni"), "{after}");
}

// ===========================================================================
// SATIR SIRASI — madde 8.1
// ===========================================================================

#[test]
fn satir_tasinir() {
    let fx = Fixture::new("m1");
    fx.write("a.md", "bir\niki\nüç\n");
    move_line(fx.path(), "a.md", 1, 3, &fx.guard("a.md")).unwrap();
    assert_eq!(fx.read("a.md"), "iki\nüç\nbir\n");
}

#[test]
fn satir_sonu_olmayan_satir_tasinmaz() {
    let fx = Fixture::new("m2");
    let original = "bir\niki";
    fx.write("a.md", original);

    let err = move_line(fx.path(), "a.md", 2, 1, &fx.guard("a.md"))
        .expect_err("son satır taşınmamalı");
    assert_eq!(err.code(), "target_mismatch");
    assert_eq!(fx.read("a.md"), original);
}

// ===========================================================================
// HIZLI YAKALAMA — madde 10
// ===========================================================================

#[test]
fn hizli_yakalama_ham_satir_ekler() {
    let fx = Fixture::new("q1");
    fx.write("Gelen Kutusu.md", "önceki satır\n");

    append_to_inbox(fx.path(), "Gelen Kutusu.md", "aklıma gelen şey").unwrap();
    // Madde 10.2: yapı kurmaz — madde işareti, başlık, frontmatter YOK.
    assert_eq!(fx.read("Gelen Kutusu.md"), "önceki satır\naklıma gelen şey\n");
}

#[test]
fn hizli_yakalama_dosya_yoksa_olusturmaz() {
    // Madde 10.2: "yeni dosya açmaz." Sınırın kendisi.
    let fx = Fixture::new("q2");

    let err = append_to_inbox(fx.path(), "Gelen Kutusu.md", "metin")
        .expect_err("dosya yoksa reddedilmeli");

    assert_eq!(err.code(), "inbox_missing");
    assert!(
        !fx.path().join("Gelen Kutusu.md").exists(),
        "ARKELÉS dosya OLUŞTURMAMALI"
    );
}

#[test]
fn hizli_yakalama_cok_satirli_metni_tek_satira_indirir() {
    let fx = Fixture::new("q3");
    fx.write("Gelen Kutusu.md", "");
    append_to_inbox(fx.path(), "Gelen Kutusu.md", "birinci\n\nikinci\nüçüncü").unwrap();
    assert_eq!(fx.read("Gelen Kutusu.md"), "birinci ikinci üçüncü\n");
}

#[test]
fn hizli_yakalama_satir_sonu_bicimini_korur() {
    let fx = Fixture::new("q4");
    fx.write("Gelen Kutusu.md", "önceki\r\n");
    append_to_inbox(fx.path(), "Gelen Kutusu.md", "yeni").unwrap();
    assert_eq!(fx.read("Gelen Kutusu.md"), "önceki\r\nyeni\r\n");
}

#[test]
fn hizli_yakalama_eksik_satir_sonunu_tamamlar() {
    let fx = Fixture::new("q5");
    fx.write("Gelen Kutusu.md", "sonu yok");
    append_to_inbox(fx.path(), "Gelen Kutusu.md", "yeni").unwrap();
    assert_eq!(fx.read("Gelen Kutusu.md"), "sonu yok\nyeni\n");
}

#[test]
fn hizli_yakalama_bos_metni_reddeder() {
    let fx = Fixture::new("q6");
    fx.write("Gelen Kutusu.md", "x\n");
    assert!(append_to_inbox(fx.path(), "Gelen Kutusu.md", "   ").is_err());
    assert_eq!(fx.read("Gelen Kutusu.md"), "x\n", "dosya değişmemeli");
}

#[test]
fn hizli_yakalama_turkce_ve_emoji() {
    let fx = Fixture::new("q7");
    fx.write("Gelen Kutusu.md", "");
    append_to_inbox(fx.path(), "Gelen Kutusu.md", "Öğrenciyiz için fikir 🌱").unwrap();
    assert_eq!(fx.read("Gelen Kutusu.md"), "Öğrenciyiz için fikir 🌱\n");
}

// ===========================================================================
// ATOMIC WRITE — madde 20.1
// ===========================================================================

#[test]
fn atomic_write_gecici_dosya_birakmaz() {
    let fx = Fixture::new("a1");
    fx.write("a.md", "- [ ] görev\n");

    set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &fx.guard("a.md")).unwrap();

    let leftovers: Vec<_> = std::fs::read_dir(fx.path())
        .unwrap()
        .filter_map(Result::ok)
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.contains("arkeles-tmp"))
        .collect();

    assert!(leftovers.is_empty(), "geçici dosya kalmamalı: {leftovers:?}");
}

#[cfg(unix)]
#[test]
fn atomic_write_basarisizliginda_orijinal_korunur() {
    use std::os::unix::fs::PermissionsExt;

    let fx = Fixture::new("a2");
    fx.write("alt/a.md", "- [ ] görev\n");
    let guard = fx.guard("alt/a.md");

    let dir = fx.path().join("alt");
    let mut perms = std::fs::metadata(&dir).unwrap().permissions();
    perms.set_mode(0o500); // r-x: yeni dosya yaratılamaz
    std::fs::set_permissions(&dir, perms).unwrap();

    let result = set_task_status(fx.path(), "alt/a.md", 1, "görev", TaskStatus::Done, &guard);

    // İzni geri ver ki fixture silinebilsin ve dosya okunabilsin.
    let mut perms = std::fs::metadata(&dir).unwrap().permissions();
    perms.set_mode(0o700);
    std::fs::set_permissions(&dir, perms).unwrap();

    let err = result.expect_err("yazma başarısız olmalı");
    assert_eq!(err.code(), "write_failed");
    assert_eq!(fx.read("alt/a.md"), "- [ ] görev\n", "orijinal korunmalı");
}

// ===========================================================================
// HASH DÖNGÜSÜ — madde 15.6, 9
// ===========================================================================

#[test]
fn yazma_sonrasi_hash_degisir_ayni_yazma_tekrarinda_ise_ayni_kalir() {
    /*
     * Bu, dosya izleyici döngüsünün temel taşı:
     *  - yazma yeni bir hash üretir → index güncellenmeli
     *  - aynı içerik tekrar yazılırsa hash AYNI kalır → izleyici no-op
     */
    let fx = Fixture::new("h1");
    fx.write("a.md", "- [ ] görev\n");
    let before = crate::vault::reader::hash(&fx.read_bytes("a.md"));

    let r1 = set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &fx.guard("a.md")).unwrap();
    assert_ne!(r1.new_hash, before, "yazma hash'i değiştirmeli");
    assert_eq!(
        r1.new_hash,
        crate::vault::reader::hash(&fx.read_bytes("a.md")),
        "dönen hash diskteki içerikle eşleşmeli"
    );

    // Zaten "done" olan görevi yine "done" yapmak: içerik değişmez.
    let r2 = set_task_status(fx.path(), "a.md", 1, "görev", TaskStatus::Done, &fx.guard("a.md")).unwrap();
    assert_eq!(r2.new_hash, r1.new_hash, "aynı içerik aynı hash");
}
