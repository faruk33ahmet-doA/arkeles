/*!
Hermes IPC komutları — Anayasa madde 12, 18, 19.3; Sprint 4.

Webview yalnız komut ADI ve dar parametreler gönderir; adres, token ve
ağ işi Rust'ta kalır. Aksiyon adı ALLOWLIST'ten doğrulanır — keyfi bir
dizge Hermes'e ulaşamaz (Sprint 4 madde 19).
*/

use tauri::State;

use crate::error::CoreResult;
use crate::hermes::capabilities::ArkelesSemanticCapabilities;
use crate::hermes::{self, contract};
use crate::types::{AvailableAction, HermesHealth, HermesSummary, Job};
use crate::AppState;

/// Anayasa madde 18.1: sürüm + yetenek listesi.
///
/// HATA DÖNDÜRMEZ (madde 18.4). `async` çünkü ağ işi IPC iş parçacığını
/// bloke etmemeli — Sprint 4 madde 21: "Hermes bağlantısı ana UI'yı
/// yavaşlatamaz."
#[tauri::command]
pub async fn hermes_health() -> HermesHealth {
    tauri::async_runtime::spawn_blocking(hermes::health)
        .await
        .unwrap_or_else(|_| HermesHealth::unreachable())
}

/// Dashboard özeti — Sprint 4 madde 4, 13.
#[tauri::command]
pub async fn hermes_summary(state: State<'_, AppState>) -> CoreResult<HermesSummary> {
    let health = tauri::async_runtime::spawn_blocking(hermes::health)
        .await
        .unwrap_or_else(|_| HermesHealth::unreachable());

    state.index.hermes_summary(health)
}

/*
 * Kullanılabilir aksiyonlar — Anayasa madde 18.2.
 *
 * "Hermes'in bildirmediği yeteneği arayüzde HİÇ göstermez."
 *
 * Allowlist ∩ Hermes'in ilan ettikleri. Kesişim boşsa arayüzde hiçbir
 * semantik aksiyon görünmez ve bu SAKİN bir durumdur (madde 18.4).
 */
#[tauri::command]
pub async fn hermes_actions() -> Vec<AvailableAction> {
    let health = tauri::async_runtime::spawn_blocking(hermes::health)
        .await
        .unwrap_or_else(|_| HermesHealth::unreachable());

    if !health.reachable {
        return Vec::new();
    }

    let semantic = ArkelesSemanticCapabilities {
        items: health.capabilities,
    };

    contract::ACTIONS
        .iter()
        .filter(|action| semantic.supports(action.required_capabilities))
        .map(|action| AvailableAction {
            id: action.id.to_string(),
            label: action.label.to_string(),
            required_capabilities: action
                .required_capabilities
                .iter()
                .map(|value| value.to_string())
                .collect(),
        })
        .collect()
}

/*
 * Semantik iş gönderir — Sprint 4 madde 2, 6.
 *
 * OPTİMİSTİK: iş deftere QUEUED olarak yazılır ve HEMEN döner. Hermes'e
 * iletim arka planda olur — "Hermes cevabı beklenmez" (madde 6).
 */
#[tauri::command]
pub async fn submit_job(
    state: State<'_, AppState>,
    action: String,
    workspace: Option<String>,
    input: String,
) -> CoreResult<Job> {
    state.index.submit_job(&action, workspace.as_deref(), &input)
}

/// Aktif işler (queued + running).
#[tauri::command]
pub fn active_jobs(state: State<'_, AppState>) -> CoreResult<Vec<Job>> {
    state.index.active_jobs()
}

/// Aktivite geçmişi — Sprint 4 madde 14.
#[tauri::command]
pub fn job_history(
    state: State<'_, AppState>,
    since: String,
    workspace: Option<String>,
    status: Option<String>,
) -> CoreResult<Vec<Job>> {
    state
        .index
        .job_history(&since, workspace.as_deref(), status.as_deref())
}

/// İşi yeniden kuyruğa alır — madde 15. OTOMATİK DEĞİL, kullanıcı ister.
#[tauri::command]
pub fn retry_job(state: State<'_, AppState>, job_id: String) -> CoreResult<()> {
    state.index.retry_job(&job_id)
}

/// İşi iptal eder — madde 16. Çalışan turda Hermes'e ağ çağrısı gider;
/// IPC iş parçacığını bloke etmemek için ayrı iş parçacığında (madde 21).
#[tauri::command]
pub async fn cancel_job(app: tauri::AppHandle, job_id: String) -> CoreResult<()> {
    tauri::async_runtime::spawn_blocking(move || {
        use tauri::Manager;
        app.state::<AppState>().index.cancel_job(&job_id)
    })
    .await
    .unwrap_or(Err(crate::error::CoreError::HermesUnreachable))
}
