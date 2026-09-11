/*!
İş kuyruğu testleri — Sprint 4 madde 20.

Gerçek Hermes'e BAĞLANMAZ: bellek içi SQLite üzerinde defterin davranışı
test edilir. Hermes'ten gelen cevaplar `apply_remote_status`'a doğrudan
JSON olarak verilir — mock Hermes'in yerini bu tutar ve ağ gerektirmez.
*/

use rusqlite::Connection;
use serde_json::json;

use super::*;
use crate::index::schema;

fn db() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    conn.pragma_update(None, "foreign_keys", true).unwrap();
    schema::apply(&conn).unwrap();
    conn
}

fn submit_one(conn: &Connection) -> Job {
    submit(conn, "task.execute", Some("wif"), "Bütçe raporunu hazırla").unwrap()
}

// ===========================================================================
// GÖNDERME VE ALLOWLIST — madde 19
// ===========================================================================

#[test]
fn is_kuyruga_queued_olarak_girer() {
    let conn = db();
    let job = submit_one(&conn);

    assert_eq!(job.status, "queued");
    assert_eq!(job.action, "task.execute");
    assert_eq!(job.action_label, "Hermes'e görev ver");
    assert_eq!(job.workspace.as_deref(), Some("wif"));
    assert_eq!(job.source, "arkeles");
    assert!(job.started_at.is_none());
    assert!(job.finished_at.is_none());
    // Madde 7: Hermes bildirmeden progress YOK.
    assert!(job.progress.is_none());
}

#[test]
fn allowlist_disi_aksiyon_reddedilir() {
    // Sprint 4 madde 19: "keyfi action adı çalıştırılamıyor."
    let conn = db();
    for hostile in [
        "shell.exec",
        "rm -rf /",
        "task.execute; DROP TABLE jobs",
        "../../etc/passwd",
        "",
        "TASK.EXECUTE",
    ] {
        let err = submit(&conn, hostile, Some("wif"), "metin").unwrap_err();
        assert_eq!(err.code(), "action_not_allowed", "kabul edildi: {hostile}");
    }
    // Hiçbiri deftere yazılmamalı.
    let count: i64 = conn.query_row("SELECT COUNT(*) FROM jobs", [], |r| r.get(0)).unwrap();
    assert_eq!(count, 0);
}

#[test]
fn bos_girdi_reddedilir() {
    let conn = db();
    assert!(submit(&conn, "task.execute", None, "   ").is_err());
    assert!(submit(&conn, "task.execute", None, "").is_err());
}

#[test]
fn uzun_girdi_ozete_kirpilir() {
    // Defterin işi başlık tutmak; metin Hermes'e gider, burada saklanmaz.
    let conn = db();
    let long = "x".repeat(500);
    let job = submit(&conn, "task.execute", None, &long).unwrap();
    assert!(job.summary.chars().count() <= 120);
}

#[test]
fn workspace_ayrimi_korunur() {
    let conn = db();
    submit(&conn, "task.execute", Some("wif"), "wif işi").unwrap();
    submit(&conn, "task.execute", Some("gen"), "gen işi").unwrap();

    let today = crate::vault::time::today_iso();
    let wif = history(&conn, &today, Some("wif"), None).unwrap();
    assert_eq!(wif.len(), 1);
    assert_eq!(wif[0].summary, "wif işi");

    let gen = history(&conn, &today, Some("gen"), None).unwrap();
    assert_eq!(gen.len(), 1);
    assert_eq!(gen[0].summary, "gen işi");
}

// ===========================================================================
// HERMES DURUM BİLDİRİMLERİ — madde 3, 7
// ===========================================================================

#[test]
fn hermes_running_bildirince_defter_gunellenir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "running"})).unwrap();

    let updated = read_one(&conn, &job.id).unwrap();
    assert_eq!(updated.status, "running");
    assert!(updated.finished_at.is_none());
}

