/*!
Hermes sağlık ve özet — Anayasa madde 18.

18.1  Hermes sürüm ve YETENEK listesi yayınlar.
18.2  ARKELÉS bildirilmeyen yeteneği arayüzde HİÇ göstermez.
18.4  Ulaşılamamak HATA DEĞİLDİR — `reachable: false` geçerli cevaptır.
*/

use rusqlite::Connection;

use crate::error::CoreResult;
use crate::hermes::{contract, http, jobs, rpc};
use crate::types::{HermesHealth, HermesSummary};

impl HermesHealth {
    /// Madde 18.3 tablosundaki varsayılan durum.
    pub fn unreachable() -> Self {
        Self { reachable: false, version: None, capabilities: Vec::new() }
    }
}

/// `GET /api/health` + `GET /api/status` — Anayasa madde 18.1.
///
/// HİÇ HATA DÖNDÜRMEZ. Madde 18.4: ulaşılamamak geçerli bir cevaptır;
/// hata döndürmek arayüzü kırmızı uyarıya zorlardı.
pub fn health() -> HermesHealth {
    // 1) Erişilebilirlik: /api/health kimlik gerektirmez.
    let Some(health) = http::get_json("/api/health") else {
        return HermesHealth::unreachable();
    };

    let version = health
        .get("version")
        .and_then(|v| v.as_str())
        .map(str::to_string);

    /*
     * 2) Yetenekler: /api/status kimlik İSTER. Token yoksa veya yanlışsa
     * burası `None` döner — Hermes AYAKTA ama ARKELÉS onunla konuşamıyor.
     *
     * Bu durumda `reachable: true` ama `capabilities: []` döneriz: kullanıcı
     * Hermes'in çalıştığını görür, semantik aksiyonlar görünmez (madde 18.2).
     */
    let mut capabilities = http::get_json("/api/status")
        .map(|status| contract::declared_capabilities(&status))
        .unwrap_or_default();

    // 3) Gateway'in kendi ilanı (Sprint 5, canlı doğrulandı). Token yoksa
    //    veya kanal açılamazsa boş — sağlık yine `reachable: true` kalır.
    if let Ok(flags) = rpc::call("gateway.capabilities", serde_json::json!({})) {
        for flag in contract::declared_flags(&flags) {
            if !capabilities.contains(&flag) {
                capabilities.push(flag);
            }
        }
    }

    HermesHealth { reachable: true, version, capabilities }
}

/// Dashboard özeti — Sprint 4 madde 4, 13.
pub fn summary(conn: &Connection, health: HermesHealth) -> CoreResult<HermesSummary> {
    let (running, queued, completed_today, failed_today) = jobs::counts(conn)?;

    Ok(HermesSummary {
        reachable: health.reachable,
        version: health.version,
        running,
        queued,
        completed_today,
        failed_today,
        last_completed: jobs::last_completed(conn)?,
    })
}

#[cfg(test)]
mod tests {
    /// Sprint 5: kullanıcının canlı bulgusu — çalışan 0.21.0 "ulaşılamıyor"
    /// görünüyordu. ARKELÉS'in TAM sağlık yolunu gerçek Hermes'e karşı koşar.
    ///
    ///   cargo test canli_saglik -- --ignored --nocapture
    #[test]
    #[ignore = "çalışan yerel Hermes gerektirir"]
    fn canli_saglik() {
        let health = super::health();
        eprintln!("CANLI SAĞLIK: {health:?}");
        assert!(health.reachable, "çalışan Hermes ulaşılamaz sınıflandı");
    }
}
