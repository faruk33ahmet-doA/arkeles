/*!
İş modülü sorgu testleri — Sprint 3 madde 16.

39.5: gerçek vault üzerinde test YOK. Bellek içi SQLite fixture.

Bu testlerin çoğu bir SIZINTI testidir: WIF verisinin GEN ekranına
düşmediğini, workspace alanı olmayan notun hiçbir kuruma düşmediğini
doğrular. Yanlış kuruma düşen veri, sessizce yanlış karar aldırır.
*/

use rusqlite::Connection;

use crate::index::{schema, work};

fn memory_index() -> Connection {
    let conn = Connection::open_in_memory().expect("bellek içi db");
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    schema::apply(&conn).expect("şema");
    conn
}

#[allow(clippy::too_many_arguments)]
fn note(
    conn: &Connection,
    id: &str,
    title: &str,
    workspace: Option<&str>,
    kind: &str,
    managed: bool,
    modified: &str,
    frontmatter: &str,
) {
    conn.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at,
                            frontmatter, workspace, kind)
         VALUES (?1, ?2, ?3, ?4, 'h', ?5, ?6, ?7, ?8)",
        rusqlite::params![
            id,
            format!("{id}.md"),
            title,
            managed as i64,
            modified,
            frontmatter,
            workspace,
            kind
        ],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notes_fts (note_id, title, body) VALUES (?1, ?2, 'gövde')",
        rusqlite::params![id, title],
    )
    .unwrap();
}

fn task(conn: &Connection, id: &str, note_id: &str, title: &str, status: &str, ws: Option<&str>) {
    conn.execute(
        "INSERT INTO tasks (id, note_id, line_number, title, status, workspace)
         VALUES (?1, ?2, 1, ?3, ?4, ?5)",
        rusqlite::params![id, note_id, title, status, ws],
    )
    .unwrap();
}

// ===========================================================================
// SIZINTI TESTLERİ — Sprint 3 madde 16
// ===========================================================================

#[test]
fn wif_filtresi_yalniz_wif_verisini_getirir() {
    let conn = memory_index();
    note(&conn, "n1", "WIF Notu", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "n2", "GEN Notu", Some("gen"), "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "wif görevi", "open", Some("wif"));
    task(&conn, "t2", "n2", "gen görevi", "open", Some("gen"));

    let (tasks, _) = work::tasks(&conn, "wif").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "wif görevi");

    let (notes, _) = work::notes(&conn, "wif", "note").unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].title, "WIF Notu");
}

#[test]
fn gen_filtresi_yalniz_gen_verisini_getirir() {
    let conn = memory_index();
    note(&conn, "n1", "WIF Notu", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "n2", "GEN Notu", Some("gen"), "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "wif görevi", "open", Some("wif"));
    task(&conn, "t2", "n2", "gen görevi", "open", Some("gen"));

    let (tasks, _) = work::tasks(&conn, "gen").unwrap();
    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].title, "gen görevi");
}

