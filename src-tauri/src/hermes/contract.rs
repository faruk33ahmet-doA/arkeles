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
 * Hermes'in bildirdiklerinden YETENEK TÜRETME.
 *
 * Hermes yetenekleri bizim adlandırmamızla ilan etmiyor; `/api/status`
 * bileşen sağlığı ve yapılandırılmış platformlar döndürüyor. Bu fonksiyon
 * en ince adapter: Hermes'in gerçek cevabını bizim yetenek adlarımıza
 * çevirir.
 *
 * VARSAYIM YAPMAZ: Hermes açıkça bir `capabilities` dizisi döndürüyorsa O
 * kullanılır. Döndürmüyorsa, ajanın çalışır olması `task.*` ailesini
 * mümkün kılar — çünkü Hermes'in TEK ve ASIL yeteneği budur: bir istem alıp
 * iş yapmak. Bunun ötesinde bir şey ÇIKARSANMAZ.
 */
pub fn derive_capabilities(status: &serde_json::Value) -> Vec<String> {
    // 1) Hermes açıkça ilan ediyorsa onu kullan (ileri uyumluluk).
    if let Some(list) = status.get("capabilities").and_then(|v| v.as_array()) {
        let declared: Vec<String> = list
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::to_string)
            .collect();
        if !declared.is_empty() {
            return declared;
        }
    }

    // 2) Aksi halde: ajan sağlıklıysa istem tabanlı aksiyonlar mümkündür.
    let agent_ok = status
        .get("components")
        .and_then(|c| c.as_object())
        .map(|components| {
            components.values().any(|v| {
                v.get("status")
                    .and_then(|s| s.as_str())
                    .is_some_and(|s| s == "ok" || s == "healthy" || s == "running")
            })
        })
        .unwrap_or(false);

    let gateway_ok = status
        .get("gateway")
        .and_then(|g| g.get("running"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if agent_ok || gateway_ok {
        // Bu dördü tek bir mekanizmanın (ajana istem gönderme) yüzleridir.
        return vec![
            "task.execute".into(),
            "note.create".into(),
            "report.create".into(),
            "analysis.create".into(),
        ];
    }

    Vec::new()
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
    fn hermes_acikca_ilan_ederse_o_kullanilir() {
        let status = json!({"capabilities": ["pdf.create", "telegram.send"]});
        assert_eq!(derive_capabilities(&status), vec!["pdf.create", "telegram.send"]);
    }

    #[test]
    fn saglikli_bilesen_istem_yeteneklerini_acar() {
        let status = json!({"components": {"agent": {"status": "ok"}}});
        let caps = derive_capabilities(&status);
        assert!(caps.contains(&"task.execute".to_string()));
        assert!(caps.contains(&"report.create".to_string()));
    }

    #[test]
    fn calisan_gateway_de_yeterlidir() {
        let status = json!({"gateway": {"running": true}});
        assert!(derive_capabilities(&status).contains(&"task.execute".to_string()));
    }

    #[test]
    fn saglik_yoksa_yetenek_yok() {
        // Madde 18.2: bildirilmeyen yetenek arayüzde HİÇ görünmez.
        assert!(derive_capabilities(&json!({})).is_empty());
        assert!(derive_capabilities(&json!({"components": {"agent": {"status": "down"}}})).is_empty());
        assert!(derive_capabilities(&json!({"gateway": {"running": false}})).is_empty());
    }

    #[test]
    fn bos_capabilities_dizisi_saglik_yoluna_duser() {
        let status = json!({"capabilities": [], "gateway": {"running": true}});
        assert!(derive_capabilities(&status).contains(&"task.execute".to_string()));
    }

    #[test]
    fn bozuk_status_panik_yapmaz() {
        for value in [json!(null), json!("metin"), json!(5), json!([1, 2])] {
            assert!(derive_capabilities(&value).is_empty());
        }
    }
}