#[test]
fn tum_gecerli_durumlar_kabul_edilir() {
    let conn = db();
    for status in ["queued", "running", "completed", "failed", "cancelled"] {
        let job = submit(&conn, "task.execute", None, status).unwrap();
        apply_remote_status(&conn, &job.id, &json!({"status": status})).unwrap();
        assert_eq!(read_one(&conn, &job.id).unwrap().status, status);
    }
}

#[test]
fn uydurma_durum_deftere_girmez() {
    // ARKELÉS Hermes'in bildirmediği bir durumu ASLA yazmaz.
    let conn = db();
    let job = submit_one(&conn);

    for bogus in ["thinking", "almost_done", "", "COMPLETED", "42"] {
        apply_remote_status(&conn, &job.id, &json!({"status": bogus})).unwrap();
        assert_eq!(
            read_one(&conn, &job.id).unwrap().status,
            "queued",
            "uydurma durum kabul edildi: {bogus}"
        );
    }
}

#[test]
fn bitmis_isler_finished_at_alir() {
    let conn = db();
    for status in ["completed", "failed", "cancelled"] {
        let job = submit(&conn, "task.execute", None, status).unwrap();
        apply_remote_status(&conn, &job.id, &json!({"status": status})).unwrap();
        assert!(
            read_one(&conn, &job.id).unwrap().finished_at.is_some(),
            "{status} için bitiş zamanı yok"
        );
    }
}

// --- PROGRESS: sahte yüzde YASAK (madde 7) ---

#[test]
fn hermes_progress_verirse_gosterilir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "running", "progress": 42})).unwrap();
    assert_eq!(read_one(&conn, &job.id).unwrap().progress, Some(42));
}

#[test]
fn hermes_progress_vermezse_none_kalir() {
    // Madde 7: "%37, %82 gibi uydurma yüzdeler YASAK."
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "running"})).unwrap();
    assert!(read_one(&conn, &job.id).unwrap().progress.is_none());
}

#[test]
fn gecersiz_progress_yok_sayilir() {
    let conn = db();
    let job = submit_one(&conn);
    for bogus in [json!(-5), json!(150), json!("yarısı"), json!(null)] {
        apply_remote_status(&conn, &job.id, &json!({"status": "running", "progress": bogus}))
            .unwrap();
        assert!(
            read_one(&conn, &job.id).unwrap().progress.is_none(),
            "geçersiz progress kabul edildi: {bogus}"
        );
    }
}

// ===========================================================================
// HATA DOKTRİNİ — madde 15
// ===========================================================================

#[test]
fn failed_is_hata_mesaji_tasir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(
        &conn,
        &job.id,
        &json!({"status": "failed", "error": "Model yanıt vermedi"}),
    )
    .unwrap();

    let updated = read_one(&conn, &job.id).unwrap();
    assert_eq!(updated.status, "failed");
    assert_eq!(updated.error_code.as_deref(), Some("hermes_failed"));
    assert_eq!(updated.error_message.as_deref(), Some("Model yanıt vermedi"));
}

#[test]
fn uzun_hata_mesaji_kirpilir_stack_trace_sizmaz() {
    // Madde 19: "raw stack trace olarak gösterilmiyor."
    let conn = db();
    let job = submit_one(&conn);
    let trace = "Traceback (most recent call last):\n".repeat(50);
    apply_remote_status(&conn, &job.id, &json!({"status": "failed", "error": trace})).unwrap();

    let message = read_one(&conn, &job.id).unwrap().error_message.unwrap();
    assert!(message.chars().count() <= 200, "mesaj kırpılmadı");
}

#[test]
fn retry_yalniz_failed_isi_kuyruga_alir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "failed", "error": "hata"})).unwrap();

    retry(&conn, &job.id).unwrap();
    let after = read_one(&conn, &job.id).unwrap();
    assert_eq!(after.status, "queued");
    assert!(after.error_code.is_none(), "hata temizlenmeli");
    assert!(after.finished_at.is_none());
}

