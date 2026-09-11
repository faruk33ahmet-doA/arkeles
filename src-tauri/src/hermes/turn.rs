/*!
Hermes turu — bir ARKELÉS isteğinin Hermes'teki GERÇEK yaşam döngüsü.
Sprint 5: yerel Hermes 0.21.0 üzerinde canlı doğrulandı.

  1. `session.create {title}`             → `{session_id, …}`
  2. `prompt.submit {session_id, text}`   → `{"status":"streaming"}` — tur başladı
  3. aynı sokette `message.complete` olayı → `payload.status`:
        complete | error | interrupted
  İptal: `session.interrupt {session_id}` → Hermes turu keser ve
  `message.complete (status=interrupted)` yayınlar.

Sprint 4 varsayımları YANLIŞTI ve kaldırıldı:
  - `command.dispatch` gerçek ama eğik çizgi komutları içindir (/goal, /loop)
  - `session.status` ve `session.cancel` diye metod YOK

Bu dosya Hermes'e özgü metod ve olay adlarının TEK yeridir: defter (jobs),
arayüz ve alan modeli bu adları bilmez; yalnız `Outcome` görür.
*/

use std::time::Duration;

use serde_json::{json, Value};

use crate::error::{CoreError, CoreResult};
use crate::hermes::rpc::{self, Channel};

/// Bir turun en uzun süresi. Aşılırsa iş "zaman aşımı" ile kapanır —
/// sonsuza kadar "çalışıyor" görünmez.
pub const TURN_TIMEOUT: Duration = Duration::from_secs(20 * 60);

/// Hermes'in bildirdiği tur sonucu — alan modeline çıkan TEK biçim.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Completed,
    /// Hermes'in kısa hata metni ve explicit `error_surface` sınıfı.
    Failed {
        code: &'static str,
        message: String,
    },
    Cancelled,
}

#[cfg(test)]
impl Outcome {
    /// Yapılandırılmış hata sınıfı bildirmeyen eski/bozuk Hermes cevabı.
    pub fn failed(message: impl Into<String>) -> Self {
        Self::Failed {
            code: "hermes_failed",
            message: message.into(),
        }
    }
}

/// Turu baştan sona yürütür. BLOKLAYICI — çağıran kendi iş parçacığındadır.
///
/// `on_started` Hermes istemi kabul ettiğinde oturum kimliğiyle çağrılır.
/// `false` dönerse (kullanıcı bu arada iptal etti) tur Hermes'te de kesilir.
pub fn run(title: &str, prompt: &str, on_started: impl FnOnce(&str) -> bool) -> CoreResult<Outcome> {
    let mut channel = Channel::open()?;
    run_on(&mut channel, title, prompt, TURN_TIMEOUT, on_started)
}

pub(crate) fn run_on(
    channel: &mut Channel,
    title: &str,
    prompt: &str,
    turn_timeout: Duration,
    on_started: impl FnOnce(&str) -> bool,
) -> CoreResult<Outcome> {
    let session = channel.call("session.create", json!({ "title": title }))?;
    let session_id = session
        .get("session_id")
        .and_then(Value::as_str)
        .filter(|id| !id.is_empty())
        .ok_or(CoreError::HermesMalformed)?
        .to_string();

    // Kullanıcı metni PARAMETRE olarak gider; hiçbir komut satırına girmez.
    let submitted = channel.call(
        "prompt.submit",
        json!({ "session_id": session_id, "text": prompt }),
    )?;
    if !submitted.is_object() {
        return Err(CoreError::HermesMalformed);
    }

    if !on_started(&session_id) {
        let _ = channel.call("session.interrupt", json!({ "session_id": session_id }));
        return Ok(Outcome::Cancelled);
    }

    let payload = channel.wait_event(&session_id, "message.complete", turn_timeout)?;
    outcome_of(&payload)
}

/// `message.complete` yükü → sonuç. Tanınmayan durum UYDURULMAZ.
pub(crate) fn outcome_of(payload: &Value) -> CoreResult<Outcome> {
    match payload.get("status").and_then(Value::as_str) {
        Some("complete") => Ok(Outcome::Completed),
        Some("interrupted") => Ok(Outcome::Cancelled),
        Some("error") => Ok(Outcome::Failed {
            code: failure_code(payload),
            message: rpc::short_text(payload.get("text"), "Hermes işi tamamlayamadı."),
        }),
        _ => Err(CoreError::HermesMalformed),
    }
}

/// Hermes 0.21.0 `message.complete.error_surface.layer` alanını açıkça
/// yayınlar. Yalnız tanımlı katmanlar eşlenir; hata metninden tür tahmini yok.
fn failure_code(payload: &Value) -> &'static str {
    match payload
        .get("error_surface")
        .and_then(|surface| surface.get("layer"))
        .and_then(Value::as_str)
    {
        Some("billing") => "provider_billing",
        Some("auth") => "provider_auth",
        Some("provider") => "provider_failed",
        Some("endpoint") => "provider_endpoint",
        Some("streaming") => "provider_streaming",
        Some("disk") => "host_disk_full",
        _ => "hermes_failed",
    }
}

