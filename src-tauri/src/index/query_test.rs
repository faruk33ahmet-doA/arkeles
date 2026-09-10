/*!
Index sorgu testleri — Anayasa madde 39.3.

"Zorunlu test kapsamı: Rust markdown parser'ı ve INDEX KATMANI.
 En riskli ve en sessiz bozulan yer burasıdır."

Bu testler bellek içi SQLite üzerinde çalışır; disk veya vault gerektirmez.
*/

use rusqlite::Connection;

use crate::index::query;
use crate::index::schema;

/// Şema uygulanmış boş bir bellek içi index.
fn memory_index() -> Connection {
    let conn = Connection::open_in_memory().expect("bellek içi db");
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    schema::apply(&conn).expect("şema");
    conn
}


/*
 * Sorgular artık (sonuç, satır_hatası_sayısı) döner — Sprint 1 borcu #4.
 * Testler sonuca bakar; hata sayısının 0 olduğunu da DOĞRULAR, çünkü
 * fixture verisi temiz ve hatalı satır çıkması bir gerileme olurdu.
 */
fn today_ok(conn: &Connection) -> crate::types::TodayView {
    let (view, errors) = query::today(conn).expect("today sorgusu");
    assert_eq!(errors, 0, "temiz fixture'da satır hatası olmamalı");
    view
}

fn dashboard_ok(conn: &Connection) -> crate::types::DashboardView {
    let (view, errors) = query::dashboard(conn).expect("dashboard sorgusu");
    assert_eq!(errors, 0, "temiz fixture'da satır hatası olmamalı");
    view
}

fn search_ok(conn: &Connection, q: &str) -> Vec<crate::types::SearchHit> {
    let (hits, errors) = query::search(conn, q, 20).expect("arama sorgusu");
    assert_eq!(errors, 0, "temiz fixture'da satır hatası olmamalı");
    hits
}

fn insert_note(conn: &Connection, id: &str, title: &str, body: &str, frontmatter: &str) {
    conn.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at, frontmatter)
         VALUES (?1, ?2, ?3, 1, 'hash', '2026-09-10T10:00:00Z', ?4)",
        rusqlite::params![id, format!("{id}.md"), title, frontmatter],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, ?3)",
        rusqlite::params![id, title, body],
    )
    .unwrap();
}

fn insert_task(conn: &Connection, id: &str, note_id: &str, title: &str, status: &str, due: Option<&str>, ws: Option<&str>) {
    conn.execute(
        "INSERT INTO tasks (id, note_id, line_number, title, status, workspace, due)
         VALUES (?1, ?2, 1, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, note_id, title, status, ws, due],
    )
    .unwrap();
}

#[test]
fn arama_baslikta_eslesir() {
    let conn = memory_index();
    insert_note(&conn, "n1", "TüGA Sponsorluk", "gövde metni", "{}");

    let hits = search_ok(&conn, "sponsor");
    assert_eq!(hits.len(), 1, "önek araması başlıkta eşleşmeli");
    assert_eq!(hits[0].title, "TüGA Sponsorluk");
}

#[test]
fn arama_govdede_eslesir_ve_snippet_dondurur() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Başlık", "burada sponsorluk dosyası geçiyor", "{}");

    let hits = search_ok(&conn, "sponsorluk");
    assert_eq!(hits.len(), 1);
    assert!(!hits[0].snippet.is_empty(), "snippet üretilmeli");
}

#[test]
fn arama_bos_sorguda_bos_doner() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Bir şey", "gövde", "{}");
    assert!(search_ok(&conn, "").is_empty());
    assert!(search_ok(&conn, "   ").is_empty());
}

#[test]
fn arama_fts_operatorlerini_metin_olarak_ele_alir() {
    // Kullanıcı FTS operatörü yazarsa sorgu ÇÖKMEMELİ (enjeksiyon yok).
    let conn = memory_index();
    insert_note(&conn, "n1", "Normal not", "gövde", "{}");

    for hostile in ["OR", "AND", "NEAR(a b)", "\"", "a*b", "(", ")", "-", "^"] {
        let result = query::search(&conn, hostile, 20);
        assert!(result.is_ok(), "sorgu çökmemeli: {hostile}");
        assert_eq!(result.unwrap().1, 0, "satır hatası olmamalı: {hostile}");
    }
}

#[test]
fn arama_turkce_karakterlerde_calisir() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Öğrenciyiz Planı", "içerik", "{}");

    let hits = search_ok(&conn, "öğrenci");
    assert_eq!(hits.len(), 1, "Türkçe önek araması eşleşmeli");
}

