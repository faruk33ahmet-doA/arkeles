/*!
Webview'e çıkan tipler — Anayasa madde 14.2.

"Elle yazılmış IPC tip tanımı yasaktır. Sözleşme tek yerde yaşar."

Bu modüldeki her tip `ts-rs` ile `src/lib/generated/` altına TypeScript
olarak üretilir. Üretim komutu:  cargo test export_bindings

Alan adları `camelCase`'e çevrilir çünkü tüketici TypeScript.
*/

use serde::Serialize;
use ts_rs::TS;

/// Anayasa madde 17.3: vault yolu yapılandırılmamış olabilir.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct VaultStatus {
    pub path: Option<String>,
    /// Madde 11.5: SAYIM analiz değildir.
    pub note_count: u32,
    pub indexed_at: Option<String>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct IndexStatus {
    pub ready: bool,
    /// Madde 15.5: sürüm değişince index otomatik yeniden kurulur.
    /// `u32`: ts-rs `i64`'ü `bigint`'e eşler; şema sürümü için gereksiz ağırlık.
    pub schema_version: u32,
    pub rebuilding: bool,
    /// Son taramanın süresi — performans bütçesi ölçümü (madde 34.2).
    pub last_scan_ms: Option<u32>,
    /// Sprint 1 borcu #4: sessizce yutulan satır hataları artık GÖRÜNÜR.
    /// 0'dan büyükse index'te bozuk veri var demektir — sakin ama görünür.
    pub row_errors: u32,
}

/// Anayasa madde 18.1 sözleşmesi.
#[derive(Debug, Clone, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct HermesHealth {
    /// Madde 18.4: `false` bir HATA DEĞİL, sakin bir durumdur.
    pub reachable: bool,
    pub version: Option<String>,
    /// Madde 18.2: arayüz yalnız bu listedeki yetenekleri gösterir.
    pub capabilities: Vec<String>,
}

/// Madde 8.3: durum değişimi mekanik mutasyondur (Sprint 2).
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub title: String,
    /// "open" | "done"
    pub status: String,
    pub note_id: String,
    pub workspace: Option<String>,
    pub due: Option<String>,
    /// Madde 16.3: yönetilmeyen notun görevine mekanik mutasyon uygulanmaz.
    pub managed: bool,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct NoteSummary {
    pub id: String,
    pub title: String,
    pub modified_at: String,
    pub managed: bool,
}

/// Bugün görünümü — Anayasa madde 36.2.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct TodayView {
    /// Vadesi GEÇMİŞ açık görevler.
    pub overdue: Vec<Task>,
    /// Bugün vadesi olan görevler.
    pub due: Vec<Task>,
    /// Bugün oluşturulan veya değiştirilen notlar.
    pub touched_notes: Vec<NoteSummary>,
}

/// Panel özeti — Anayasa madde 24.2.
#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
    pub name: String,
    pub open_tasks: u32,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct LifeScore {
    pub value: f64,
    /// Madde 11.3: skor tazeliğiyle birlikte saklanır.
    pub computed_at: String,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct DashboardView {
    /// Madde 11.1/11.2: HERMES hesaplar, ARKELÉS yalnız okur.
    /// Hermes hiç hesaplamadıysa `None` — hata değil, henüz yok.
    pub life_score: Option<LifeScore>,
    /// Madde 11.5: sayım. Analiz değil.
    pub workspaces: Vec<WorkspaceSummary>,
    pub critical_tasks: Vec<Task>,
}

#[derive(Debug, Serialize, TS)]
#[ts(export, export_to = "../../src/lib/generated/")]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub note_id: String,
    pub title: String,
    /// Eşleşen bağlam parçası. Madde 19.5: yalnız arayüze gider, log'a GİTMEZ.
    pub snippet: String,
}