#[test]
fn retry_tamamlanmis_isi_bozmaz() {
    // Otomatik sonsuz retry YASAK; tamamlanmış iş yeniden çalıştırılmaz.
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "completed"})).unwrap();

    retry(&conn, &job.id).unwrap();
    assert_eq!(read_one(&conn, &job.id).unwrap().status, "completed");
}

// ===========================================================================
// İPTAL — madde 16
// ===========================================================================

#[test]
fn henuz_gonderilmemis_is_yerel_iptal_edilir() {
    let conn = db();
    let job = submit_one(&conn);
    cancel(&conn, &job.id).unwrap();

    let after = read_one(&conn, &job.id).unwrap();
    assert_eq!(after.status, "cancelled");
    assert!(after.finished_at.is_some());
}

// ===========================================================================
// ÇIKTILAR — madde 8
// ===========================================================================

#[test]
fn not_ciktisi_saklanir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(
        &conn,
        &job.id,
        &json!({
            "status": "completed",
            "outputs": [{"kind": "note", "ref": "01ARZ3NDEKTSV4RRFFQ69G5FAV", "label": "WIF Raporu"}]
        }),
    )
    .unwrap();

    let outputs = read_one(&conn, &job.id).unwrap().outputs;
    assert_eq!(outputs.len(), 1);
    assert_eq!(outputs[0].kind, "note");
    assert_eq!(outputs[0].label, "WIF Raporu");
}

#[test]
fn belge_ciktisi_saklanir() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(
        &conn,
        &job.id,
        &json!({
            "status": "completed",
            "outputs": [{"kind": "document", "ref": "raporlar/wif.pdf", "label": "wif.pdf"}]
        }),
    )
    .unwrap();

    let outputs = read_one(&conn, &job.id).unwrap().outputs;
    assert_eq!(outputs[0].kind, "document");
    assert_eq!(outputs[0].reference, "raporlar/wif.pdf");
}

#[test]
fn taninmayan_cikti_turu_saklanmaz() {
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(
        &conn,
        &job.id,
        &json!({
            "status": "completed",
            "outputs": [
                {"kind": "hologram", "ref": "x", "label": "y"},
                {"kind": "note", "ref": "ok", "label": "Gerçek"}
            ]
        }),
    )
    .unwrap();

    let outputs = read_one(&conn, &job.id).unwrap().outputs;
    assert_eq!(outputs.len(), 1, "yalnız tanınan tür saklanmalı");
    assert_eq!(outputs[0].kind, "note");
}

#[test]
fn cikti_yoksa_uydurulmaz() {
    // Madde 8: "Sonuç yoksa sonuç uydurma."
    let conn = db();
    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "completed"})).unwrap();
    assert!(read_one(&conn, &job.id).unwrap().outputs.is_empty());
}

#[test]
fn ciktilar_yeniden_bildirilince_cogaltilmaz() {
    let conn = db();
    let job = submit_one(&conn);
    let payload = json!({
        "status": "running",
        "outputs": [{"kind": "note", "ref": "a", "label": "A"}]
    });
    apply_remote_status(&conn, &job.id, &payload).unwrap();
    apply_remote_status(&conn, &job.id, &payload).unwrap();

    assert_eq!(read_one(&conn, &job.id).unwrap().outputs.len(), 1);
}

// ===========================================================================
// BOZUK CEVAPLAR — madde 20
// ===========================================================================

#[test]
fn bozuk_cevap_defteri_bozmaz() {
    let conn = db();
    let job = submit_one(&conn);

    for malformed in [
        json!(null),
        json!("metin"),
        json!(42),
        json!([]),
        json!({"beklenmeyen": "alan"}),
        json!({"status": null}),
        json!({"outputs": "dizi değil"}),
    ] {
        // Panik ATMAMALI ve durumu bozmamalı.
        apply_remote_status(&conn, &job.id, &malformed).unwrap();
        assert_eq!(read_one(&conn, &job.id).unwrap().status, "queued");
    }
}

// ===========================================================================
// SAYIMLAR VE LİSTELER — madde 4, 13, 14
// ===========================================================================

