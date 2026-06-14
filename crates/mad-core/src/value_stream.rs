use std::collections::HashMap;

use serde::{Deserialize, Deserializer, Serialize};

/// Per-vendor value stream map (VSM): process flow with nodes, edges, and messages.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct ValueStreamMap {
    #[serde(default)]
    pub nodes: Vec<VsmNode>,
    #[serde(default)]
    pub edges: Vec<VsmEdge>,
    #[serde(default)]
    pub messages: Vec<VsmMessage>,
    #[serde(default)]
    pub flow_types: Vec<VsmFlowTypeDef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum VsmNodeType {
    Process,
    Decision,
    Info,
    Delay,
    External,
    Customer,
    Supplier,
    Inventory,
    Kaizen,
}

impl Default for VsmNodeType {
    fn default() -> Self {
        Self::Process
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmFlowTypeDef {
    pub id: String,
    pub label: String,
    pub color: String,
    #[serde(default)]
    pub dash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmNode {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub node_type: VsmNodeType,
    pub x: f64,
    pub y: f64,
    #[serde(default = "default_node_width")]
    pub width: f64,
    #[serde(default = "default_node_height")]
    pub height: f64,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub lead_time_minutes: Option<f64>,
    #[serde(default)]
    pub cycle_time_minutes: Option<f64>,
    #[serde(default)]
    pub author: Option<String>,
}

fn default_node_width() -> f64 {
    180.0
}

fn default_node_height() -> f64 {
    72.0
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmEdge {
    pub id: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default = "default_edge_type")]
    pub edge_type: String,
    #[serde(default)]
    pub duration_minutes: Option<f64>,
    /// Decision branch handle: `yes` (bottom) or `no` (right).
    #[serde(default)]
    pub source_handle: Option<String>,
}

fn default_edge_type() -> String {
    "material".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VsmMessage {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub edge_id: Option<String>,
}

/// Named value stream attached to a vendor (a vendor may have several).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ValueStreamEntry {
    pub id: String,
    pub name: String,
    #[serde(flatten)]
    pub map: ValueStreamMap,
}

impl ValueStreamEntry {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            id: new_value_stream_id(),
            name: name.into(),
            map: ValueStreamMap::default(),
        }
    }

    pub fn from_legacy(map: ValueStreamMap) -> Self {
        Self {
            id: "default".into(),
            name: "Value stream".into(),
            map,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.map.nodes.is_empty() && self.map.edges.is_empty() && self.map.messages.is_empty()
    }
}

pub fn new_value_stream_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let millis = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    format!("vsm-{millis}")
}

/// Deserializes vendor → streams, accepting legacy single-map JSON per vendor.
pub fn deserialize_vendor_value_streams<'de, D>(
    deserializer: D,
) -> Result<HashMap<String, Vec<ValueStreamEntry>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw: HashMap<String, serde_json::Value> =
        HashMap::deserialize(deserializer).unwrap_or_default();
    Ok(raw
        .into_iter()
        .map(|(vendor_id, value)| (vendor_id, parse_vendor_streams(value)))
        .collect())
}

fn parse_vendor_streams(value: serde_json::Value) -> Vec<ValueStreamEntry> {
    match value {
        serde_json::Value::Array(items) => items
            .into_iter()
            .filter_map(|item| serde_json::from_value::<ValueStreamEntry>(item).ok())
            .collect(),
        serde_json::Value::Object(obj) => {
            if obj.contains_key("id") && obj.contains_key("name") {
                serde_json::from_value(serde_json::Value::Object(obj))
                    .ok()
                    .into_iter()
                    .collect()
            } else {
                let map: ValueStreamMap =
                    serde_json::from_value(serde_json::Value::Object(obj)).unwrap_or_default();
                if map.nodes.is_empty() && map.edges.is_empty() && map.messages.is_empty() {
                    vec![]
                } else {
                    vec![ValueStreamEntry::from_legacy(map)]
                }
            }
        }
        _ => vec![],
    }
}