#[test]
fn wif_verisi_gen_ekranina_sizmaz() {
    let conn = memory_index();
    for i in 0..20 {
        note(&conn, &format!("w{i}"), "WIF", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
        task(&conn, &format!("wt{i}"), &format!("w{i}"), "wif", "open", Some("wif"));
    }
    note(&conn, "g1", "GEN", Some("gen"), "note", true, "2026-09-10T10:00:00Z", "{}");

    let (gen_tasks, _) = work::tasks(&conn, "gen").unwrap();
    assert!(gen_tasks.is_empty(), "GEN'de WIF görevi görünmemeli");

    let (gen_notes, _) = work::notes(&conn, "gen", "note").unwrap();
    assert_eq!(gen_notes.len(), 1);

    let gen = work::overview(&conn, "gen").unwrap().0;
    assert_eq!(gen.active_tasks, 0);
    assert_eq!(gen.done_tasks, 0);
}

#[test]
fn workspace_alani_olmayan_not_hicbir_kuruma_dusmez() {
    let conn = memory_index();
    note(&conn, "n1", "Sahipsiz", None, "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "sahipsiz görev", "open", None);

    for ws in ["wif", "gen", "tuga", "burkon", "merci", "kepder", "ogrenciyiz"] {
        assert!(work::tasks(&conn, ws).unwrap().0.is_empty(), "{ws} kirlendi");
        assert!(work::notes(&conn, ws, "note").unwrap().0.is_empty(), "{ws} kirlendi");
    }
}

// ===========================================================================
// ÇALIŞMA ALANI LİSTESİ — Sprint 3 madde 2
// ===========================================================================

#[test]
fn yedi_kurum_da_listelenir() {
    let conn = memory_index();
    let list = work::list_workspaces(&conn).unwrap();
    assert_eq!(list.len(), 7, "madde 36.1: dock ve İş ekranı hepsini gösterir");

    let active: Vec<&str> = list.iter().filter(|w| w.active).map(|w| w.id.as_str()).collect();
    assert_eq!(active, vec!["wif", "gen"]);
}

#[test]
fn kurum_sayimlari_dogru() {
    let conn = memory_index();
    note(&conn, "n1", "A", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "n2", "B", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "a", "open", Some("wif"));
    task(&conn, "t2", "n1", "b", "in_progress", Some("wif"));
    task(&conn, "t3", "n1", "c", "deferred", Some("wif"));
    task(&conn, "t4", "n1", "d", "done", Some("wif"));

    let wif = work::list_workspaces(&conn)
        .unwrap()
        .into_iter()
        .find(|w| w.id == "wif")
        .unwrap();

    // Bitmemiş = açık + sürüyor + bekliyor. Bitmiş sayılmaz.
    assert_eq!(wif.outstanding_tasks, 3);
    assert_eq!(wif.note_count, 2);
}

// ===========================================================================
// GENEL BAKIŞ — Sprint 3 madde 3
// ===========================================================================

#[test]
fn genel_bakis_gorev_durumlarini_ayirir() {
    let conn = memory_index();
    note(&conn, "n1", "A", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "açık", "open", Some("wif"));
    task(&conn, "t2", "n1", "sürüyor", "in_progress", Some("wif"));
    task(&conn, "t3", "n1", "bekliyor", "deferred", Some("wif"));
    task(&conn, "t4", "n1", "bitti", "done", Some("wif"));
    task(&conn, "t5", "n1", "iptal", "cancelled", Some("wif"));

    let view = work::overview(&conn, "wif").unwrap().0;
    assert_eq!(view.active_tasks, 2, "açık + sürüyor");
    assert_eq!(view.waiting_tasks, 1, "bekliyor");
    assert_eq!(view.done_tasks, 1, "iptal, bitmiş sayılmaz");
}

/*
 * Öncelik — Anayasa madde 11.2.
 * ARKELÉS öncelik ANALİZİ YAPMAZ. Hermes yazdıysa okur, yoksa gizler.
 */
#[test]
fn oncelik_hermes_yazdiysa_okunur() {
    let conn = memory_index();
    note(
        &conn, "p1", "Öncelik", Some("wif"), "note", true, "2026-09-10T10:00:00Z",
        r#"{"arkeles_type":"priority","value":"Bütçe onayını bitir"}"#,
    );

    let view = work::overview(&conn, "wif").unwrap().0;
    assert_eq!(view.priority.as_deref(), Some("Bütçe onayını bitir"));
}

#[test]
fn oncelik_yoksa_none_doner_ve_uydurulmaz() {
    let conn = memory_index();
    note(&conn, "n1", "Sıradan", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "n1", "acil görünen bir görev", "open", Some("wif"));

    let view = work::overview(&conn, "wif").unwrap().0;
    assert!(view.priority.is_none(), "ARKELÉS öncelik ÜRETMEZ (madde 11.2)");
}

#[test]
fn oncelik_baska_kurumdan_sizmaz() {
    let conn = memory_index();
    note(
        &conn, "p1", "GEN Önceliği", Some("gen"), "note", true, "2026-09-10T10:00:00Z",
        r#"{"arkeles_type":"priority","value":"GEN işi"}"#,
    );
    assert!(work::overview(&conn, "wif").unwrap().0.priority.is_none());
}

#[test]
fn son_aktivite_en_yeni_not_zamanidir() {
    let conn = memory_index();
    note(&conn, "n1", "Eski", Some("wif"), "note", true, "2026-09-01T10:00:00Z", "{}");
    note(&conn, "n2", "Yeni", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");

    let view = work::overview(&conn, "wif").unwrap().0;
    assert_eq!(view.last_activity.as_deref(), Some("2026-09-10T10:00:00Z"));
}

#[test]
fn bos_calisma_alani_duzgun_doner() {
    // Sprint 3 madde 16: "boş çalışma alanı düzgün render ediliyor."
    let conn = memory_index();
    let view = work::overview(&conn, "burkon").unwrap().0;

    assert_eq!(view.active_tasks, 0);
    assert_eq!(view.waiting_tasks, 0);
    assert_eq!(view.done_tasks, 0);
    assert!(view.last_activity.is_none());
    assert!(view.priority.is_none());
    assert!(view.recent_notes.is_empty());
    assert!(view.next_meeting.is_none());
}

// ===========================================================================
// TOPLANTILAR — Sprint 3 madde 9
// ===========================================================================

#[test]
fn yaklasan_toplanti_gecmisi_atlar() {
    let conn = memory_index();
    let today = crate::vault::time::today_iso();

    conn.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at,
                            workspace, kind, event_date)
         VALUES ('m1','m1.md','Geçmiş',1,'h','2026-01-01T00:00:00Z','wif','meeting','2020-01-01')",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO notes (id, source_path, title, managed, content_hash, modified_at,
                            workspace, kind, event_date)
         VALUES ('m2','m2.md','Gelecek',1,'h','2026-01-01T00:00:00Z','wif','meeting','2099-01-01')",
        [],
    )
    .unwrap();
    let _ = today;

    let view = work::overview(&conn, "wif").unwrap().0;
    assert_eq!(view.next_meeting.map(|m| m.title).as_deref(), Some("Gelecek"));

    // Liste ise HEPSİNİ döner — geçmiş toplantı da bir kayıttır.
    let (all, _) = work::meetings(&conn, "wif").unwrap();
    assert_eq!(all.len(), 2);
}