#[test]
fn sayimlar_dogru() {
    let conn = db();
    let a = submit(&conn, "task.execute", None, "a").unwrap();
    let b = submit(&conn, "task.execute", None, "b").unwrap();
    let c = submit(&conn, "task.execute", None, "c").unwrap();
    submit(&conn, "task.execute", None, "d").unwrap(); // queued kalır

    apply_remote_status(&conn, &a.id, &json!({"status": "running"})).unwrap();
    apply_remote_status(&conn, &b.id, &json!({"status": "completed"})).unwrap();
    apply_remote_status(&conn, &c.id, &json!({"status": "failed", "error": "x"})).unwrap();

    let (running, queued, completed, failed) = counts(&conn).unwrap();
    assert_eq!((running, queued, completed, failed), (1, 1, 1, 1));
}

#[test]
fn aktif_liste_yalniz_queued_ve_running() {
    let conn = db();
    let a = submit(&conn, "task.execute", None, "a").unwrap();
    let b = submit(&conn, "task.execute", None, "b").unwrap();
    submit(&conn, "task.execute", None, "c").unwrap();

    apply_remote_status(&conn, &a.id, &json!({"status": "running"})).unwrap();
    apply_remote_status(&conn, &b.id, &json!({"status": "completed"})).unwrap();

    let active_jobs = active(&conn).unwrap();
    assert_eq!(active_jobs.len(), 2);
    assert!(active_jobs.iter().all(|j| j.status != "completed"));
}

#[test]
fn gecmis_duruma_gore_suzulur() {
    let conn = db();
    let a = submit(&conn, "task.execute", None, "a").unwrap();
    submit(&conn, "task.execute", None, "b").unwrap();
    apply_remote_status(&conn, &a.id, &json!({"status": "failed", "error": "x"})).unwrap();

    let today = crate::vault::time::today_iso();
    let failed_jobs = history(&conn, &today, None, Some("failed")).unwrap();
    assert_eq!(failed_jobs.len(), 1);
    assert_eq!(failed_jobs[0].summary, "a");
}

#[test]
fn son_tamamlanan_is_bulunur() {
    let conn = db();
    assert!(last_completed(&conn).unwrap().is_none());

    let job = submit_one(&conn);
    apply_remote_status(&conn, &job.id, &json!({"status": "completed"})).unwrap();
    assert_eq!(last_completed(&conn).unwrap().unwrap().id, job.id);
}

#[test]
fn kuyruk_ve_calisan_kimlikleri() {
    let conn = db();
    let a = submit(&conn, "task.execute", None, "a").unwrap();
    submit(&conn, "task.execute", None, "b").unwrap();
    apply_remote_status(&conn, &a.id, &json!({"status": "running"})).unwrap();

    assert_eq!(queued_ids(&conn).unwrap().len(), 1);
    assert_eq!(running_ids(&conn).unwrap(), vec![a.id]);
}

// ===========================================================================
// KAYNAK — madde 17 (Telegram)
// ===========================================================================

#[test]
fn arkeles_kaynakli_isler_isaretlenir() {
    let conn = db();
    assert_eq!(submit_one(&conn).source, "arkeles");
}

#[test]
fn telegram_kaynakli_is_kaynagini_korur() {
    // Hermes Telegram'dan aldığı bir işi yayınlarsa kaynağı GÖSTERİLİR.
    // Hermes bunu desteklemiyorsa bu satır hiç oluşmaz — veri uydurulmaz.
    let conn = db();
    conn.execute(
        "INSERT INTO jobs (id, action, summary, status, created_at, source)
         VALUES ('t1', 'task.execute', 'Telegram işi', 'running', ?1, 'telegram')",
        rusqlite::params![crate::vault::time::now_iso()],
    )
    .unwrap();

    assert_eq!(read_one(&conn, "t1").unwrap().source, "telegram");
}

#[test]
fn olmayan_is_hata_doner() {
    let conn = db();
    assert!(read_one(&conn, "yok").is_err());
}
