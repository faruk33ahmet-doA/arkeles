# ADR-0001 — Hermes-native capability adapter

**Durum:** Kabul edildi
**Tarih:** 2026-09-11
**İlgili anayasa maddeleri:** 4, 12, 13, 18, 19

## Bağlam

Mac'teki canlı Hermes 0.21.0 kurulumu iki farklı HTTP yüzeyi tanımlıyor:

- `hermes serve`, `127.0.0.1:9119`: dashboard backend'i; `/api/health`,
  `/api/status`, `/api/tools/toolsets`, `/api/skills` ve `/api/ws` JSON-RPC.
- Gateway `api_server` platformu, varsayılan `127.0.0.1:8642`: etkin olduğunda
  `/v1/capabilities`, `/v1/toolsets`, `/v1/skills`, `/v1/runs` sunuyor.

Canlı doğrulamada yalnız 9119 dinliyor. `/v1/*` yolları 9119'da 404 dönüyor,
8642'de dinleyen süreç yok ve mevcut `api_server` platformu etkin değil. Buna
karşılık 9119'un gerçek iş akışı `session.create → prompt.submit →
message.complete`; iptal akışı `session.interrupt` olarak çalışıyor.

Önceki capability kapısı ARKELÉS'in `task.execute`, `note.create`,
`report.create`, `analysis.create` adlarını Hermes'in aynen ilan etmesini
bekliyordu. Bunlar Hermes-native capability adları değil, ARKELÉS semantik
aksiyonlarıdır.

## Karar

ARKELÉS tek canlı entegrasyon olarak 9119 sözleşmesini korur. Ayrı API server
etkinleştirilmez ve paralel `/v1/runs` sürücüsü eklenmez.

`HermesCapabilitySnapshot → ArkelesSemanticCapabilities` dönüşümü tek adapter
modülünde, fail-closed kurallarla yapılır:

| ARKELÉS semantik yeteneği | Zorunlu explicit Hermes kaynağı |
|---|---|
| `run.submit` | `run_submission`, `session_chat`, `session_chat_streaming` veya canlı gateway'in `per_session_exclusive_submit=true` ilanı |
| `file.output` | `file` toolset'i `enabled=true`, `configured=true` ve `write_file` veya `patch` aracı |
| `session.search` | `session_search` toolset'i açık/yapılandırılmış ve `session_search` aracı |
| `skills.use` | `skills` toolset'i açık/yapılandırılmış ve `/api/skills` içinde en az bir açık skill |

ARKELÉS aksiyon gereksinimleri:

- Görev ve analiz: `run.submit`
- Yeni not ve rapor: `run.submit + file.output`

Health, reachability, provider/model durumu veya yalnız bileşen sağlığı hiçbir
semantik yetenek açmaz. Bozuk ya da eksik payload güvenli biçimde kapalıdır.

## Sonuçlar

- Capability sahteciliği kalkar; ARKELÉS adları Hermes'ten beklenmez.
- 7 kurum tek Hermes motoruna workspace bağlamı olarak gitmeye devam eder.
- Provider/model ve Obsidian vault/index/watcher/safe-write yapıları değişmez.
- UI yalnız kullanıcı dostu etiketleri gösterir; teknik kaynaklar IPC verisinde
  denetlenebilir kalır fakat kullanıcıya dökülmez.