#[test]
fn tarihsiz_toplanti_listelenmez() {
    // Veri uydurulmaz: tarihi olmayan toplantı takvimde yeri olmayan nottur.
    let conn = memory_index();
    note(&conn, "m1", "Tarihsiz", Some("wif"), "meeting", true, "2026-09-10T10:00:00Z", "{}");
    assert!(work::meetings(&conn, "wif").unwrap().0.is_empty());
}

// ===========================================================================
// PROJELER — Sprint 3 madde 7
// ===========================================================================

#[test]
fn proje_yalniz_frontmatter_soyluyorsa_projedir() {
    let conn = memory_index();
    note(&conn, "n1", "WIF Projesi diye geçen not", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "p1", "Gerçek proje", Some("wif"), "project", true, "2026-09-10T10:00:00Z", "{}");

    let (projects, _) = work::notes(&conn, "wif", "project").unwrap();
    assert_eq!(projects.len(), 1, "başlıkta 'proje' geçmesi onu proje YAPMAZ");
    assert_eq!(projects[0].title, "Gerçek proje");
}

// ===========================================================================
// NOT DETAYI VE İLİŞKİLER — Sprint 3 madde 10, 11
// ===========================================================================

#[test]
fn not_detayi_govde_ve_gorevleri_getirir() {
    let conn = memory_index();
    note(&conn, "n1", "Detay", Some("wif"), "note", true, "2026-09-10T10:00:00Z", r#"{"a":1}"#);
    task(&conn, "t1", "n1", "görev", "open", Some("wif"));

    let detail = work::note_detail(&conn, "n1").unwrap();
    assert_eq!(detail.title, "Detay");
    assert!(detail.managed);
    assert_eq!(detail.body, "gövde");
    assert_eq!(detail.tasks.len(), 1);
    assert!(detail.frontmatter.contains("\"a\""));
}

#[test]
fn iliskiler_giden_ve_gelen_olarak_ayrilir() {
    let conn = memory_index();
    note(&conn, "n1", "Kaynak", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "n2", "Hedef", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(&conn, "n3", "Başka", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");

    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n1','Hedef')", []).unwrap();
    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n3','Hedef')", []).unwrap();

    let hedef = work::note_detail(&conn, "n2").unwrap();
    assert!(hedef.outgoing.is_empty());
    let incoming: Vec<&str> = hedef.incoming.iter().map(|l| l.title.as_str()).collect();
    assert_eq!(incoming, vec!["Başka", "Kaynak"]);

    let kaynak = work::note_detail(&conn, "n1").unwrap();
    assert_eq!(kaynak.outgoing.len(), 1);
    assert_eq!(kaynak.outgoing[0].title, "Hedef");
    assert_eq!(kaynak.outgoing[0].note_id.as_deref(), Some("n2"));
}

#[test]
fn kirik_baglanti_da_gosterilir() {
    // Obsidian'da kırık bağlantı bir BİLGİDİR; gizlemek veri saklamaktır.
    let conn = memory_index();
    note(&conn, "n1", "Kaynak", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n1','Olmayan Not')", []).unwrap();

    let detail = work::note_detail(&conn, "n1").unwrap();
    assert_eq!(detail.outgoing.len(), 1);
    assert_eq!(detail.outgoing[0].title, "Olmayan Not");
    assert!(detail.outgoing[0].note_id.is_none(), "hedef yoksa kimlik boş");
}

#[test]
fn not_kendine_gelen_baglanti_gostermez() {
    let conn = memory_index();
    note(&conn, "n1", "Kendine", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n1','Kendine')", []).unwrap();

    let detail = work::note_detail(&conn, "n1").unwrap();
    assert!(detail.incoming.is_empty(), "kendine bağlantı 'gelen' değildir");
}

#[test]
fn yonetilmeyen_not_okunur_ama_isaretlenir() {
    // Sprint 3 madde 16: arkeles_id olmayan not OKUNUR, mekanik yazma YAPAMAZ.
    let conn = memory_index();
    note(&conn, "path:123", "Yönetilmeyen", Some("wif"), "note", false, "2026-09-10T10:00:00Z", "{}");
    task(&conn, "t1", "path:123", "görev", "open", Some("wif"));

    let detail = work::note_detail(&conn, "path:123").unwrap();
    assert_eq!(detail.title, "Yönetilmeyen");
    assert!(!detail.managed, "yazma yetkisi YOK");
    assert!(!detail.tasks[0].managed);
}

#[test]
fn olmayan_not_detayi_hata_doner() {
    let conn = memory_index();
    assert!(work::note_detail(&conn, "yok").is_err());
}

// ===========================================================================
// BELGELER — Sprint 3 madde 8
// ===========================================================================

fn document(conn: &Connection, id: &str, path: &str, name: &str, ext: &str) {
    conn.execute(
        "INSERT INTO documents (id, source_path, file_name, extension, size_bytes, modified_at)
         VALUES (?1, ?2, ?3, ?4, 1024, '2026-09-10T10:00:00Z')",
        rusqlite::params![id, path, name, ext],
    )
    .unwrap();
}

#[test]
fn belge_baglanan_notun_kurumuna_ait_olur() {
    let conn = memory_index();
    note(&conn, "n1", "WIF Notu", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    document(&conn, "d1", "ekler/butce.pdf", "butce.pdf", "pdf");
    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n1','butce.pdf')", []).unwrap();

    let (docs, _) = work::documents(&conn, "wif").unwrap();
    assert_eq!(docs.len(), 1);
    assert_eq!(docs[0].file_name, "butce.pdf");

    assert!(work::documents(&conn, "gen").unwrap().0.is_empty(), "GEN'e sızmamalı");
}

#[test]
fn hicbir_nottan_baglanmayan_belge_hicbir_kuruma_dusmez() {
    let conn = memory_index();
    note(&conn, "n1", "WIF Notu", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    document(&conn, "d1", "ekler/yetim.pdf", "yetim.pdf", "pdf");

    for ws in ["wif", "gen", "tuga"] {
        assert!(work::documents(&conn, ws).unwrap().0.is_empty(), "{ws}");
    }
}

#[test]
fn belge_uzantisiz_adla_da_baglanabilir() {
    // Obsidian'da [[butce]] yazıp butce.pdf'e bağlanmak yaygındır.
    let conn = memory_index();
    note(&conn, "n1", "WIF", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    document(&conn, "d1", "ekler/butce.pdf", "butce.pdf", "pdf");
    conn.execute("INSERT INTO links (source_id, target_ref) VALUES ('n1','butce')", []).unwrap();

    assert_eq!(work::documents(&conn, "wif").unwrap().0.len(), 1);
}

// ===========================================================================
// BÜYÜK LİSTE — Sprint 3 madde 14
// ===========================================================================

#[test]
fn iki_yuzden_fazla_kayit_tam_doner() {
    let conn = memory_index();
    note(&conn, "n1", "Kaynak", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    for i in 0..250 {
        task(&conn, &format!("t{i}"), "n1", &format!("görev {i}"), "open", Some("wif"));
    }

    let (tasks, errors) = work::tasks(&conn, "wif").unwrap();
    assert_eq!(tasks.len(), 250, "sorgu kırpmaz; sanallaştırma ARAYÜZ işidir");
    assert_eq!(errors, 0);
}

// ===========================================================================
// MAKİNE KAYITLARI — Hermes'in yazdıkları insan listelerinde görünmez
// ===========================================================================

#[test]
fn hermes_makine_kayitlari_not_listesinde_gorunmez() {
    /*
     * Hermes bir çalışma alanına öncelik yazdığında o bir NOT dosyasıdır,
     * ama kullanıcının "Notlar" panelinde yeri yoktur — orada gürültüdür.
     * Öncelik DEĞERİ yine okunur ve Genel Bakış'ta görünür.
     */
    let conn = memory_index();
    note(&conn, "n1", "Gerçek not", Some("wif"), "note", true, "2026-09-10T10:00:00Z", "{}");
    note(
        &conn, "s1", "oncelik", Some("wif"), "system", true, "2026-09-10T11:00:00Z",
        r#"{"arkeles_type":"priority","value":"Bütçeyi bitir"}"#,
    );

    let (notes, _) = work::notes(&conn, "wif", "note").unwrap();
    assert_eq!(notes.len(), 1, "makine kaydı listelenmemeli");
    assert_eq!(notes[0].title, "Gerçek not");

    let view = work::overview(&conn, "wif").unwrap().0;
    assert_eq!(view.recent_notes.len(), 1, "son notlarda da görünmemeli");
    // Ama DEĞERİ okunur:
    assert_eq!(view.priority.as_deref(), Some("Bütçeyi bitir"));
}

#[test]
fn makine_kaydi_kurum_sayimina_dahil_ama_listeye_degil() {
    // Sayım "kaç not var" sorusunun cevabıdır; gizlemek yanlış olurdu.
    let conn = memory_index();
    note(&conn, "s1", "skor", Some("wif"), "system", true, "2026-09-10T10:00:00Z", "{}");

    let wif = work::list_workspaces(&conn)
        .unwrap()
        .into_iter()
        .find(|w| w.id == "wif")
        .unwrap();
    assert_eq!(wif.note_count, 1);
}
