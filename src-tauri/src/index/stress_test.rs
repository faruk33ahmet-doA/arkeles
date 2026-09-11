/*!
2.000 notluk yük testi — Sprint 5 madde 10, Anayasa madde 34.1.

Bütçeler (34.1):
  Tam index kurulumu (2.000 not)      < 1 sn
  Bugün görünümü sorgusu (2.000 not)  < 10 ms

Gerçek vault'a DOKUNMAZ (madde 39.5): sentetik vault geçici dizinde üretilir
ve test sonunda silinir. Üretim ile AYNI yol kullanılır: dosya tabanlı SQLite,
WAL, gerçek ayrıştırıcı ve `builder::full_scan`.

Sayılar yalnız release derlemede anlamlıdır; bütçe iddiası da yalnız orada
yapılır:

  cargo test --release stres_2000 -- --ignored --nocapture
*/

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use rusqlite::Connection;

use super::{builder, query, schema, work};

const NOTES: usize = 2000;
const DOCUMENTS: usize = 150;
const WORKSPACES: &[&str] = &["WIF", "GEN", "TüGA", "Burkon", "Merci", "KEPDER", "Öğrenciyiz"];

struct Vault(PathBuf);

impl Drop for Vault {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn generate(root: &Path) {
    let today = crate::vault::time::today_iso();
    std::fs::create_dir_all(root.join("raporlar")).unwrap();

    for i in 0..NOTES {
        let workspace = WORKSPACES[i % WORKSPACES.len()];
        let folder = root.join(format!("{workspace}/{}", i % 12));
        std::fs::create_dir_all(&folder).unwrap();

        let (kind, event) = if i % 5 == 0 {
            ("meeting", format!("event_date: 2026-09-{:02}\n", 1 + i % 28))
        } else {
            ("note", String::new())
        };
        // ~%10 yönetilmeyen not: gerçek vault'ta arkeles_id taşımayan notlar olur.
        let id_line = if i % 10 == 9 { String::new() } else { format!("arkeles_id: 01J{i:023}\n") };
        let due_today = if i % 7 == 0 { format!(" 📅 {today}") } else { String::new() };
        let overdue = if i % 11 == 0 { " 📅 2026-01-15" } else { "" };

        let body = format!(
            "---\n{id_line}workspace: {workspace}\nkind: {kind}\n{event}tags: [proje, etiket-{t}]\n---\n\
             # Not {i}\n\n\
             Bu paragraf {workspace} için sentetik bir içeriktir. Karar, risk ve takip \
             maddeleri burada anlatılır; [[Not {prev}]] ve [[rapor-{doc}.pdf]] ile ilişkilidir.\n\n\
             ## Görevler\n\
             - [ ] Taslak hazırla{due_today}\n\
             - [ ] Geri bildirim topla{overdue}\n\
             - [x] Toplantı notunu paylaş\n\
             - [ ] Sonraki adım due:: 2026-12-01\n\n\
             > Alıntı: sentetik veri.\n\n{filler}\n",
            t = i % 10,
            prev = i.saturating_sub(1),
            doc = i % DOCUMENTS,
            filler = "Ek bağlam cümlesi. ".repeat(40),
        );
        std::fs::write(folder.join(format!("Not {i}.md")), body).unwrap();
    }