/// Yürüyen turu keser. Sonucu (cancelled) tur kanalı bildirir.
pub fn interrupt(session_id: &str) -> CoreResult<()> {
    rpc::call("session.interrupt", json!({ "session_id": session_id })).map(|_| ())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hermes::fake;

    const TOKEN: &str = "dogru-token";

    fn open(port: u16) -> Channel {
        Channel::open_at(port, TOKEN, Duration::from_millis(400)).unwrap()
    }

    /// Gerçek Hermes'in kabul akışı: oturum aç, istemi kabul et.
    fn accept_turn(ws: &mut fake::Socket, session_id: &str) -> Value {
        let create = fake::recv(ws);
        assert_eq!(create["method"], "session.create");
        fake::reply(ws, &create, json!({ "session_id": session_id, "message_count": 0 }));
        let submit = fake::recv(ws);
        assert_eq!(submit["method"], "prompt.submit");
        assert_eq!(submit["params"]["session_id"], session_id);
        submit
    }

    #[test]
    fn tam_tur_tamamlanir() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s1");
            assert!(submit["params"]["text"].as_str().unwrap().contains("rapor"));
            fake::event(ws, "s1", "message.delta", json!({ "text": "…" }));
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
            // Başka oturumun olayı YOK SAYILIR.
            fake::event(ws, "baska", "message.complete", json!({ "status": "error" }));
            fake::event(ws, "s1", "message.complete", json!({ "status": "complete", "text": "tamam" }));
        });

        let mut started = None;
        let outcome = run_on(&mut open(server.port), "başlık", "rapor hazırla", Duration::from_secs(5), |id| {
            started = Some(id.to_string());
            true
        })
        .unwrap();

        assert_eq!(outcome, Outcome::Completed);
        assert_eq!(started.as_deref(), Some("s1"));
        server.finish();
    }

    #[test]
    fn tamamlanma_kabulden_once_gelse_de_kaybolmaz() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s2");
            fake::event(ws, "s2", "message.complete", json!({ "status": "complete" }));
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
        });
        let outcome = run_on(&mut open(server.port), "t", "p", Duration::from_secs(5), |_| true).unwrap();
        assert_eq!(outcome, Outcome::Completed);
        server.finish();
    }

    #[test]
    fn hermes_hatasi_failed_tek_satir() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s3");
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
            let text = format!("Error: model yanıt vermedi\n{}", "  at frame\n".repeat(30));
            fake::event(ws, "s3", "message.complete", json!({ "status": "error", "text": text }));
        });
        let outcome = run_on(&mut open(server.port), "t", "p", Duration::from_secs(5), |_| true).unwrap();
        assert_eq!(outcome, Outcome::failed("Error: model yanıt vermedi"));
        server.finish();
    }

    #[test]
    fn provider_hatasi_explicit_error_surface_ile_siniflanir() {
        let outcome = outcome_of(&json!({
            "status": "error",
            "text": "Kredi yetersiz",
            "error_surface": {"layer": "billing", "code": "billing", "retryable": false}
        }))
        .unwrap();
        assert_eq!(
            outcome,
            Outcome::Failed {
                code: "provider_billing",
                message: "Kredi yetersiz".into()
            }
        );
    }

    #[test]
    fn bozuk_error_surface_metin_tahmini_yapmaz() {
        let outcome = outcome_of(&json!({
            "status": "error",
            "text": "HTTP 402 credits",
            "error_surface": {"layer": 42}
        }))
        .unwrap();
        assert_eq!(outcome, Outcome::failed("HTTP 402 credits"));
    }

    #[test]
    fn kesilen_tur_iptal_olur() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s4");
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
            fake::event(ws, "s4", "message.complete", json!({ "status": "interrupted" }));
        });
        let outcome = run_on(&mut open(server.port), "t", "p", Duration::from_secs(5), |_| true).unwrap();
        assert_eq!(outcome, Outcome::Cancelled);
        server.finish();
    }

    #[test]
    fn baslamadan_iptal_edilen_is_hermeste_de_kesilir() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s5");
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
            let interrupt = fake::recv(ws);
            assert_eq!(interrupt["method"], "session.interrupt");
            assert_eq!(interrupt["params"]["session_id"], "s5");
            fake::reply(ws, &interrupt, json!({ "status": "interrupted" }));
        });
        let outcome = run_on(&mut open(server.port), "t", "p", Duration::from_secs(5), |_| false).unwrap();
        assert_eq!(outcome, Outcome::Cancelled);
        server.finish();
    }

    #[test]
    fn oturum_kimligi_yoksa_malformed() {
        let server = fake::serve(TOKEN, |ws| {
            let create = fake::recv(ws);
            fake::reply(ws, &create, json!({ "message_count": 0 }));
        });
        let err = run_on(&mut open(server.port), "t", "p", Duration::from_secs(5), |_| true).unwrap_err();
        assert_eq!(err.code(), "hermes_malformed_response");
        server.finish();
    }

    #[test]
    fn tamamlanma_gelmezse_timeout() {
        let server = fake::serve(TOKEN, |ws| {
            let submit = accept_turn(ws, "s6");
            fake::reply(ws, &submit, json!({ "status": "streaming" }));
            std::thread::sleep(Duration::from_millis(900));
        });
        let err = run_on(&mut open(server.port), "t", "p", Duration::from_millis(500), |_| true).unwrap_err();
        assert_eq!(err.code(), "hermes_timeout");
        server.finish();
    }

    #[test]
    fn tanimsiz_tamamlanma_durumu_uydurulmaz() {
        for payload in [json!({}), json!({ "status": "almost" }), json!(null), json!("complete")] {
            assert!(outcome_of(&payload).is_err(), "kabul edildi: {payload}");
        }
    }
}
