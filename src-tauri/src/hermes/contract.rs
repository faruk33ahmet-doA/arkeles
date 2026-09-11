/*!
Hermes sözleşmesi — Sprint 4.

BU DOSYA HERMES'İN GERÇEK API'SİNİ TARİF EDER, HEDEFLENEN BİRİNİ DEĞİL.
Yerel kurulum (`~/.hermes/hermes-agent`) incelenerek çıkarıldı:

  Taşıma   : `hermes serve` → 127.0.0.1:9119 (varsayılan)
  Kimlik   : `X-Hermes-Session-Token: <token>` başlığı
             (eski yol: `Authorization: Bearer <token>`)
  Token    : `HERMES_DASHBOARD_SESSION_TOKEN` ortam değişkeni; verilmezse
             Hermes her açılışta yenisini üretir ve yalnız kendi SPA'sına verir
  REST     : GET /api/health      → erişilebilirlik (kimlik gerektirmez)
             GET /api/status      → bileşen sağlığı, yapılandırılmış platformlar
             GET /api/sessions    → ajan oturumları = İŞLER
  JSON-RPC : ws://127.0.0.1:9119/api/ws?token=<token>  (JSON-RPC 2.0)
             `gateway.ping` ve `*.­*` biçiminde metodlar

ANAYASA UYUMU:
  19.1  Yalnız 127.0.0.1'e bağlanılır.
  19.3  Token JS'e HİÇ inmez — bu modül yalnız Rust'ta çalışır.
  18.1  Yetenekler Hermes'in BİLDİRDİĞİNDEN türetilir, varsayılmaz.
  18.4  Ulaşılamamak HATA DEĞİLDİR.

SPRINT 1 DÜZELTMESİ: o sprintte port 7717 ve düz `GET /health` varsayılmıştı.
Gerçek sözleşme farklı çıktı; bu dosya onu değiştirir.
*/

use std::path::PathBuf;

/// Hermes `serve` varsayılan portu (`hermes serve --help`).
pub const DEFAULT_PORT: u16 = 9119;

/// Hermes'in beklediği kimlik başlığı.
pub const SESSION_HEADER: &str = "X-Hermes-Session-Token";

/// Bağlantı ayarları. Ortam değişkenleriyle geçersiz kılınabilir.
pub struct Endpoint {
    pub port: u16,
}

impl Endpoint {
    pub fn resolve() -> Self {
        let port = std::env::var("ARKELES_HERMES_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_PORT);
        Self { port }
    }

    pub fn http_authority(&self) -> String {
        format!("127.0.0.1:{}", self.port)
    }
}

/*
 * Token kaynağı — Anayasa madde 19.2.
 *
 * İki yer denenir:
 *   1. `~/.arkeles/hermes.token`  — kullanıcının ARKELÉS için ayırdığı token
 *   2. `HERMES_DASHBOARD_SESSION_TOKEN` ortam değişkeni
 *
 * Dosya yolu TERCİH EDİLİR çünkü izinleri doğrulanabilir. 0600 değilse
 * KULLANILMAZ: dünyaya okunabilir bir sır, sır değildir.
 */
pub fn token_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".arkeles").join("hermes.token"))
}