    for d in 0..DOCUMENTS {
        std::fs::write(root.join(format!("raporlar/rapor-{d}.pdf")), b"%PDF-1.4 sentetik").unwrap();
    }
}

fn open_like_production(path: &Path) -> Connection {
    let conn = Connection::open(path).unwrap();
    conn.pragma_update(None, "journal_mode", "WAL").unwrap();
    conn.pragma_update(None, "synchronous", "NORMAL").unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    schema::apply(&conn).unwrap();
    conn
}

/// Aynı sorguyu tekrar tekrar koşar: (medyan, p95).
fn sample(runs: usize, mut f: impl FnMut()) -> (Duration, Duration) {
    f(); // ısınma: hazırlanmış sorgu önbelleği
    let mut times: Vec<Duration> = (0..runs)
        .map(|_| {
            let start = Instant::now();
            f();
            start.elapsed()
        })
        .collect();
    times.sort();
    (times[runs / 2], times[(runs * 95) / 100])
}

fn ms(d: Duration) -> String {
    format!("{:.2} ms", d.as_secs_f64() * 1000.0)
}

#[test]
#[ignore = "yük testi — release derlemede elle koşulur"]
fn stres_2000_not() {
    let base = std::env::temp_dir().join(format!("arkeles-stres-{}", std::process::id()));
    let vault = Vault(base.join("vault"));
    std::fs::create_dir_all(&vault.0).unwrap();
    generate(&vault.0);

    let db_path = base.join("index.sqlite3");
    let mut conn = open_like_production(&db_path);

    // --- Tam index kurulumu (soğuk: boş index) ---
    let start = Instant::now();
    let report = builder::full_scan(&mut conn, &vault.0).unwrap();
    let cold = start.elapsed();

    // --- Tekrar tarama (sıcak: dosyalar değişmedi) ---
    let start = Instant::now();
    builder::full_scan(&mut conn, &vault.0).unwrap();
    let warm = start.elapsed();

    let count = |sql: &str| -> i64 { conn.query_row(sql, [], |r| r.get(0)).unwrap() };
    let notes = count("SELECT COUNT(*) FROM notes");
    let tasks = count("SELECT COUNT(*) FROM tasks");
    let links = count("SELECT COUNT(*) FROM links");
    let documents = count("SELECT COUNT(*) FROM documents");
    let unmanaged = count("SELECT COUNT(*) FROM notes WHERE managed = 0");

    // --- Sorgular ---
    let (today_med, today_p95) = sample(200, || {
        query::today(&conn).unwrap();
    });
    let (dash_med, dash_p95) = sample(100, || {
        query::dashboard(&conn).unwrap();
    });
    let (ws_med, _) = sample(100, || {
        work::list_workspaces(&conn).unwrap();
    });
    let (ov_med, _) = sample(100, || {
        work::overview(&conn, "wif").unwrap();
    });
    let (tasks_med, _) = sample(100, || {
        work::tasks(&conn, "wif").unwrap();
    });
    let (notes_med, _) = sample(100, || {
        work::notes(&conn, "wif", "note").unwrap();
    });
    let (search_med, search_p95) = sample(100, || {
        query::search(&conn, "karar", 20).unwrap();
    });
    let detail_id: String = conn
        .query_row("SELECT id FROM notes WHERE title = 'Not 1000'", [], |r| r.get(0))
        .unwrap();
    let (detail_med, _) = sample(100, || {
        work::note_detail(&conn, &detail_id).unwrap();
    });

    // --- Artımlı güncelleme: tek dosya değişti (izleyici yolu) ---
    let changed = vault.0.join("WIF/0/Not 0.md");
    let mut text = std::fs::read_to_string(&changed).unwrap();
    text.push_str("\n- [ ] Yeni görev\n");
    std::fs::write(&changed, text).unwrap();
    let start = Instant::now();
    builder::sync_one(&mut conn, &vault.0, &changed).unwrap();
    let incremental = start.elapsed();

    let db_bytes = std::fs::metadata(&db_path).map(|m| m.len()).unwrap_or(0)
        + std::fs::metadata(base.join("index.sqlite3-wal")).map(|m| m.len()).unwrap_or(0);

    let release = !cfg!(debug_assertions);
    eprintln!("\n===== ARKELÉS 2.000 NOT YÜK TESTİ ({}) =====", if release { "release" } else { "debug" });
    eprintln!("veri          : {notes} not ({unmanaged} yönetilmeyen), {tasks} görev, {links} bağlantı, {documents} belge");
    eprintln!("satır hatası  : {}", report.row_errors);
    eprintln!("index boyutu  : {:.1} MB", db_bytes as f64 / 1_048_576.0);
    eprintln!("tam kurulum   : {}   (bütçe < 1000 ms)", ms(cold));
    eprintln!("tekrar tarama : {}", ms(warm));
    eprintln!("artımlı (1)   : {}", ms(incremental));
    eprintln!("bugün         : medyan {}  p95 {}   (bütçe < 10 ms)", ms(today_med), ms(today_p95));
    eprintln!("dashboard     : medyan {}  p95 {}", ms(dash_med), ms(dash_p95));
    eprintln!("kurum listesi : medyan {}", ms(ws_med));
    eprintln!("genel bakış   : medyan {}", ms(ov_med));
    eprintln!("kurum görevleri: medyan {}", ms(tasks_med));
    eprintln!("kurum notları : medyan {}", ms(notes_med));
    eprintln!("FTS arama     : medyan {}  p95 {}", ms(search_med), ms(search_p95));
    eprintln!("not detayı    : medyan {}", ms(detail_med));
    eprintln!("==============================================\n");

    assert_eq!(notes as usize, NOTES, "her not index'e girmeli");
    assert_eq!(report.row_errors, 0);
    if release {
        assert!(cold < Duration::from_millis(1000), "tam kurulum bütçesi aşıldı: {}", ms(cold));
        assert!(today_p95 < Duration::from_millis(10), "bugün bütçesi aşıldı: {}", ms(today_p95));
    }

    drop(conn);
    let _ = std::fs::remove_dir_all(&base);
}

/*
 * GERİLEME — Sprint 5 kurulum hızlandırması.
 *
 * Yeni notlarda türetilmiş satır silme ATLANIYOR; var olan (düzenlenmiş)
 * notta ise eski görev, bağlantı ve FTS kaydı KALMAMALI.
 */
#[test]
fn duzenlenen_notta_eski_turetilmis_satir_kalmaz() {
    let base = std::env::temp_dir().join(format!("arkeles-reindex-{}", std::process::id()));
    let vault = Vault(base.join("vault"));
    std::fs::create_dir_all(&vault.0).unwrap();
    let note = vault.0.join("a.md");
    std::fs::write(
        &note,
        "---\narkeles_id: 01J00000000000000000000001\n---\n# A\neskikelime [[Eski]]\n- [ ] eski görev\n- [ ] ikinci\n",
    )
    .unwrap();

    let mut conn = open_like_production(&base.join("index.sqlite3"));
    builder::full_scan(&mut conn, &vault.0).unwrap();

    std::fs::write(
        &note,
        "---\narkeles_id: 01J00000000000000000000001\n---\n# A\nyenikelime [[Yeni]]\n- [ ] yeni görev\n",
    )
    .unwrap();
    builder::sync_one(&mut conn, &vault.0, &note).unwrap();

    assert_eq!(count(&conn, "SELECT COUNT(*) FROM notes"), 1);
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM tasks"), 1, "eski görev kaldı");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM links WHERE target_ref = 'Eski'"), 0, "eski bağlantı kaldı");
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM notes_fts"), 1, "FTS kaydı çoğaldı");
    assert!(query::search(&conn, "eskikelime", 5).unwrap().0.is_empty(), "eski metin aranabiliyor");
    assert_eq!(query::search(&conn, "yenikelime", 5).unwrap().0.len(), 1);

    // Tam tarama da (değişmemiş dosya) hiçbir şeyi çoğaltmaz.
    builder::full_scan(&mut conn, &vault.0).unwrap();
    assert_eq!(count(&conn, "SELECT COUNT(*) FROM notes_fts"), 1);

    drop(conn);
    let _ = std::fs::remove_dir_all(&base);
}

fn count(conn: &Connection, sql: &str) -> i64 {
    conn.query_row(sql, [], |r| r.get(0)).unwrap()
}
