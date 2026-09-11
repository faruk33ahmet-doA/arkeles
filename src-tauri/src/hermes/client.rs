/*!
Hermes sağlık ve özet — Anayasa madde 18.

18.1  Hermes sürüm ve YETENEK listesi yayınlar.
18.2  ARKELÉS bildirilmeyen yeteneği arayüzde HİÇ göstermez.
18.4  Ulaşılamamak HATA DEĞİLDİR — `reachable: false` geçerli cevaptır.
*/

use rusqlite::Connection;

use crate::error::CoreResult;
use crate::hermes::{contract, http, jobs};
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
    let capabilities = http::get_json("/api/status")
        .map(|status| contract::derive_capabilities(&status))
        .unwrap_or_default();

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