pub fn read_token() -> Option<String> {
    if let Some(token) = read_token_file() {
        return Some(token);
    }
    std::env::var("HERMES_DASHBOARD_SESSION_TOKEN")
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn read_token_file() -> Option<String> {
    let path = token_path()?;
    let metadata = std::fs::metadata(&path).ok()?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        // Madde 19.2: grup veya diğerleri erişebiliyorsa REDDET.
        if metadata.permissions().mode() & 0o077 != 0 {
            return None;
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;

    let token = std::fs::read_to_string(&path).ok()?.trim().to_string();
    if token.is_empty() {
        return None;
    }
    Some(token)
}

/*
 * YETENEK ALLOWLIST — Anayasa madde 18.2, Sprint 4 madde 19.
 *
 * "keyfi action adı çalıştırılamıyor."
 *
 * ARKELÉS yalnız BU listedeki aksiyonları çağırabilir. Liste derleme
 * zamanında sabittir; arayüzden gelen bir dizge bu listede yoksa çağrı
 * çekirdekte reddedilir — Hermes'e hiç ulaşmaz.
 */
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActionDef {
    /// ARKELÉS'in kanonik aksiyon adı.
    pub id: &'static str,
    /// Arayüzde görünen ad.
    pub label: &'static str,
    /// Bu aksiyonun görünmesi için Hermes'in ilan etmesi gereken yetenek.
    pub capability: &'static str,
}

pub const ACTIONS: &[ActionDef] = &[
    ActionDef {
        id: "task.execute",
        label: "Hermes'e görev ver",
        capability: "task.execute",
    },
    ActionDef {
        id: "note.create",
        label: "Yeni not oluştur",
        capability: "note.create",
    },
    ActionDef {
        id: "report.create",
        label: "Rapor oluştur",
        capability: "report.create",
    },
    ActionDef {
        id: "analysis.create",
        label: "Analiz iste",
        capability: "analysis.create",
    },
];

pub fn find_action(id: &str) -> Option<&'static ActionDef> {
    ACTIONS.iter().find(|a| a.id == id)
}

/*
 * YETENEK OKUMA — Anayasa madde 18.2. SPRINT 5'TE DÜZELTİLDİ.
 *
 * "ARKELÉS, Hermes'in bildirmediği yeteneği UI'da gösteremez."
 *
 * SPRINT 4 İHLALİ (kaldırıldı): o sürüm, Hermes açıkça yetenek listesi
 * döndürmediğinde "ajan sağlıklı → istem aksiyonları muhtemelen mümkün"
 * diye DÖRT yetenek VARSAYIYORDU. Bu, sağlık bilgisini (erişilebilirlik)
 * yetki bilgisiyle (yetenek) karıştırmaktı ve madde 18.2'yi doğrudan
 * ihlal ediyordu.
 *
 * ŞİMDİKİ KURAL — tek kaynak, çıkarsama yok:
 *   Hermes cevabında açık bir `capabilities` dizisi VARSA → o kullanılır.
 *   YOKSA → boş liste. Semantik aksiyonlar görünmez.
 *
 * Sağlık (`gateway.running`, `components.*.status`) burada OKUNMAZ bile —
 * okunursa biri ileride onu tekrar yeteneğe çevirmeye çalışır.
 */
pub fn declared_capabilities(status: &serde_json::Value) -> Vec<String> {
    status
        .get("capabilities")
        .and_then(|v| v.as_array())
        .map(|list| {
            list.iter()
                .filter_map(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

/*
 * GATEWAY YETENEKLERİ — Sprint 5.
 *
 * Hermes'in gerçek ilan kanalı `gateway.capabilities` RPC'sidir; cevap
 * `{ad: bool}` sözlüğüdür (canlı 0.21.0: yalnız
 * `per_session_exclusive_submit`). Yalnız AÇIKÇA `true` olan adlar
 * ilan sayılır — yine ÇIKARSAMA YOK.
 */
pub fn declared_flags(result: &serde_json::Value) -> Vec<String> {
    result
        .as_object()
        .map(|map| {
            map.iter()
                .filter(|(_, value)| value.as_bool() == Some(true))
                .map(|(name, _)| name.trim().to_string())
                .filter(|name| !name.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/*
 * İSTEM METNİ — Sprint 5.
 *
 * Gerçek Hermes'in iş kabul eden yüzeyi serbest metin istemidir
 * (`prompt.submit`); yapısal "aksiyon" parametresi YOKTUR. Aksiyon ve
 * çalışma alanı bu yüzden metnin başlığına yazılır. İkisi de allowlist'ten
 * gelir; kullanıcı metni hiçbir komut satırına girmez, düz metin gider.
 */
pub fn session_title(action: &ActionDef, workspace: Option<&str>) -> String {
    match workspace {
        Some(workspace) => format!("ARKELÉS · {} · {workspace}", action.label),
        None => format!("ARKELÉS · {}", action.label),
    }
}

pub fn prompt_for(action: &ActionDef, workspace: Option<&str>, input: &str) -> String {
    let mut text = format!("[ARKELÉS isteği — {}]\n", action.label);
    if let Some(workspace) = workspace {
        text.push_str(&format!("Çalışma alanı: {workspace}\n"));
    }
    text.push('\n');
    text.push_str(input.trim());
    text
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn aksiyon_allowlist_disi_reddedilir() {
        // Sprint 4 madde 19: "keyfi action adı çalıştırılamıyor."
        assert!(find_action("task.execute").is_some());
        assert!(find_action("rm -rf /").is_none());
        assert!(find_action("shell.exec").is_none());
        assert!(find_action("").is_none());
        assert!(find_action("task.execute; drop table").is_none());
    }

    #[test]
    fn aksiyon_kimlikleri_tekil() {
        let mut ids: Vec<&str> = ACTIONS.iter().map(|a| a.id).collect();
        let count = ids.len();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), count);
    }

    #[test]
    fn acikca_ilan_edilen_yetenek_kullanilir() {
        let status = json!({"capabilities": ["pdf.create", "telegram.send"]});
        assert_eq!(declared_capabilities(&status), vec!["pdf.create", "telegram.send"]);
    }

    /*
     * GERİLEME TESTİ — Sprint 4 anayasa ihlali.
     *
     * Sağlıklı ajan / çalışan gateway YETENEK DEĞİLDİR. Bu testler o
     * çıkarsamanın geri gelmesini engeller (madde 18.2).
     */
    #[test]
    fn saglikli_ajan_yetenek_uretmez() {
        let status = json!({"components": {"agent": {"status": "ok"}}});
        assert!(declared_capabilities(&status).is_empty());
    }

    #[test]
    fn calisan_gateway_yetenek_uretmez() {
        let status = json!({"gateway": {"running": true}});
        assert!(declared_capabilities(&status).is_empty());
    }

    #[test]
    fn saglik_ve_bos_liste_birlikte_de_yetenek_uretmez() {
        let status = json!({
            "capabilities": [],
            "gateway": {"running": true},
            "components": {"agent": {"status": "ok"}}
        });
        assert!(declared_capabilities(&status).is_empty());
    }

    #[test]
    fn dizge_olmayan_ve_bos_ogeler_atlanir() {
        let status = json!({"capabilities": ["task.execute", 5, null, "", "  "]});
        assert_eq!(declared_capabilities(&status), vec!["task.execute"]);
    }

    #[test]
    fn bozuk_status_panik_yapmaz() {
        for value in [json!(null), json!("metin"), json!(5), json!([1, 2])] {
            assert!(declared_capabilities(&value).is_empty());
        }
    }

    #[test]
    fn gateway_bayraklari_yalniz_true_olanlar() {
        let flags = declared_flags(&json!({
            "per_session_exclusive_submit": true,
            "kapali": false,
            "dizge": "true",
            "sayi": 1
        }));
        assert_eq!(flags, vec!["per_session_exclusive_submit".to_string()]);
        assert!(declared_flags(&json!(null)).is_empty());
        assert!(declared_flags(&json!(["task.execute"])).is_empty());
    }

    #[test]
    fn gercek_hermes_bayragi_semantik_aksiyon_acmaz() {
        // Canlı 0.21.0 cevabı: hiçbir allowlist aksiyonuyla eşleşmez.
        let flags = declared_flags(&json!({"per_session_exclusive_submit": true}));
        assert!(ACTIONS.iter().all(|a| !flags.iter().any(|f| f == a.capability)));
    }

    #[test]
    fn istem_aksiyonu_alani_ve_metni_tasir() {
        let action = find_action("report.create").unwrap();
        let prompt = prompt_for(action, Some("wif"), "  Haftalık bütçe  ");
        assert!(prompt.starts_with("[ARKELÉS isteği — Rapor oluştur]"));
        assert!(prompt.contains("Çalışma alanı: wif"));
        assert!(prompt.ends_with("Haftalık bütçe"));
        assert_eq!(session_title(action, None), "ARKELÉS · Rapor oluştur");
    }
}
