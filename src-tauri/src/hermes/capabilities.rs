/*!
Hermes-native ilanlar -> ARKELÉS semantik yetenekleri.

Bu modül capability dönüşümünün TEK yeridir. Sağlık veya erişilebilirlikten
yetenek çıkarmaz. Kaynakların tamamı Hermes'in açık ilanlarıdır:

  - `/api/status.capabilities` ya da aynı biçimdeki `features` bayrakları
  - `gateway.capabilities` JSON-RPC bayrakları
  - `/api/tools/toolsets` içindeki enabled + configured toolset/tool adları
  - `/api/skills` içindeki açık beceriler

Canlı macOS Hermes 0.21.0'da iş gönderme kanıtı
`gateway.capabilities.per_session_exclusive_submit=true` bayrağıdır. Bu bayrak
Hermes kaynak kodunda `prompt.submit` istemcisinin kullanmadan önce görmesi
gereken, uygulanan bir capability olarak tanımlıdır. ARKELÉS bunu yalnız
`run.submit` semantik yeteneğine eşler; health/status bilgisini kullanmaz.
*/

use std::collections::BTreeSet;

use serde_json::Value;

use crate::types::HermesCapability;

pub const RUN_SUBMIT: &str = "run.submit";
pub const FILE_OUTPUT: &str = "file.output";
pub const SESSION_SEARCH: &str = "session.search";
pub const SKILLS_USE: &str = "skills.use";

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct HermesCapabilitySnapshot {
    features: BTreeSet<String>,
    gateway_flags: BTreeSet<String>,
    toolsets: Vec<Toolset>,
    enabled_skills: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Toolset {
    name: String,
    tools: BTreeSet<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArkelesSemanticCapabilities {
    pub items: Vec<HermesCapability>,
}

impl HermesCapabilitySnapshot {
    pub fn from_values(
        status: Option<&Value>,
        gateway_flags: Option<&Value>,
        toolsets: Option<&Value>,
        skills: Option<&Value>,
    ) -> Self {
        Self {
            features: declared_features(status),
            gateway_flags: declared_true_flags(gateway_flags),
            toolsets: declared_toolsets(toolsets),
            enabled_skills: declared_enabled_skills(skills),
        }
    }

    fn run_sources(&self) -> Vec<String> {
        let mut sources = Vec::new();
        for feature in ["run_submission", "session_chat", "session_chat_streaming"] {
            if self.features.contains(feature) {
                sources.push(format!("capabilities.{feature}"));
            }
        }
        if self.gateway_flags.contains("per_session_exclusive_submit") {
            sources.push("gateway.capabilities.per_session_exclusive_submit".into());
        }
        sources
    }

    fn tool_sources(&self, toolset_name: &str, accepted_tools: &[&str]) -> Vec<String> {
        let Some(toolset) = self.toolsets.iter().find(|item| item.name == toolset_name) else {
            return Vec::new();
        };

        accepted_tools
            .iter()
            .filter(|tool| toolset.tools.contains(**tool))
            .map(|tool| format!("toolsets.{toolset_name}.{tool}"))
            .collect()
    }
}

impl ArkelesSemanticCapabilities {
    pub fn from_snapshot(snapshot: &HermesCapabilitySnapshot) -> Self {
        let mut items = Vec::new();

        push_if_declared(
            &mut items,
            RUN_SUBMIT,
            "Görev çalıştırabilir",
            snapshot.run_sources(),
        );
        push_if_declared(
            &mut items,
            FILE_OUTPUT,
            "Dosya çıktısı üretebilir",
            snapshot.tool_sources("file", &["write_file", "patch"]),
        );
        push_if_declared(
            &mut items,
            SESSION_SEARCH,
            "Geçmiş konuşmaları arayabilir",
            snapshot.tool_sources("session_search", &["session_search"]),
        );

        let mut skill_sources =
            snapshot.tool_sources("skills", &["skills_list", "skill_view", "skill_manage"]);
        if snapshot.enabled_skills.is_empty() {
            skill_sources.clear();
        } else {
            skill_sources.push("skills.enabled".into());
        }
        push_if_declared(
            &mut items,
            SKILLS_USE,
            "Hazır becerileri kullanabilir",
            skill_sources,
        );

        Self { items }
    }

    pub fn supports(&self, required: &[&str]) -> bool {
        required
            .iter()
            .all(|id| self.items.iter().any(|capability| capability.id == *id))
    }
}

fn push_if_declared(
    target: &mut Vec<HermesCapability>,
    id: &str,
    label: &str,
    sources: Vec<String>,
) {
    if sources.is_empty() {
        return;
    }
    target.push(HermesCapability {
        id: id.to_string(),
        label: label.to_string(),
        sources,
    });
}

fn declared_features(value: Option<&Value>) -> BTreeSet<String> {
    let Some(value) = value else {
        return BTreeSet::new();
    };

    let mut features = BTreeSet::new();
    if let Some(list) = value.get("capabilities").and_then(Value::as_array) {
        features.extend(valid_strings(list.iter()));
    }
    if let Some(flags) = value.get("features").and_then(Value::as_object) {
        features.extend(
            flags
                .iter()
                .filter(|(_, enabled)| enabled.as_bool() == Some(true))
                .map(|(name, _)| name.trim())
                .filter(|name| !name.is_empty())
                .map(str::to_string),
        );
    }
    features
}

fn declared_true_flags(value: Option<&Value>) -> BTreeSet<String> {
    value
        .and_then(Value::as_object)
        .map(|flags| {
            flags
                .iter()
                .filter(|(_, enabled)| enabled.as_bool() == Some(true))
                .map(|(name, _)| name.trim())
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn declared_toolsets(value: Option<&Value>) -> Vec<Toolset> {
    payload_array(value)
        .map(|list| {
            list.iter()
                .filter_map(|item| {
                    // Güvenli kapalı: iki bayrak da açıkça true olmalı.
                    if item.get("enabled").and_then(Value::as_bool) != Some(true)
                        || item.get("configured").and_then(Value::as_bool) != Some(true)
                    {
                        return None;
                    }
                    let name = item.get("name")?.as_str()?.trim();
                    if name.is_empty() {
                        return None;
                    }
                    let tools = item.get("tools")?.as_array()?;
                    Some(Toolset {
                        name: name.to_string(),
                        tools: valid_strings(tools.iter()),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn declared_enabled_skills(value: Option<&Value>) -> BTreeSet<String> {
    payload_array(value)
        .map(|list| {
            list.iter()
                .filter(|item| item.get("enabled").and_then(Value::as_bool) == Some(true))
                .filter_map(|item| item.get("name").and_then(Value::as_str))
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn payload_array(value: Option<&Value>) -> Option<&Vec<Value>> {
    value.and_then(|root| {
        root.as_array()
            .or_else(|| root.get("data").and_then(Value::as_array))
    })
}

fn valid_strings<'a>(values: impl Iterator<Item = &'a Value>) -> BTreeSet<String> {
    values
        .filter_map(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn mapped(
        status: Option<&Value>,
        flags: Option<&Value>,
        toolsets: Option<&Value>,
        skills: Option<&Value>,
    ) -> ArkelesSemanticCapabilities {
        ArkelesSemanticCapabilities::from_snapshot(&HermesCapabilitySnapshot::from_values(
            status, flags, toolsets, skills,
        ))
    }

    #[test]
    fn explicit_run_capability_semantik_aksiyonu_acar() {
        let status = json!({"features": {"run_submission": true}});
        let semantic = mapped(Some(&status), None, None, None);
        let action = crate::hermes::contract::find_action("task.execute").unwrap();

        assert!(semantic.supports(action.required_capabilities));
        assert_eq!(semantic.items[0].label, "Görev çalıştırabilir");
        assert_eq!(semantic.items[0].sources, ["capabilities.run_submission"]);
    }

    #[test]
    fn canli_gateway_submit_bayragi_run_yetenegine_eslenir() {
        let flags = json!({"per_session_exclusive_submit": true});
        let semantic = mapped(None, Some(&flags), None, None);

        assert!(semantic.supports(&[RUN_SUBMIT]));
        assert_eq!(
            semantic.items[0].sources,
            ["gateway.capabilities.per_session_exclusive_submit"]
        );
    }

    #[test]
    fn health_var_capability_yoksa_guvenli_kapali() {
        let status = json!({
            "gateway": {"running": true},
            "components": {"agent": {"status": "ok"}}
        });
        assert!(mapped(Some(&status), None, None, None).items.is_empty());
    }

    #[test]
    fn bozuk_capability_payloadlari_guvenli_kapali() {
        let status = json!({"capabilities": "run_submission", "features": ["run_submission"]});
        let flags = json!(["per_session_exclusive_submit"]);
        let toolsets = json!({"data": {"name": "file"}});
        let skills = json!({"data": "skill"});

        assert!(
            mapped(Some(&status), Some(&flags), Some(&toolsets), Some(&skills))
                .items
                .is_empty()
        );
    }

    #[test]
    fn explicit_toolset_mapping_dogru_ve_kapali_toolset_acmaz() {
        let toolsets = json!([
            {"name": "file", "enabled": true, "configured": true,
             "tools": ["read_file", "write_file", "patch"]},
            {"name": "session_search", "enabled": true, "configured": true,
             "tools": ["session_search"]},
            {"name": "skills", "enabled": false, "configured": true,
             "tools": ["skills_list"]}
        ]);
        let semantic = mapped(None, None, Some(&toolsets), None);

        assert!(semantic.supports(&[FILE_OUTPUT, SESSION_SEARCH]));
        assert!(!semantic.supports(&[SKILLS_USE]));
        let file = semantic
            .items
            .iter()
            .find(|item| item.id == FILE_OUTPUT)
            .unwrap();
        assert_eq!(
            file.sources,
            ["toolsets.file.write_file", "toolsets.file.patch"]
        );
    }

    #[test]
    fn note_ve_rapor_icin_run_ve_file_birlikte_gerekir() {
        let status = json!({"features": {"run_submission": true}});
        let without_file = mapped(Some(&status), None, None, None);
        let note = crate::hermes::contract::find_action("note.create").unwrap();
        let report = crate::hermes::contract::find_action("report.create").unwrap();
        assert!(!without_file.supports(note.required_capabilities));
        assert!(!without_file.supports(report.required_capabilities));

        let toolsets = json!({"data": [{
            "name": "file", "enabled": true, "configured": true,
            "tools": ["write_file"]
        }]});
        let with_file = mapped(Some(&status), None, Some(&toolsets), None);
        assert!(with_file.supports(note.required_capabilities));
        assert!(with_file.supports(report.required_capabilities));
    }

    #[test]
    fn skill_yetenegi_toolset_ve_acik_skill_birlikteyken_var() {
        let toolsets = json!([{
            "name": "skills", "enabled": true, "configured": true,
            "tools": ["skills_list", "skill_view"]
        }]);
        let skills = json!([{"name": "research", "enabled": true}]);
        assert!(mapped(None, None, Some(&toolsets), Some(&skills)).supports(&[SKILLS_USE]));
    }
}