#[test]
fn bugun_gorunumu_geciken_ve_bugunu_ayirir() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Not", "gövde", "{}");

    let today = crate::vault::time::today_iso();
    insert_task(&conn, "t1", "n1", "geciken", "open", Some("2020-01-01"), Some("WIF"));
    insert_task(&conn, "t2", "n1", "bugün", "open", Some(&today), Some("WIF"));
    insert_task(&conn, "t3", "n1", "gelecek", "open", Some("2099-01-01"), Some("WIF"));

    let view = today_ok(&conn);
    assert_eq!(view.overdue.len(), 1);
    assert_eq!(view.overdue[0].title, "geciken");
    assert_eq!(view.due.len(), 1);
    assert_eq!(view.due[0].title, "bugün");
    // Gelecek görev hiçbir grupta olmamalı.
}

#[test]
fn bugun_gorunumu_bitmis_gecikeni_saymaz() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Not", "gövde", "{}");
    insert_task(&conn, "t1", "n1", "bitmiş geciken", "done", Some("2020-01-01"), None);

    let view = today_ok(&conn);
    assert!(view.overdue.is_empty(), "bitmiş görev geciken sayılmaz");
}

#[test]
fn panel_calisma_alanlarini_sayar() {
    let conn = memory_index();
    insert_note(&conn, "n1", "Not", "gövde", "{}");
    insert_task(&conn, "t1", "n1", "a", "open", None, Some("WIF"));
    insert_task(&conn, "t2", "n1", "b", "open", None, Some("WIF"));
    insert_task(&conn, "t3", "n1", "c", "open", None, Some("TüGA"));
    insert_task(&conn, "t4", "n1", "d", "done", None, Some("TüGA"));

    let view = dashboard_ok(&conn);
    let wif = view.workspaces.iter().find(|w| w.name == "WIF").unwrap();
    let tuga = view.workspaces.iter().find(|w| w.name == "TüGA").unwrap();
    assert_eq!(wif.open_tasks, 2);
    assert_eq!(tuga.open_tasks, 1, "bitmiş görev sayılmaz");
}

/*
 * Hayat skoru — Anayasa madde 11.1/11.2.
 * ARKELÉS HESAPLAMAZ, yalnız Hermes'in vault'a yazdığını OKUR.
 */
#[test]
fn hayat_skoru_hermesin_yazdigi_nottan_okunur() {
    let conn = memory_index();
    insert_note(
        &conn,
        "n1",
        "skor",
        "gövde",
        r#"{"arkeles_type":"life_score","value":72,"computed_at":"2026-09-10T06:00:00Z"}"#,
    );

    let view = dashboard_ok(&conn);
    let score = view.life_score.expect("skor okunmalı");
    assert_eq!(score.value, 72.0);
    assert_eq!(score.computed_at, "2026-09-10T06:00:00Z");
}

#[test]
fn hayat_skoru_yoksa_none_doner() {
    let conn = memory_index();
    insert_note(&conn, "n1", "sıradan not", "gövde", "{}");
    assert!(dashboard_ok(&conn).life_score.is_none());
}

#[test]
fn hayat_skoru_bozuksa_none_doner() {
    // ARKELÉS düzeltmeye ÇALIŞMAZ (madde 8.2): eksik/bozuk skor yok sayılır.
    let conn = memory_index();
    insert_note(
        &conn,
        "n1",
        "skor",
        "gövde",
        r#"{"arkeles_type":"life_score","value":"yetmiş iki"}"#,
    );
    assert!(dashboard_ok(&conn).life_score.is_none());
}

#[test]
fn gorev_yonetilmeyen_notu_isaretler() {
    // Madde 16.3: yönetilmeyen notun görevine mekanik mutasyon uygulanmaz.
    let conn = memory_index();
    conn.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at)
         VALUES ('p1', 'a.md', 'Yönetilmeyen', 0, 'h', '2026-09-10T10:00:00Z')",
        [],
    )
    .unwrap();
    let today = crate::vault::time::today_iso();
    insert_task(&conn, "t1", "p1", "görev", "open", Some(&today), None);

    let view = today_ok(&conn);
    assert_eq!(view.due.len(), 1);
    assert!(!view.due[0].managed, "yönetilmeyen olarak işaretlenmeli");
}

#[test]
fn silinen_not_gorevlerini_de_dusurur() {
    // ON DELETE CASCADE — index tutarlılığı.
    let conn = memory_index();
    insert_note(&conn, "n1", "Not", "gövde", "{}");
    insert_task(&conn, "t1", "n1", "görev", "open", None, None);

    conn.execute("DELETE FROM notes WHERE id='n1'", []).unwrap();
    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM tasks", [], |r| r.get(0))
        .unwrap();
    assert_eq!(count, 0);
}

#[test]
fn not_sayimi_dogru() {
    let conn = memory_index();
    assert_eq!(query::note_count(&conn).unwrap(), 0);
    insert_note(&conn, "n1", "a", "b", "{}");
    insert_note(&conn, "n2", "c", "d", "{}");
    assert_eq!(query::note_count(&conn).unwrap(), 2);
}
